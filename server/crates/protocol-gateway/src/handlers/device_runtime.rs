//! CMD_GET_DEVICE_RUNTIME_STATUS handler。

use prost::Message;
use tracing::debug;

use common::code::ResultCode;
use common::proto::{
    CmdGetDeviceRuntimeStatus, DeviceRuntimeStatus, RspCommon, RspDeviceRuntimeStatus,
};

use crate::codec;
use crate::context::RequestContext;

const RSP_DEVICE_RUNTIME_STATUS: u32 = 0x0407;
const RSP_COMMON: u32 = 0xFF00;

/// 处理受控设备运行态查询。
pub fn handle_get_device_runtime_status(ctx: &RequestContext, payload: &[u8]) -> Vec<u8> {
    debug!("收到设备运行态查询请求");

    let _cmd = match CmdGetDeviceRuntimeStatus::decode(payload) {
        Ok(cmd) => cmd,
        Err(_) => return error_response(ctx.seq_id, ResultCode::ValidationFailed, "消息解码失败"),
    };

    let registry = match ctx.device_runtime_registry.as_ref() {
        Some(registry) => registry,
        None => return error_response(ctx.seq_id, ResultCode::InternalError, "设备运行态未初始化"),
    };

    let devices = registry
        .list()
        .into_iter()
        .map(|item| DeviceRuntimeStatus {
            runtime_id: item.runtime_id,
            parent_path: item.parent_path,
            interface_path: item.interface_path,
            serial_number: item.serial_number,
            device_name: item.device_name,
            device_type: item.device_type,
            interface_type: item.interface_type,
            status: item.status,
            stage: item.stage,
            fail_code: item.fail_code,
            fail_reason: item.fail_reason,
            connected_at: item.connected_at,
            updated_at: item.updated_at,
        })
        .collect::<Vec<_>>();

    debug!(count = devices.len(), "设备运行态查询成功");

    let rsp = RspDeviceRuntimeStatus { devices };
    codec::encode_frame(
        RSP_DEVICE_RUNTIME_STATUS,
        ctx.seq_id,
        &rsp.encode_to_vec(),
    )
    .unwrap_or_default()
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
    use std::sync::Arc;

    use auth_session::{AuthService, SessionManager};
    use device_runtime::{DeviceRuntimeCreate, DeviceRuntimeRegistry};
    use log_audit::AuditService;
    use storage::Storage;
    use storage_test_support::initialize_database;
    use tempfile::{NamedTempFile, TempPath};
    use whitelist::WhitelistManager;

    fn context_with_registry(registry: Arc<DeviceRuntimeRegistry>) -> (RequestContext, TempPath) {
        let db = NamedTempFile::new().unwrap().into_temp_path();
        initialize_database(&db);
        let storage = Arc::new(Storage::open(&db).unwrap());
        let auth = Arc::new(AuthService::new(
            Arc::clone(&storage),
            SessionManager::new(),
        ));
        let audit = Arc::new(AuditService::new(Arc::clone(&storage), &db));
        let whitelist = Arc::new(WhitelistManager::new(Arc::clone(&storage)).unwrap());

        (
            RequestContext {
                seq_id: 7,
                session: None,
                source_ip: "127.0.0.1".to_string(),
                auth_service: auth,
                audit_service: audit,
                whitelist_manager: Some(whitelist),
                device_manager: None,
                device_runtime_registry: Some(registry),
                storage: Some(storage),
                policy_service: None,
                license_validator: None,
                system_upgrade_mgr: None,
                virusdb_upgrade_mgr: None,
            },
            db,
        )
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

        let (ctx, _path) = context_with_registry(registry);
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
        let (mut ctx, _path) = context_with_registry(Arc::new(DeviceRuntimeRegistry::new()));
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
}
