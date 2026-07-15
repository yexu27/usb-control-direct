//! 系统管理 handler（0x0500/0x0502/0x0503/0x0504）。

use prost::Message;

use sha2::{Digest, Sha256};
use tracing::{debug, error, info, warn};

use common::audit_const::{action_type, log_type};
use common::code::ResultCode;
use common::proto::{
    CmdGetSystemInfo, CmdUpdateDeviceDesc, CmdUploadSystemUpgrade, CmdUploadVirusdbUpgrade,
    RspCommon, RspSystemInfo,
};

use super::audit_helper::{log_operation, log_operation_full, OperationDetail, OperationOutcome};
use super::license_state::read_license_snapshot;
use crate::codec;
use crate::context::RequestContext;
use crate::post_send::{HandlerOutcome, PostSendAction};
use crate::upgrade_error::map_upgrade_error;
use system_upgrade::{ActiveReleaseStore, PrepareUpgradeRequest};

/// RspSystemInfo 消息类型。
const RSP_SYSTEM_INFO: u32 = 0x0501;

/// RspCommon 消息类型。
const RSP_COMMON: u32 = 0xFF00;

/// CMD_GET_SYSTEM_INFO (0x0500) handler。
pub fn handle_get_system_info(ctx: &RequestContext, payload: &[u8]) -> Vec<u8> {
    debug!("收到系统信息查询请求");

    let _cmd = match CmdGetSystemInfo::decode(payload) {
        Ok(c) => c,
        Err(_) => {
            return sysinfo_error(ctx.seq_id, ResultCode::ValidationFailed, "消息解码失败");
        }
    };

    let storage = match ctx.storage() {
        Some(s) => s,
        None => {
            return sysinfo_error(ctx.seq_id, ResultCode::InternalError, "存储服务未初始化");
        }
    };

    let system_version = match ActiveReleaseStore::new(ctx.system_upgrade_root.clone())
        .and_then(|store| store.current())
    {
        Ok(Some(active)) => active.version.to_string(),
        Ok(None) | Err(_) => {
            return sysinfo_error(
                ctx.seq_id,
                ResultCode::InternalError,
                "有效系统版本读取失败",
            );
        }
    };
    let virus_db_version = config_value(storage, "virus_db_version");
    let license = match read_license_snapshot(storage, common::time::now_unix()) {
        Ok(license) => license,
        Err(_) => {
            return sysinfo_error(ctx.seq_id, ResultCode::InternalError, "授权状态读取失败");
        }
    };
    let virus_db_updated_at = config_value(storage, "virus_db_updated_at")
        .parse::<i64>()
        .unwrap_or(0);

    let rsp = RspSystemInfo {
        system_version,
        virus_db_version,
        authorized: license.authorized,
        auth_expire_time: license.expire_time,
        device_description: license.device_description,
        virus_db_updated_at,
        auth_status: license.status,
    };
    codec::encode_frame(RSP_SYSTEM_INFO, ctx.seq_id, &rsp.encode_to_vec()).unwrap_or_default()
}

