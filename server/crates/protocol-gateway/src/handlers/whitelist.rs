//! S05 白名单协议 handler。

use prost::Message;
use tracing::{debug, info, warn};

use common::code::ResultCode;
use common::mapping::{
    add_method_int_to_str, add_method_str_to_int, permission_int_to_str, permission_str_to_int,
};
use common::proto::{
    CmdAddWhitelist, CmdListWhitelist, CmdRemoveWhitelist, CmdUpdateWhitelist, RspCommon,
    RspListWhitelist, WhitelistDevice,
};
use common::types::DeviceType;
use storage::model::OperationLogInsert;
use usb_identify::descriptor::{admission_status_str, detect_spoof, interface_type_str};
use whitelist::service::AddWhitelistRequest;
use whitelist::WhitelistError;

use crate::codec;
use crate::context::RequestContext;

const RSP_LIST_WHITELIST: u32 = 0x0101;
const RSP_COMMON: u32 = 0xFF00;

/// CMD_LIST_WHITELIST (0x0100)。
pub fn handle_list_whitelist(ctx: &RequestContext, payload: &[u8]) -> Vec<u8> {
    debug!("收到白名单列表查询请求");
    let _cmd = match CmdListWhitelist::decode(payload) {
        Ok(c) => c,
        Err(_) => return error_response(ctx.seq_id, ResultCode::ValidationFailed, "消息解码失败"),
    };

    let mgr = match ctx.whitelist_manager.as_ref() {
        Some(m) => m,
        None => return error_response(ctx.seq_id, ResultCode::InternalError, "白名单服务未初始化"),
    };

    let list = match mgr.query_all() {
        Ok(l) => {
            debug!(count = l.len(), "白名单列表查询成功");
            l
        }
        Err(e) => {
            warn!(reason = %e, "白名单列表查询失败");
            return error_response(ctx.seq_id, ResultCode::InternalError, &e.to_string());
        }
    };

    let devices: Vec<WhitelistDevice> = list
        .iter()
        .map(|item| WhitelistDevice {
            serial_number: item.serial_number.clone(),
            vid: item.vid.clone().unwrap_or_default(),
            pid: item.pid.clone().unwrap_or_default(),
            device_name: item.device_name.clone().unwrap_or_default(),
            capacity_bytes: item.capacity_bytes.unwrap_or(0),
            permission: permission_int_to_str(item.permission)
                .unwrap_or("readonly")
                .to_string(),
            description: item.description.clone().unwrap_or_default(),
            add_method: add_method_int_to_str(item.add_method)
                .unwrap_or("device")
                .to_string(),
            created_at: item.created_at,
            device_type: item.device_type.clone(),
        })
        .collect();

    let rsp = RspListWhitelist { devices };
    codec::encode_frame(RSP_LIST_WHITELIST, ctx.seq_id, &rsp.encode_to_vec()).unwrap_or_default()
}

