//! S04 文件访问控制策略协议 handler。

use prost::Message;

use common::code::ResultCode;
use common::proto::{
    CmdAddBlacklistExtension, CmdGetFilePolicy, CmdRemoveBlacklistExtension,
    CmdUpdateFilePolicySwitch, FileTypeBlacklistItem, RspCommon, RspFilePolicy,
};
use storage::model::OperationLogInsert;
use storage::StorageError;

use crate::codec;
use crate::context::RequestContext;

const RSP_FILE_POLICY: u32 = 0x0201;
const RSP_COMMON: u32 = 0xFF00;

/// 合法的 policy_key 值。
const VALID_POLICY_KEYS: &[&str] = &[
    "exec_control",
    "auto_read_control",
    "file_type_blacklist_control",
];

/// CMD_GET_FILE_POLICY (0x0200)。
pub fn handle_get_file_policy(ctx: &RequestContext, payload: &[u8]) -> Vec<u8> {
    let _cmd = match CmdGetFilePolicy::decode(payload) {
        Ok(c) => c,
        Err(_) => return error_response(ctx.seq_id, ResultCode::ValidationFailed, "消息解码失败"),
    };

    let storage = match ctx.storage() {
        Some(s) => s,
        None => return error_response(ctx.seq_id, ResultCode::InternalError, "存储服务未初始化"),
    };

    let exec_control = storage
        .policy_query("exec_control")
        .ok()
        .flatten()
        .map(|p| p.enabled == 1)
        .unwrap_or(false);

    let auto_read_control = storage
        .policy_query("auto_read_control")
        .ok()
        .flatten()
        .map(|p| p.enabled == 1)
        .unwrap_or(false);

    let file_type_blacklist = storage
        .policy_query("file_type_blacklist_control")
        .ok()
        .flatten()
        .map(|p| p.enabled == 1)
        .unwrap_or(false);

    let blacklist_items = storage.blacklist_query_all().unwrap_or_default();
    let blacklist: Vec<FileTypeBlacklistItem> = blacklist_items
        .iter()
        .map(|item| FileTypeBlacklistItem {
            extension: item.extension.clone(),
            description: item.description.clone().unwrap_or_default(),
            is_default: item.is_default == 1,
            created_at: item.created_at,
        })
        .collect();

    let rsp = RspFilePolicy {
        exec_control_enabled: exec_control,
        auto_read_control_enabled: auto_read_control,
        file_type_blacklist_enabled: file_type_blacklist,
        blacklist,
    };

    codec::encode_frame(RSP_FILE_POLICY, ctx.seq_id, &rsp.encode_to_vec()).unwrap_or_default()
}

/// CMD_UPDATE_FILE_POLICY_SWITCH (0x0202)。
pub fn handle_update_file_policy_switch(ctx: &RequestContext, payload: &[u8]) -> Vec<u8> {
    let cmd = match CmdUpdateFilePolicySwitch::decode(payload) {
        Ok(c) => c,
        Err(_) => return error_response(ctx.seq_id, ResultCode::ValidationFailed, "消息解码失败"),
    };

    if !VALID_POLICY_KEYS.contains(&cmd.policy_key.as_str()) {
        return error_response(
            ctx.seq_id,
            ResultCode::PolicyKeyInvalid,
            &format!("无效的策略标识: {}", cmd.policy_key),
        );
    }

    let storage = match ctx.storage() {
        Some(s) => s,
        None => return error_response(ctx.seq_id, ResultCode::InternalError, "存储服务未初始化"),
    };

    match storage.policy_update(&cmd.policy_key, cmd.enabled) {
        Ok(()) => {
            write_audit_log(ctx, "file_policy", "update", Some(&cmd.policy_key), 0, None);
            success_response(ctx.seq_id)
        }
        Err(e) => error_response(ctx.seq_id, ResultCode::InternalError, &e.to_string()),
    }
}