/// CMD_UPLOAD_SYSTEM_UPGRADE (0x0502) handler。
pub fn handle_upload_system_upgrade(ctx: &RequestContext, payload: &[u8]) -> HandlerOutcome {
    let cmd = match CmdUploadSystemUpgrade::decode(payload) {
        Ok(c) => c,
        Err(_) => {
            return HandlerOutcome::response(error_response(
                ctx.seq_id,
                ResultCode::ValidationFailed,
                "消息解码失败",
            ));
        }
    };

    let session = match ctx.session_required() {
        Ok(s) => s,
        Err(code) => {
            return HandlerOutcome::response(error_response(ctx.seq_id, code, "会话状态异常"));
        }
    };

    debug!(user = %session.username, target_version = %cmd.target_version, "收到系统升级请求");

    let storage = match ctx.storage() {
        Some(storage) => storage,
        None => {
            return HandlerOutcome::response(error_response(
                ctx.seq_id,
                ResultCode::InternalError,
                "存储服务未初始化",
            ));
        }
    };
    let license = match read_license_snapshot(storage, common::time::now_unix()) {
        Ok(license) => license,
        Err(_) => {
            audit_system_upgrade_stage(
                ctx,
                session,
                &cmd.target_version,
                "authorization",
                "",
                ResultCode::InternalError,
                None,
                Some("授权状态读取失败"),
            );
            return HandlerOutcome::response(error_response(
                ctx.seq_id,
                ResultCode::InternalError,
                "授权状态读取失败",
            ));
        }
    };
    if !license.authorized {
        audit_system_upgrade_stage(
            ctx,
            session,
            &cmd.target_version,
            "authorization",
            "",
            ResultCode::Unauthorized,
            None,
            Some("装置未授权或授权已过期"),
        );
        return HandlerOutcome::response(error_response(
            ctx.seq_id,
            ResultCode::Unauthorized,
            "装置未授权或授权已过期",
        ));
    }

    let coordinator = &ctx.system_upgrade_coordinator;

    let package_sha256 = hex::encode(Sha256::digest(&cmd.upgrade_data));
    let target_version = cmd.target_version.clone();
    audit_system_upgrade_stage(
        ctx,
        session,
        &target_version,
        "upload",
        &package_sha256,
        ResultCode::Success,
        None,
        None,
    );

    let request = PrepareUpgradeRequest {
        package_bytes: cmd.upgrade_data,
        client_target_version: target_version.clone(),
        client_sha256: cmd.sha256_checksum,
        username: session.username.clone(),
        role: session.role,
        source_ip: ctx.source_ip.clone(),
    };
    match coordinator.prepare(request) {
        Ok(prepared) => {
            audit_system_upgrade_stage(
                ctx,
                session,
                &prepared.target_version.to_string(),
                "validation",
                &package_sha256,
                ResultCode::Success,
                Some(&prepared.upgrade_id),
                None,
            );
            HandlerOutcome {
                response: success_response(ctx.seq_id),
                post_send_action: Some(PostSendAction::StartSystemUpgrade {
                    upgrade_id: prepared.upgrade_id,
                }),
            }
        }
        Err(error) => {
            let mapped = map_upgrade_error(&error);
            audit_system_upgrade_stage(
                ctx,
                session,
                &target_version,
                "validation",
                &package_sha256,
                mapped.result_code,
                None,
                Some(mapped.message),
            );
            HandlerOutcome::response(error_response(
                ctx.seq_id,
                mapped.result_code,
                mapped.message,
            ))
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn audit_system_upgrade_stage(
    ctx: &RequestContext,
    session: &auth_session::session::SessionInfo,
    target_version: &str,
    stage: &str,
    package_sha256: &str,
    result_code: ResultCode,
    upgrade_id: Option<&str>,
    fail_reason: Option<&str>,
) {
    let mut detail = serde_json::json!({
        "stage": stage,
        "package_sha256": package_sha256,
        "result_code": result_code.as_u16(),
    });
    if let Some(upgrade_id) = upgrade_id {
        detail["upgrade_id"] = serde_json::Value::String(upgrade_id.to_string());
    }
    log_operation_full(
        ctx,
        session,
        log_type::PROGRAM_UPGRADE,
        action_type::SYSTEM_UPGRADE,
        target_version,
        OperationOutcome {
            result: i32::from(result_code != ResultCode::Success),
            fail_reason,
        },
        &OperationDetail {
            related_version: Some(target_version.to_string()),
            detail: Some(detail.to_string()),
            ..Default::default()
        },
    );
}

/// CMD_UPLOAD_VIRUSDB_UPGRADE (0x0503) handler。
pub fn handle_upload_virusdb_upgrade(ctx: &RequestContext, payload: &[u8]) -> Vec<u8> {
    let cmd = match CmdUploadVirusdbUpgrade::decode(payload) {
        Ok(c) => c,
        Err(_) => {
            return error_response(ctx.seq_id, ResultCode::ValidationFailed, "消息解码失败");
        }
    };

    let session = match ctx.session_required() {
        Ok(s) => s,
        Err(code) => return error_response(ctx.seq_id, code, "会话状态异常"),
    };

    debug!(user = %session.username, target_version = %cmd.target_version, "收到病毒库升级请求");

    let mgr = match ctx.virusdb_upgrade_mgr.as_ref() {
        Some(m) => m,
        None => {
            return error_response(
                ctx.seq_id,
                ResultCode::InternalError,
                "病毒库升级管理器未初始化",
            );
        }
    };

    let storage = match ctx.storage() {
        Some(s) => s,
        None => {
            return error_response(ctx.seq_id, ResultCode::InternalError, "存储服务未初始化");
        }
    };

    let current_version = config_value(storage, "virus_db_package_version");

    if !verify_sha256(&cmd.upgrade_data, &cmd.sha256_checksum) {
        log_operation(
            ctx,
            session,
            log_type::PROGRAM_UPGRADE,
            action_type::VIRUSDB_UPGRADE,
            &cmd.target_version,
            1,
            Some("SHA-256 校验失败"),
        );
        return error_response(
            ctx.seq_id,
            ResultCode::UpgradeChecksumError,
            "升级包 SHA-256 校验失败",
        );
    }

    if let Err(e) = mgr.validate_upgrade(&cmd.target_version, &current_version) {
        log_operation(
            ctx,
            session,
            log_type::PROGRAM_UPGRADE,
            action_type::VIRUSDB_UPGRADE,
            &cmd.target_version,
            1,
            Some(&e.to_string()),
        );
        return error_response(ctx.seq_id, e.to_result_code(), "病毒库版本校验失败");
    }

    if let Err(e) = mgr.apply_upgrade(&cmd.upgrade_data) {
        log_operation(
            ctx,
            session,
            log_type::PROGRAM_UPGRADE,
            action_type::VIRUSDB_UPGRADE,
            &cmd.target_version,
            1,
            Some(&e.to_string()),
        );
        return error_response(ctx.seq_id, e.to_result_code(), "病毒库升级安装失败");
    }

    let clamav_status = match mgr.read_status() {
        Ok(status) => status,
        Err(e) => {
            error!(reason = %e, "病毒库已升级但真实版本读取失败");
            log_operation(
                ctx,
                session,
                log_type::PROGRAM_UPGRADE,
                action_type::VIRUSDB_UPGRADE,
                &cmd.target_version,
                1,
                Some("病毒库真实版本读取失败"),
            );
            return error_response(
                ctx.seq_id,
                ResultCode::InternalError,
                "病毒库真实版本读取失败",
            );
        }
    };

    if let Err(_e) = storage.config_set("virus_db_version", &clamav_status.virus_db_version) {
        error!("病毒库已升级但真实版本号持久化失败");
        log_operation(
            ctx,
            session,
            log_type::PROGRAM_UPGRADE,
            action_type::VIRUSDB_UPGRADE,
            &cmd.target_version,
            1,
            Some("病毒库版本号持久化失败"),
        );
        return error_response(
            ctx.seq_id,
            ResultCode::InternalError,
            "病毒库版本号持久化失败",
        );
    }
    if let Err(_e) = storage.config_set(
        "virus_db_updated_at",
        &clamav_status.virus_db_updated_at.to_string(),
    ) {
        error!("病毒库已升级但更新时间持久化失败");
        log_operation(
            ctx,
            session,
            log_type::PROGRAM_UPGRADE,
            action_type::VIRUSDB_UPGRADE,
            &cmd.target_version,
            1,
            Some("更新时间持久化失败"),
        );
        return error_response(ctx.seq_id, ResultCode::InternalError, "更新时间持久化失败");
    }
    if let Err(_e) = storage.config_set("virus_db_package_version", &cmd.target_version) {
        error!("病毒库已升级但升级包版本持久化失败");
        log_operation(
            ctx,
            session,
            log_type::PROGRAM_UPGRADE,
            action_type::VIRUSDB_UPGRADE,
            &cmd.target_version,
            1,
            Some("病毒库升级包版本持久化失败"),
        );
        return error_response(
            ctx.seq_id,
            ResultCode::InternalError,
            "病毒库升级包版本持久化失败",
        );
    }

    info!(user = %session.username, target_version = %cmd.target_version, "病毒库升级成功");

    let ext = OperationDetail {
        related_version: Some(cmd.target_version.clone()),
        ..Default::default()
    };
    log_operation_full(
        ctx,
        session,
        log_type::PROGRAM_UPGRADE,
        action_type::VIRUSDB_UPGRADE,
        &cmd.target_version,
        OperationOutcome {
            result: 0,
            fail_reason: None,
        },
        &ext,
    );
    success_response(ctx.seq_id)
}

/// CMD_UPDATE_DEVICE_DESC (0x0504) handler。
pub fn handle_update_device_desc(ctx: &RequestContext, payload: &[u8]) -> Vec<u8> {
    let cmd = match CmdUpdateDeviceDesc::decode(payload) {
        Ok(c) => c,
        Err(_) => {
            return error_response(ctx.seq_id, ResultCode::ValidationFailed, "消息解码失败");
        }
    };

    let session = match ctx.session_required() {
        Ok(s) => s,
        Err(code) => return error_response(ctx.seq_id, code, "会话状态异常"),
    };

    debug!("收到设备描述更新请求");

    if !validate_device_desc(&cmd.description) {
        return error_response(
            ctx.seq_id,
            ResultCode::DeviceDescFormatError,
            "设备描述格式错误",
        );
    }

    let storage = match ctx.storage() {
        Some(s) => s,
        None => {
            return error_response(ctx.seq_id, ResultCode::InternalError, "存储服务未初始化");
        }
    };

    match storage.config_set("device_description", &cmd.description) {
        Ok(()) => {
            info!(user = %session.username, desc = %cmd.description, "设备描述更新成功");
            log_operation(
                ctx,
                session,
                log_type::SYSTEM_MANAGEMENT,
                action_type::DEVICE_DESC_UPDATE,
                &cmd.description,
                0,
                None,
            );
            success_response(ctx.seq_id)
        }
        Err(e) => {
            warn!(user = %session.username, reason = %e, "设备描述更新失败");
            log_operation(
                ctx,
                session,
                log_type::SYSTEM_MANAGEMENT,
                action_type::DEVICE_DESC_UPDATE,
                &cmd.description,
                1,
                Some(&e.to_string()),
            );
            error_response(ctx.seq_id, ResultCode::InternalError, "设备描述保存失败")
        }
    }
}

/// 校验设备描述格式。
fn validate_device_desc(desc: &str) -> bool {
    !desc.is_empty()
        && desc.chars().count() <= 32
        && desc.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// 从系统配置读取值。
fn config_value(storage: &storage::Storage, key: &str) -> String {
    storage
        .config_get(key)
        .ok()
        .flatten()
        .and_then(|c| c.config_value)
        .unwrap_or_default()
}

/// 构造系统信息错误响应（使用 RspCommon 传递错误信息）。
fn sysinfo_error(seq_id: u32, code: ResultCode, msg: &str) -> Vec<u8> {
    let rsp = RspCommon {
        success: false,
        result_code: code.as_u16() as i32,
        error_message: msg.to_string(),
    };
    codec::encode_frame(RSP_COMMON, seq_id, &rsp.encode_to_vec()).unwrap_or_default()
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

/// 校验升级包 SHA-256 摘要。checksum 为空时跳过校验（向后兼容）。
fn verify_sha256(data: &[u8], checksum: &str) -> bool {
    if checksum.is_empty() {
        return true;
    }
    let digest = Sha256::digest(data);
    let hex = format!("{digest:x}");
    hex.eq_ignore_ascii_case(checksum)
}