/// CMD_ADD_WHITELIST (0x0104)。
pub fn handle_add_whitelist(ctx: &RequestContext, payload: &[u8]) -> Vec<u8> {
    let cmd = match CmdAddWhitelist::decode(payload) {
        Ok(c) => c,
        Err(_) => return error_response(ctx.seq_id, ResultCode::ValidationFailed, "消息解码失败"),
    };

    if cmd.serial_number.trim().is_empty() {
        return error_response(ctx.seq_id, ResultCode::SerialNumberEmpty, "序列号不能为空");
    }

    let mgr = match ctx.whitelist_manager.as_ref() {
        Some(m) => m,
        None => return error_response(ctx.seq_id, ResultCode::InternalError, "白名单服务未初始化"),
    };

    let permission = match permission_str_to_int(&cmd.permission) {
        Ok(p) => p,
        Err(_) => {
            return error_response(
                ctx.seq_id,
                ResultCode::ValidationFailed,
                &format!("无效的权限值: {}", cmd.permission),
            )
        }
    };

    let add_method = match add_method_str_to_int(&cmd.add_method) {
        Ok(m) => m,
        Err(_) => {
            return error_response(
                ctx.seq_id,
                ResultCode::ValidationFailed,
                &format!("无效的添加方式: {}", cmd.add_method),
            )
        }
    };

    let serial_number = cmd.serial_number.clone();
    debug!(sn = %serial_number, method = %cmd.add_method, "收到白名单添加请求");
    let description = (!cmd.description.is_empty()).then(|| cmd.description.clone());
    let add_result = if add_method == 0 {
        let dm = match ctx.device_manager.as_ref() {
            Some(manager) => manager,
            None => {
                return error_response(ctx.seq_id, ResultCode::InternalError, "设备管理器未初始化")
            }
        };
        let dm_guard = match dm.read() {
            Ok(guard) => guard,
            Err(_) => {
                return error_response(
                    ctx.seq_id,
                    ResultCode::InternalError,
                    "设备管理器锁获取失败",
                )
            }
        };
        let info = match dm_guard.connected_device_by_serial(&cmd.serial_number) {
            Some(info) => info,
            None => {
                return error_response(
                    ctx.seq_id,
                    ResultCode::ValidationFailed,
                    "设备已移除，请重新插入后再添加",
                )
            }
        };
        let is_in_whitelist = mgr.is_whitelisted(&info.serial_number).is_some();
        let is_spoof = detect_spoof(info);
        if is_spoof {
            return error_response(
                ctx.seq_id,
                ResultCode::DeviceSpoofSuspected,
                "设备描述符异常，疑似伪装设备，禁止添加",
            );
        }
        if dm_guard.is_badusb_by_serial(&cmd.serial_number) {
            return error_response(
                ctx.seq_id,
                ResultCode::DeviceSpoofSuspected,
                "疑似 BadUSB 伪装设备（同时具备存储和键盘接口），禁止添加",
            );
        }
        if matches!(
            info.device_type,
            DeviceType::Unknown | DeviceType::Unsupported
        ) {
            return error_response(
                ctx.seq_id,
                ResultCode::DeviceUnsupported,
                "不支持的USB设备类型，无法添加",
            );
        }
        if info.device_type != DeviceType::Storage || interface_type_str(info) != "mass_storage" {
            return error_response(
                ctx.seq_id,
                ResultCode::DeviceNotStorage,
                "仅支持添加大容量存储设备",
            );
        }
        if admission_status_str(info, is_in_whitelist, is_spoof) != "addable" {
            return error_response(ctx.seq_id, ResultCode::AlreadyExists, "该设备已在白名单中");
        }

        let req = AddWhitelistRequest {
            serial_number: cmd.serial_number.clone(),
            vid: (!info.vid.is_empty()).then(|| info.vid.clone()),
            pid: (!info.pid.is_empty()).then(|| info.pid.clone()),
            device_name: (!info.device_name.is_empty()).then(|| info.device_name.clone()),
            capacity_bytes: info.capacity_bytes,
            device_type: "storage".to_string(),
            description: description.clone(),
            permission,
            add_method,
        };
        mgr.add(req)
    } else {
        if cmd.device_type != "storage" {
            return error_response(
                ctx.seq_id,
                ResultCode::DeviceNotStorage,
                "仅支持添加大容量存储设备",
            );
        }
        let req = AddWhitelistRequest {
            serial_number: cmd.serial_number.clone(),
            vid: (!cmd.vid.is_empty()).then(|| cmd.vid.clone()),
            pid: (!cmd.pid.is_empty()).then(|| cmd.pid.clone()),
            device_name: (!cmd.device_name.is_empty()).then(|| cmd.device_name.clone()),
            capacity_bytes: (cmd.capacity_bytes != 0).then_some(cmd.capacity_bytes),
            device_type: cmd.device_type.clone(),
            description,
            permission,
            add_method,
        };
        mgr.add(req)
    };

    match add_result {
        Ok(_id) => {
            info!(sn = %serial_number, method = cmd.add_method, "白名单添加成功");
            write_audit_log(ctx, "whitelist_add", "add", Some(&serial_number), 0, None);
            success_response(ctx.seq_id)
        }
        Err(WhitelistError::AlreadyExists(_)) => {
            info!(sn = %serial_number, "白名单添加失败：设备已存在");
            write_audit_log(ctx, "whitelist_add", "add", Some(&serial_number), 1, Some("该设备已在白名单中"));
            error_response(ctx.seq_id, ResultCode::AlreadyExists, "该设备已在白名单中")
        }
        Err(e) => {
            warn!(sn = %serial_number, reason = %e, "白名单添加失败");
            write_audit_log(ctx, "whitelist_add", "add", Some(&serial_number), 1, Some(&e.to_string()));
            error_response(ctx.seq_id, e.to_result_code(), &e.to_string())
        }
    }
}

