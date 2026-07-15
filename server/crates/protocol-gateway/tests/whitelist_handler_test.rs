mod support;

use common::code::ResultCode;
use common::proto::{CmdAddWhitelist, RspCommon};
use prost::Message;
use protocol_gateway::codec;
use protocol_gateway::handlers::whitelist::handle_add_whitelist;
use std::sync::mpsc;
use std::sync::{Arc, Barrier, RwLock, TryLockError};
use std::thread;
use std::time::{Duration, Instant};
use support::{request_fixture, RequestFixture};

use common::types::DeviceType;
use usb_identify::descriptor::UsbDeviceInfo;
use usb_identify::monitor::DeviceManager;
use whitelist::WhitelistManager;

fn device(device_type: DeviceType, interface_class: u8) -> UsbDeviceInfo {
    UsbDeviceInfo {
        sys_path: "/sys/real".into(),
        dev_path: Some("/dev/sda1".into()),
        serial_number: "REAL-SN".into(),
        vid: "0951".into(),
        pid: "1666".into(),
        device_name: "Real USB Disk".into(),
        device_type,
        interface_class,
        interface_subclass: 0x06,
        interface_protocol: 0x50,
        capacity_bytes: Some(4096),
    }
}

fn context(device: Option<UsbDeviceInfo>) -> RequestFixture {
    let mut fixture = request_fixture(9);
    let whitelist = Arc::new(WhitelistManager::new(Arc::clone(&fixture.storage)).unwrap());
    let mut manager = DeviceManager::new();
    if let Some(device) = device {
        manager.add_interface(device);
    }
    fixture.context.whitelist_manager = Some(whitelist);
    fixture.context.device_manager = Some(Arc::new(RwLock::new(manager)));
    fixture
}

fn command(add_method: &str) -> CmdAddWhitelist {
    CmdAddWhitelist {
        session_token: String::new(),
        serial_number: "REAL-SN".into(),
        vid: "FAKE-VID".into(),
        pid: "FAKE-PID".into(),
        device_name: "Fake Name".into(),
        capacity_bytes: 999,
        permission: "readonly".into(),
        description: "test".into(),
        add_method: add_method.into(),
        device_type: "keyboard".into(),
    }
}

fn decode_common(response: &[u8]) -> RspCommon {
    let (_, payload, _) = codec::try_decode_frame(response).unwrap().unwrap();
    RspCommon::decode(payload.as_slice()).unwrap()
}

#[test]
fn device_add_rejects_device_removed_after_listing() {
    let fixture = context(Some(device(DeviceType::Storage, 0x08)));
    let ctx = fixture.context;
    ctx.device_manager
        .as_ref()
        .unwrap()
        .write()
        .unwrap()
        .remove_interface("/sys/real");
    let rsp = decode_common(&handle_add_whitelist(
        &ctx,
        &command("device").encode_to_vec(),
    ));

    assert!(!rsp.success);
    assert_eq!(
        rsp.result_code,
        ResultCode::ValidationFailed.as_u16() as i32
    );
    assert_eq!(rsp.error_message, "设备已移除，请重新插入后再添加");
    assert!(ctx
        .whitelist_manager
        .as_ref()
        .unwrap()
        .query_by_sn("REAL-SN")
        .unwrap()
        .is_none());
}

#[test]
fn device_add_uses_current_device_identification_fields() {
    let fixture = context(Some(device(DeviceType::Storage, 0x08)));
    let ctx = fixture.context;
    let rsp = decode_common(&handle_add_whitelist(
        &ctx,
        &command("device").encode_to_vec(),
    ));
    assert!(rsp.success);

    let item = ctx
        .whitelist_manager
        .as_ref()
        .unwrap()
        .query_by_sn("REAL-SN")
        .unwrap()
        .unwrap();
    assert_eq!(item.vid.as_deref(), Some("0951"));
    assert_eq!(item.pid.as_deref(), Some("1666"));
    assert_eq!(item.device_name.as_deref(), Some("Real USB Disk"));
    assert_eq!(item.capacity_bytes, Some(4096));
    assert_eq!(item.device_type, "storage");
}