/// CMD_ADD_BLACKLIST_EXTENSION (0x0203)。
pub fn handle_add_blacklist_extension(ctx: &RequestContext, payload: &[u8]) -> Vec<u8> {
    let cmd = match CmdAddBlacklistExtension::decode(payload) {
        Ok(c) => c,
        Err(_) => return error_response(ctx.seq_id, ResultCode::ValidationFailed, "消息解码失败"),
    };

    let extension = match normalize_handler_extension(&cmd.extension) {
        Ok(extension) => extension,
        Err(code) => return error_response(ctx.seq_id, code, "文件后缀格式错误"),
    };

    let storage = match ctx.storage() {
        Some(s) => s,
        None => return error_response(ctx.seq_id, ResultCode::InternalError, "存储服务未初始化"),
    };

    match storage.blacklist_query_by_ext(&extension) {
        Ok(Some(_)) => {
            return error_response(
                ctx.seq_id,
                ResultCode::ExtensionExists,
                "该文件后缀已在黑名单中",
            );
        }
        Ok(None) => {}
        Err(e) => return error_response(ctx.seq_id, ResultCode::InternalError, &e.to_string()),
    }

    let description = if cmd.description.is_empty() {
        None
    } else {
        Some(cmd.description.as_str())
    };

    match storage.blacklist_insert(&extension, description) {
        Ok(_id) => {
            write_audit_log(ctx, "file_blacklist", "add", Some(&extension), 0, None);
            success_response(ctx.seq_id)
        }
        Err(e) => error_response(
            ctx.seq_id,
            map_blacklist_insert_error(&e),
            &e.to_string(),
        ),
    }
}

/// CMD_REMOVE_BLACKLIST_EXTENSION (0x0204)。
pub fn handle_remove_blacklist_extension(ctx: &RequestContext, payload: &[u8]) -> Vec<u8> {
    let cmd = match CmdRemoveBlacklistExtension::decode(payload) {
        Ok(c) => c,
        Err(_) => return error_response(ctx.seq_id, ResultCode::ValidationFailed, "消息解码失败"),
    };

    let extension = match normalize_handler_extension(&cmd.extension) {
        Ok(extension) => extension,
        Err(code) => return error_response(ctx.seq_id, code, "文件后缀格式错误"),
    };

    let storage = match ctx.storage() {
        Some(s) => s,
        None => return error_response(ctx.seq_id, ResultCode::InternalError, "存储服务未初始化"),
    };

    let item = match storage.blacklist_query_by_ext(&extension) {
        Ok(Some(item)) => item,
        Ok(None) => {
            return error_response(
                ctx.seq_id,
                ResultCode::ExtensionNotFound,
                "该文件后缀不在黑名单中",
            );
        }
        Err(e) => return error_response(ctx.seq_id, ResultCode::InternalError, &e.to_string()),
    };

    if item.is_default == 1 {
        return error_response(
            ctx.seq_id,
            ResultCode::DefaultExtensionNoDelete,
            "内置默认后缀不可删除",
        );
    }

    match storage.blacklist_delete(&extension) {
        Ok(()) => {
            write_audit_log(ctx, "file_blacklist", "remove", Some(&extension), 0, None);
            success_response(ctx.seq_id)
        }
        Err(e) => error_response(ctx.seq_id, ResultCode::InternalError, &e.to_string()),
    }
}

fn normalize_handler_extension(extension: &str) -> Result<String, ResultCode> {
    storage::normalize_extension(extension).map_err(|_| ResultCode::ExtensionFormatError)
}

fn map_blacklist_insert_error(error: &StorageError) -> ResultCode {
    match error {
        StorageError::AlreadyExists => ResultCode::ExtensionExists,
        StorageError::Validation(_) => ResultCode::ExtensionFormatError,
        _ => ResultCode::InternalError,
    }
}

/// 写审计日志。
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

    #[test]
    fn handler_normalization_accepts_dotted_mixed_case_extension() {
        assert_eq!(normalize_handler_extension("  .PS1 ").unwrap(), ".ps1");
    }

    #[test]
    fn handler_normalization_maps_invalid_input_to_extension_format_error() {
        assert_eq!(
            normalize_handler_extension("ps1").unwrap_err(),
            ResultCode::ExtensionFormatError
        );
    }

    #[test]
    fn duplicate_storage_error_maps_to_extension_exists() {
        assert_eq!(
            map_blacklist_insert_error(&storage::StorageError::AlreadyExists),
            ResultCode::ExtensionExists
        );
    }
}