/// CMD_REMOVE_WHITELIST (0x0105)。
pub fn handle_remove_whitelist(ctx: &RequestContext, payload: &[u8]) -> Vec<u8> {
    let cmd = match CmdRemoveWhitelist::decode(payload) {
        Ok(c) => c,
        Err(_) => return error_response(ctx.seq_id, ResultCode::ValidationFailed, "消息解码失败"),
    };

    debug!(sn = %cmd.serial_number, "收到白名单删除请求");

    let mgr = match ctx.whitelist_manager.as_ref() {
        Some(m) => m,
        None => return error_response(ctx.seq_id, ResultCode::InternalError, "白名单服务未初始化"),
    };

    match mgr.remove(&cmd.serial_number) {
        Ok(()) => {
            info!(sn = %cmd.serial_number, "白名单删除成功");
            write_audit_log(
                ctx,
                "whitelist_remove",
                "remove",
                Some(&cmd.serial_number),
                0,
                None,
            );
            success_response(ctx.seq_id)
        }
        Err(e) => {
            warn!(sn = %cmd.serial_number, reason = %e, "白名单删除失败");
            write_audit_log(ctx, "whitelist_remove", "remove", Some(&cmd.serial_number), 1, Some(&e.to_string()));
            error_response(ctx.seq_id, e.to_result_code(), &e.to_string())
        }
    }
}

/// CMD_UPDATE_WHITELIST (0x0106)。
pub fn handle_update_whitelist(ctx: &RequestContext, payload: &[u8]) -> Vec<u8> {
    let cmd = match CmdUpdateWhitelist::decode(payload) {
        Ok(c) => c,
        Err(_) => return error_response(ctx.seq_id, ResultCode::ValidationFailed, "消息解码失败"),
    };

    debug!(sn = %cmd.serial_number, "收到白名单更新请求");

    let mgr = match ctx.whitelist_manager.as_ref() {
        Some(m) => m,
        None => return error_response(ctx.seq_id, ResultCode::InternalError, "白名单服务未初始化"),
    };

    let permission = if cmd.permission.is_empty() {
        None
    } else {
        match permission_str_to_int(&cmd.permission) {
            Ok(p) => Some(p),
            Err(_) => {
                return error_response(
                    ctx.seq_id,
                    ResultCode::ValidationFailed,
                    &format!("无效的权限值: {}", cmd.permission),
                )
            }
        }
    };

    let description = if cmd.description.is_empty() {
        None
    } else {
        Some(cmd.description.as_str())
    };

    match mgr.update(&cmd.serial_number, permission, description) {
        Ok(()) => {
            info!(sn = %cmd.serial_number, "白名单更新成功");
            write_audit_log(
                ctx,
                "whitelist_update",
                "update",
                Some(&cmd.serial_number),
                0,
                None,
            );
            success_response(ctx.seq_id)
        }
        Err(e) => {
            warn!(sn = %cmd.serial_number, reason = %e, "白名单更新失败");
            write_audit_log(ctx, "whitelist_update", "update", Some(&cmd.serial_number), 1, Some(&e.to_string()));
            error_response(ctx.seq_id, e.to_result_code(), &e.to_string())
        }
    }
}

