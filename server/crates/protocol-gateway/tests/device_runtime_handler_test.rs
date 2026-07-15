mod support;

use common::code::ResultCode;
use common::proto::{CmdGetDeviceRuntimeStatus, RspCommon, RspDeviceRuntimeStatus};
use prost::Message;
use protocol_gateway::codec;
use protocol_gateway::handlers::device_runtime::handle_get_device_runtime_status;
use std::sync::Arc;
use support::{request_fixture, RequestFixture};

use device_runtime::{DeviceRuntimeCreate, DeviceRuntimeRegistry};

fn context_with_registry(registry: Arc<DeviceRuntimeRegistry>) -> RequestFixture {
    let mut fixture = request_fixture(7);
    fixture.context.device_runtime_registry = Some(registry);
    fixture
}

#[test]
fn returns_runtime_status_snapshots() {
    let registry = Arc::new(DeviceRuntimeRegistry::new());
    registry.create(DeviceRuntimeCreate {
        runtime_id: "runtime-1".to_string(),
        parent_path: "/sys/usb/1-1".to_string(),
        interface_path: "/sys/usb/1-1:1.0".to_string(),
        serial_number: "SN001".to_string(),
        device_name: "Cruzer Blade".to_string(),
        device_type: "storage".to_string(),
        interface_type: "mass_storage".to_string(),
        status: "processing".to_string(),
        stage: "scan".to_string(),
        fail_code: String::new(),
        fail_reason: String::new(),
    });

    let fixture = context_with_registry(registry);
    let ctx = fixture.context;
    let payload = CmdGetDeviceRuntimeStatus {
        session_token: "token".to_string(),
    }
    .encode_to_vec();

    let frame = handle_get_device_runtime_status(&ctx, &payload);
    let (_, payload, _) = codec::try_decode_frame(&frame).unwrap().unwrap();
    let rsp = RspDeviceRuntimeStatus::decode(payload.as_slice()).unwrap();

    assert_eq!(rsp.devices.len(), 1);
    assert_eq!(rsp.devices[0].runtime_id, "runtime-1");
    assert_eq!(rsp.devices[0].device_type, "storage");
    assert_eq!(rsp.devices[0].status, "processing");
    assert_eq!(rsp.devices[0].stage, "scan");
}

#[test]
fn missing_registry_returns_common_error() {
    let fixture = context_with_registry(Arc::new(DeviceRuntimeRegistry::new()));
    let mut ctx = fixture.context;
    ctx.device_runtime_registry = None;
    let payload = CmdGetDeviceRuntimeStatus {
        session_token: "token".to_string(),
    }
    .encode_to_vec();

    let frame = handle_get_device_runtime_status(&ctx, &payload);
    let (_, payload, _) = codec::try_decode_frame(&frame).unwrap().unwrap();
    let rsp = RspCommon::decode(payload.as_slice()).unwrap();

    assert!(!rsp.success);
    assert_eq!(rsp.result_code, ResultCode::InternalError.as_u16() as i32);
    assert_eq!(rsp.error_message, "设备运行态未初始化");
}