#[test]
fn management_add_accepts_empty_vid_and_pid_for_storage() {
    let fixture = context(None);
    let ctx = fixture.context;
    let mut cmd = command("management");
    cmd.serial_number = "MGMT-SN".into();
    cmd.vid.clear();
    cmd.pid.clear();
    cmd.device_type = "storage".into();

    let rsp = decode_common(&handle_add_whitelist(&ctx, &cmd.encode_to_vec()));
    assert!(rsp.success);
    let item = ctx
        .whitelist_manager
        .as_ref()
        .unwrap()
        .query_by_sn("MGMT-SN")
        .unwrap()
        .unwrap();
    assert!(item.vid.is_none());
    assert!(item.pid.is_none());
}

#[test]
fn management_add_rejects_non_storage_device_type() {
    let fixture = context(None);
    let ctx = fixture.context;
    let rsp = decode_common(&handle_add_whitelist(
        &ctx,
        &command("management").encode_to_vec(),
    ));

    assert!(!rsp.success);
    assert_eq!(
        rsp.result_code,
        ResultCode::DeviceNotStorage.as_u16() as i32
    );
    assert_eq!(rsp.error_message, "仅支持添加大容量存储设备");
}

#[test]
fn management_duplicate_uses_standard_already_exists_message() {
    let fixture = context(None);
    let ctx = fixture.context;
    let mut cmd = command("management");
    cmd.device_type = "storage".into();

    assert!(decode_common(&handle_add_whitelist(&ctx, &cmd.encode_to_vec())).success);
    let duplicate = decode_common(&handle_add_whitelist(&ctx, &cmd.encode_to_vec()));

    assert!(!duplicate.success);
    assert_eq!(
        duplicate.result_code,
        ResultCode::AlreadyExists.as_u16() as i32
    );
    assert_eq!(duplicate.error_message, "该设备已在白名单中");
    assert_eq!(
        ctx.whitelist_manager
            .as_ref()
            .unwrap()
            .query_all()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn concurrent_device_duplicates_are_unique_and_use_standard_message() {
    let fixture = context(Some(device(DeviceType::Storage, 0x08)));
    let ctx = fixture.context;
    let ctx = Arc::new(ctx);
    let barrier = Arc::new(Barrier::new(3));

    let threads = (0..2)
        .map(|_| {
            let ctx = Arc::clone(&ctx);
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier.wait();
                decode_common(&handle_add_whitelist(
                    &ctx,
                    &command("device").encode_to_vec(),
                ))
            })
        })
        .collect::<Vec<_>>();
    barrier.wait();
    let responses = threads
        .into_iter()
        .map(|thread| thread.join().unwrap())
        .collect::<Vec<_>>();

    assert_eq!(responses.iter().filter(|rsp| rsp.success).count(), 1);
    let duplicate = responses.iter().find(|rsp| !rsp.success).unwrap();
    assert_eq!(
        duplicate.result_code,
        ResultCode::AlreadyExists.as_u16() as i32
    );
    assert_eq!(duplicate.error_message, "该设备已在白名单中");
    assert_eq!(
        ctx.whitelist_manager
            .as_ref()
            .unwrap()
            .query_all()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn add_rejects_empty_serial_before_device_lookup() {
    for add_method in ["device", "management"] {
        let fixture = context(None);
        let ctx = fixture.context;
        let mut cmd = command(add_method);
        cmd.serial_number = "   ".into();

        let rsp = decode_common(&handle_add_whitelist(&ctx, &cmd.encode_to_vec()));

        assert!(!rsp.success);
        assert_eq!(
            rsp.result_code,
            ResultCode::SerialNumberEmpty.as_u16() as i32
        );
        assert_eq!(rsp.error_message, "序列号不能为空");
    }
}

#[test]
fn device_add_rejects_non_storage_with_standard_error() {
    let mut dev = device(DeviceType::Keyboard, 0x03);
    dev.interface_subclass = 0x01; // SUBCLASS_BOOT，避免触发 disguise 检测
    dev.interface_protocol = 0x01; // PROTOCOL_KEYBOARD
    let fixture = context(Some(dev));
    let ctx = fixture.context;

    let rsp = decode_common(&handle_add_whitelist(
        &ctx,
        &command("device").encode_to_vec(),
    ));

    assert_eq!(
        rsp.result_code,
        ResultCode::DeviceNotStorage.as_u16() as i32
    );
    assert_eq!(rsp.error_message, "仅支持添加大容量存储设备");
}

#[test]
fn device_add_rejects_spoof_with_standard_error() {
    let fixture = context(Some(device(DeviceType::Storage, 0x03)));
    let ctx = fixture.context;

    let rsp = decode_common(&handle_add_whitelist(
        &ctx,
        &command("device").encode_to_vec(),
    ));

    assert_eq!(
        rsp.result_code,
        ResultCode::DeviceSpoofSuspected.as_u16() as i32
    );
    assert_eq!(rsp.error_message, "设备描述符异常，疑似伪装设备，禁止添加");
}

#[test]
fn device_add_rejects_unknown_and_unsupported_with_standard_error() {
    for device_type in [DeviceType::Unknown, DeviceType::Unsupported] {
        let fixture = context(Some(device(device_type, 0xff)));
        let ctx = fixture.context;

        let rsp = decode_common(&handle_add_whitelist(
            &ctx,
            &command("device").encode_to_vec(),
        ));

        assert_eq!(
            rsp.result_code,
            ResultCode::DeviceUnsupported.as_u16() as i32
        );
        assert_eq!(rsp.error_message, "不支持的USB设备类型，无法添加");
    }
}

#[test]
fn device_add_holds_presence_guard_through_database_commit() {
    let fixture = context(Some(device(DeviceType::Storage, 0x08)));
    let ctx = fixture.context;
    let ctx = Arc::new(ctx);
    let whitelist = Arc::clone(ctx.whitelist_manager.as_ref().unwrap());
    let manager = Arc::clone(ctx.device_manager.as_ref().unwrap());

    let (import_entered_tx, import_entered_rx) = mpsc::channel();
    let (release_import_tx, release_import_rx) = mpsc::channel();
    let import_thread = thread::spawn(move || {
        whitelist
            .coordinate_policy_import(|| {
                import_entered_tx.send(()).unwrap();
                release_import_rx.recv().unwrap();
                Ok(Vec::new())
            })
            .unwrap();
    });
    import_entered_rx.recv().unwrap();

    let handler_ctx = Arc::clone(&ctx);
    let (handler_done_tx, handler_done_rx) = mpsc::channel();
    let handler_thread = thread::spawn(move || {
        handler_done_tx
            .send(handle_add_whitelist(
                &handler_ctx,
                &command("device").encode_to_vec(),
            ))
            .unwrap();
    });

    let deadline = Instant::now() + Duration::from_secs(1);
    loop {
        match manager.try_write() {
            Err(TryLockError::WouldBlock) => break,
            Err(TryLockError::Poisoned(_)) => panic!("device manager lock poisoned"),
            Ok(guard) => drop(guard),
        }
        assert!(
            Instant::now() < deadline,
            "handler did not acquire presence guard"
        );
        thread::yield_now();
    }

    let remove_manager = Arc::clone(&manager);
    let (remove_acquired_tx, remove_acquired_rx) = mpsc::channel();
    let (allow_remove_tx, allow_remove_rx) = mpsc::channel();
    let (remove_done_tx, remove_done_rx) = mpsc::channel();
    let remove_thread = thread::spawn(move || {
        let mut guard = remove_manager.write().unwrap();
        remove_acquired_tx
            .send(guard.connected_device_by_serial("REAL-SN").is_some())
            .unwrap();
        allow_remove_rx.recv().unwrap();
        guard.remove_interface("/sys/real");
        remove_done_tx.send(()).unwrap();
    });

    assert!(matches!(
        remove_acquired_rx.recv_timeout(Duration::from_millis(50)),
        Err(mpsc::RecvTimeoutError::Timeout)
    ));
    release_import_tx.send(()).unwrap();

    let response = handler_done_rx.recv().unwrap();
    assert!(decode_common(&response).success);
    assert!(remove_acquired_rx.recv().unwrap());
    assert!(ctx
        .whitelist_manager
        .as_ref()
        .unwrap()
        .query_by_sn("REAL-SN")
        .unwrap()
        .is_some());

    allow_remove_tx.send(()).unwrap();
    remove_done_rx.recv().unwrap();
    remove_thread.join().unwrap();
    handler_thread.join().unwrap();
    import_thread.join().unwrap();

    assert!(manager
        .read()
        .unwrap()
        .connected_device_by_serial("REAL-SN")
        .is_none());
    assert!(ctx
        .whitelist_manager
        .as_ref()
        .unwrap()
        .query_by_sn("REAL-SN")
        .unwrap()
        .is_some());
}