/// 写审计日志辅助函数。
fn write_audit_log(
    ctx: &RequestContext,
    log_type: &str,
    action_type: &str,
    target: Option<&str>,
    result: i32,
    fail_reason: Option<&str>,
) {
    let mut log = OperationLogInsert {
        op_time: 0,
        username: ctx
            .session
            .as_ref()
            .map(|s| s.username.clone())
            .unwrap_or_default(),
        role: ctx.session.as_ref().map(|s| s.role).unwrap_or(-1),
        log_type: log_type.into(),
        action_type: Some(action_type.into()),
        target: target.map(|t| t.to_string()),
        before_value: None,
        after_value: None,
        related_file: None,
        related_version: None,
        result,
        fail_reason: fail_reason.map(|r| r.to_string()),
        source_ip: Some(ctx.source_ip.clone()),
        app_version: None,
        session_id: None,
        request_id: None,
        detail: None,
    };
    let _ = ctx.audit_service.log_operation(&mut log);
}

fn success_response(seq_id: u32) -> Vec<u8> {
    let rsp = RspCommon {
        success: true,
        result_code: ResultCode::Success.as_u16() as i32,
        error_message: String::new(),
    };
    codec::encode_frame(RSP_COMMON, seq_id, &rsp.encode_to_vec()).unwrap_or_default()
}

fn error_response(seq_id: u32, code: ResultCode, msg: &str) -> Vec<u8> {
    let rsp = RspCommon {
        success: false,
        result_code: code.as_u16() as i32,
        error_message: msg.to_string(),
    };
    codec::encode_frame(RSP_COMMON, seq_id, &rsp.encode_to_vec()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::sync::{Arc, Barrier, RwLock, TryLockError};
    use std::thread;
    use std::time::{Duration, Instant};

    use auth_session::{AuthService, SessionManager};
    use common::types::DeviceType;
    use log_audit::AuditService;
    use storage::Storage;
    use tempfile::{NamedTempFile, TempPath};
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

    fn context(device: Option<UsbDeviceInfo>) -> (RequestContext, TempPath) {
        let path = NamedTempFile::new().unwrap().into_temp_path();
        let storage = Arc::new(Storage::open(&path).unwrap());
        let auth = Arc::new(AuthService::new(
            Arc::clone(&storage),
            SessionManager::new(),
        ));
        let audit = Arc::new(AuditService::new(Arc::clone(&storage), &path));
        let whitelist = Arc::new(WhitelistManager::new(Arc::clone(&storage)).unwrap());
        let mut manager = DeviceManager::new();
        if let Some(device) = device {
            manager.add(device);
        }
        (
            RequestContext {
                seq_id: 9,
                session: None,
                source_ip: "127.0.0.1".into(),
                auth_service: auth,
                audit_service: audit,
                whitelist_manager: Some(whitelist),
                device_manager: Some(Arc::new(RwLock::new(manager))),
                storage: None,
                policy_service: None,
                license_validator: None,
                system_upgrade_mgr: None,
                virusdb_upgrade_mgr: None,
            },
            path,
        )
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
        let (ctx, _path) = context(Some(device(DeviceType::Storage, 0x08)));
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
        let (ctx, _path) = context(Some(device(DeviceType::Storage, 0x08)));
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
        let (ctx, _path) = context(None);
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
        let (ctx, _path) = context(None);
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
        let (ctx, _path) = context(None);
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
        let (ctx, _path) = context(Some(device(DeviceType::Storage, 0x08)));
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
            let (ctx, _path) = context(None);
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
        let (ctx, _path) = context(Some(dev));

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
        let (ctx, _path) = context(Some(device(DeviceType::Storage, 0x03)));

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
            let (ctx, _path) = context(Some(device(device_type, 0xff)));

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
        let (ctx, _path) = context(Some(device(DeviceType::Storage, 0x08)));
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
}
