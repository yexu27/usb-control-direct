//! 日志查询/导出/清理 handler（0x0400/0x0402/0x0404）。

use prost::Message;

use tracing::{debug, error, info, warn};

use common::code::ResultCode;
use common::proto::{
    CmdDeleteLogs, CmdExportLogs, CmdQueryLogs, MalwareLogEntry, OperationLogEntry,
    RspCommon, RspExportLogs, RspQueryLogs, UsbAuditLogEntry,
};
use log_audit::query::LogType;
use storage::model::LogQueryParams;

use super::audit_helper::log_operation;

use crate::codec;
use crate::context::RequestContext;

/// RspQueryLogs 消息类型。
const RSP_QUERY_LOGS: u32 = 0x0401;

/// RspExportLogs 消息类型。
const RSP_EXPORT_LOGS: u32 = 0x0403;

/// RspCommon 消息类型。
const RSP_COMMON: u32 = 0xFF00;

/// CMD_QUERY_LOGS (0x0400) handler。
pub fn handle_query_logs(ctx: &RequestContext, payload: &[u8]) -> Vec<u8> {
    let cmd = match CmdQueryLogs::decode(payload) {
        Ok(c) => c,
        Err(_) => {
            return query_error(ctx.seq_id, ResultCode::ValidationFailed, "消息解码失败");
        }
    };

    debug!(log_type = %cmd.log_type, page = cmd.page, "收到日志查询请求");

    let log_type = match LogType::parse(&cmd.log_type) {
        Some(t) => t,
        None => {
            return query_error(ctx.seq_id, ResultCode::LogTypeInvalid, "无效的日志类型");
        }
    };

    let storage = match ctx.storage() {
        Some(s) => s,
        None => {
            return query_error(ctx.seq_id, ResultCode::InternalError, "存储服务未初始化");
        }
    };

    let params = build_query_params(&cmd);

    match log_type {
        LogType::UsbAudit => match storage.usb_audit_query_paged(&params) {
            Ok((items, total)) => {
                debug!(log_type = %cmd.log_type, total = total, "日志查询成功");
                let entries: Vec<UsbAuditLogEntry> = items.iter().map(map_usb_audit).collect();
                let rsp = RspQueryLogs {
                    success: true,
                    usb_audit_entries: entries,
                    malware_entries: Vec::new(),
                    operation_entries: Vec::new(),
                    total: total as i32,
                    page: cmd.page,
                    page_size: cmd.page_size,
                    result_code: ResultCode::Success.as_u16() as i32,
                    error_message: String::new(),
                };
                codec::encode_frame(RSP_QUERY_LOGS, ctx.seq_id, &rsp.encode_to_vec())
                    .unwrap_or_default()
            }
            Err(_e) => {
                error!(log_type = %cmd.log_type, reason = %_e, "日志查询失败");
                query_error(ctx.seq_id, ResultCode::LogQueryFailed, "日志查询失败")
            }
        },
        LogType::Malware => match storage.malware_query_paged(&params) {
            Ok((items, total)) => {
                debug!(log_type = %cmd.log_type, total = total, "日志查询成功");
                let entries: Vec<MalwareLogEntry> = items.iter().map(map_malware).collect();
                let rsp = RspQueryLogs {
                    success: true,
                    usb_audit_entries: Vec::new(),
                    malware_entries: entries,
                    operation_entries: Vec::new(),
                    total: total as i32,
                    page: cmd.page,
                    page_size: cmd.page_size,
                    result_code: ResultCode::Success.as_u16() as i32,
                    error_message: String::new(),
                };
                codec::encode_frame(RSP_QUERY_LOGS, ctx.seq_id, &rsp.encode_to_vec())
                    .unwrap_or_default()
            }
            Err(_e) => {
                error!(log_type = %cmd.log_type, reason = %_e, "日志查询失败");
                query_error(ctx.seq_id, ResultCode::LogQueryFailed, "日志查询失败")
            }
        },
        LogType::Operation => match storage.operation_log_query_paged(&params) {
            Ok((items, total)) => {
                debug!(log_type = %cmd.log_type, total = total, "日志查询成功");
                let entries: Vec<OperationLogEntry> =
                    items.iter().map(map_operation).collect();
                let rsp = RspQueryLogs {
                    success: true,
                    usb_audit_entries: Vec::new(),
                    malware_entries: Vec::new(),
                    operation_entries: entries,
                    total: total as i32,
                    page: cmd.page,
                    page_size: cmd.page_size,
                    result_code: ResultCode::Success.as_u16() as i32,
                    error_message: String::new(),
                };
                codec::encode_frame(RSP_QUERY_LOGS, ctx.seq_id, &rsp.encode_to_vec())
                    .unwrap_or_default()
            }
            Err(_e) => {
                error!(log_type = %cmd.log_type, reason = %_e, "日志查询失败");
                query_error(ctx.seq_id, ResultCode::LogQueryFailed, "日志查询失败")
            }
        },
    }
}

/// CMD_EXPORT_LOGS (0x0402) handler。
pub fn handle_export_logs(ctx: &RequestContext, payload: &[u8]) -> Vec<u8> {
    let cmd = match CmdExportLogs::decode(payload) {
        Ok(c) => c,
        Err(_) => {
            return export_error(ctx.seq_id, ResultCode::ValidationFailed, "消息解码失败");
        }
    };

    let session = match ctx.session_required() {
        Ok(s) => s,
        Err(code) => return export_error(ctx.seq_id, code, "会话状态异常"),
    };

    debug!(user = %session.username, log_type = %cmd.log_type, "收到日志导出请求");

    let log_type = match LogType::parse(&cmd.log_type) {
        Some(t) => t,
        None => {
            return export_error(ctx.seq_id, ResultCode::LogTypeInvalid, "无效的日志类型");
        }
    };

    let storage = match ctx.storage() {
        Some(s) => s,
        None => {
            return export_error(ctx.seq_id, ResultCode::InternalError, "存储服务未初始化");
        }
    };

    // 导出查询全量，分批查询避免单次查询过大
    const EXPORT_PAGE_SIZE: i32 = 5_000;

    let base_params = LogQueryParams {
        start_time: if cmd.start_time > 0 {
            Some(cmd.start_time)
        } else {
            None
        },
        end_time: if cmd.end_time > 0 {
            Some(cmd.end_time)
        } else {
            None
        },
        keyword: optional_str(&cmd.keyword),
        event_type: optional_str(&cmd.event_type),
        log_category: optional_str(&cmd.log_category),
        action_type: optional_str(&cmd.action_type),
        page: 1,
        page_size: EXPORT_PAGE_SIZE,
    };

    let csv_content = match log_type {
        LogType::UsbAudit => {
            let mut all_items = Vec::new();
            let mut page = 1;
            loop {
                let mut params = base_params.clone();
                params.page = page;
                match storage.usb_audit_query_paged(&params) {
                    Ok((items, _)) => {
                        let count = items.len();
                        all_items.extend(items);
                        if (count as i32) < EXPORT_PAGE_SIZE {
                            break;
                        }
                        page += 1;
                    }
                    Err(_e) => {
                        return export_error(ctx.seq_id, ResultCode::LogExportFailed, "日志查询失败");
                    }
                }
            }
            generate_usb_audit_csv(&all_items)
        }
        LogType::Malware => {
            let mut all_items = Vec::new();
            let mut page = 1;
            loop {
                let mut params = base_params.clone();
                params.page = page;
                match storage.malware_query_paged(&params) {
                    Ok((items, _)) => {
                        let count = items.len();
                        all_items.extend(items);
                        if (count as i32) < EXPORT_PAGE_SIZE {
                            break;
                        }
                        page += 1;
                    }
                    Err(_e) => {
                        return export_error(ctx.seq_id, ResultCode::LogExportFailed, "日志查询失败");
                    }
                }
            }
            generate_malware_csv(&all_items)
        }
        LogType::Operation => {
            let mut all_items = Vec::new();
            let mut page = 1;
            loop {
                let mut params = base_params.clone();
                params.page = page;
                match storage.operation_log_query_paged(&params) {
                    Ok((items, _)) => {
                        let count = items.len();
                        all_items.extend(items);
                        if (count as i32) < EXPORT_PAGE_SIZE {
                            break;
                        }
                        page += 1;
                    }
                    Err(_e) => {
                        return export_error(ctx.seq_id, ResultCode::LogExportFailed, "日志查询失败");
                    }
                }
            }
            generate_operation_csv(&all_items)
        }
    };

    let suggested_filename = log_audit::export::generate_filename(&cmd.log_type);
    let csv_filename = suggested_filename.replace(".zip", ".csv");

    match log_audit::export::generate_zip(&csv_filename, &csv_content) {
        Ok(zip_data) => {
            info!(user = %session.username, log_type = %cmd.log_type, size = zip_data.len(), "日志导出成功");
            log_operation(ctx, session, "log_management", "export", &cmd.log_type, 0, None);
            let rsp = RspExportLogs {
                success: true,
                zip_data,
                suggested_filename,
                result_code: ResultCode::Success.as_u16() as i32,
                error_message: String::new(),
            };
            codec::encode_frame(RSP_EXPORT_LOGS, ctx.seq_id, &rsp.encode_to_vec())
                .unwrap_or_default()
        }
        Err(e) => {
            warn!(user = %session.username, log_type = %cmd.log_type, reason = %e, "日志导出失败");
            log_operation(
                ctx,
                session,
                "log_management",
                "export",
                &cmd.log_type,
                1,
                Some(&e.to_string()),
            );
            export_error(ctx.seq_id, ResultCode::LogExportFailed, "日志导出失败")
        }
    }
}

/// CMD_DELETE_LOGS (0x0404) handler。
pub fn handle_delete_logs(ctx: &RequestContext, payload: &[u8]) -> Vec<u8> {
    let cmd = match CmdDeleteLogs::decode(payload) {
        Ok(c) => c,
        Err(_) => {
            return error_response(ctx.seq_id, ResultCode::ValidationFailed, "消息解码失败");
        }
    };

    let session = match ctx.session_required() {
        Ok(s) => s,
        Err(code) => return error_response(ctx.seq_id, code, "会话状态异常"),
    };

    debug!(user = %session.username, log_type = %cmd.log_type, "收到日志清理请求");

    let log_type = match LogType::parse(&cmd.log_type) {
        Some(t) => t,
        None => {
            return error_response(ctx.seq_id, ResultCode::LogTypeInvalid, "无效的日志类型");
        }
    };

    // 校验时间范围合法性
    if cmd.start_time <= 0 || cmd.end_time <= 0 {
        return error_response(
            ctx.seq_id,
            ResultCode::ValidationFailed,
            "起始时间和结束时间必须为正整数",
        );
    }
    if cmd.start_time > cmd.end_time {
        return error_response(
            ctx.seq_id,
            ResultCode::ValidationFailed,
            "起始时间不能大于结束时间",
        );
    }

    // 校验半年保留期
    if let Err(_e) = log_audit::query::validate_delete_time(cmd.end_time) {
        return error_response(
            ctx.seq_id,
            ResultCode::LogRetentionViolation,
            "清理范围违反半年保留期规则",
        );
    }

    let storage = match ctx.storage() {
        Some(s) => s,
        None => {
            return error_response(ctx.seq_id, ResultCode::InternalError, "存储服务未初始化");
        }
    };

    let delete_result = match log_type {
        LogType::UsbAudit => storage.usb_audit_delete_by_time(cmd.start_time, cmd.end_time),
        LogType::Malware => storage.malware_delete_by_time(cmd.start_time, cmd.end_time),
        LogType::Operation => {
            storage.operation_log_delete_by_time(cmd.start_time, cmd.end_time)
        }
    };

    match delete_result {
        Ok(count) => {
            info!(user = %session.username, log_type = %cmd.log_type, deleted = count, "日志清理成功");
            log_operation(
                ctx,
                session,
                "log_management",
                "delete",
                &format!("{}({}条)", cmd.log_type, count),
                0,
                None,
            );
            success_response(ctx.seq_id)
        }
        Err(e) => {
            warn!(user = %session.username, log_type = %cmd.log_type, reason = %e, "日志清理失败");
            log_operation(
                ctx,
                session,
                "log_management",
                "delete",
                &cmd.log_type,
                1,
                Some(&e.to_string()),
            );
            error_response(ctx.seq_id, ResultCode::InternalError, "日志清理失败")
        }
    }
}

/// 单次查询最大分页大小。
const MAX_PAGE_SIZE: i32 = 1000;

/// 构建 LogQueryParams。
fn build_query_params(cmd: &CmdQueryLogs) -> LogQueryParams {
    let page_size = cmd.page_size.clamp(1, MAX_PAGE_SIZE);
    let page = cmd.page.max(1);
    LogQueryParams {
        start_time: if cmd.start_time > 0 {
            Some(cmd.start_time)
        } else {
            None
        },
        end_time: if cmd.end_time > 0 {
            Some(cmd.end_time)
        } else {
            None
        },
        keyword: optional_str(&cmd.keyword),
        event_type: optional_str(&cmd.event_type),
        log_category: optional_str(&cmd.log_category),
        action_type: optional_str(&cmd.action_type),
        page,
        page_size,
    }
}

/// 空字符串转 None。
fn optional_str(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

/// UsbAuditLog → UsbAuditLogEntry。
fn map_usb_audit(item: &storage::model::UsbAuditLog) -> UsbAuditLogEntry {
    UsbAuditLogEntry {
        id: item.id,
        event_time: item.event_time,
        device_sn: item.device_sn.clone().unwrap_or_default(),
        device_name: item.device_name.clone().unwrap_or_default(),
        device_type: item.device_type.clone().unwrap_or_default(),
        interface_type: item.interface_type.clone().unwrap_or_default(),
        event_type: item.event_type.clone(),
        permission: item
            .permission
            .and_then(|p| common::mapping::permission_int_to_str(p).ok())
            .unwrap_or("")
            .to_string(),
        capacity_bytes: item.capacity_bytes.unwrap_or(0),
        file_path: item.file_path.clone().unwrap_or_default(),
        matched_policy: item.matched_policy.clone().unwrap_or_default(),
        result: item.result.clone(),
        fail_reason: item.fail_reason.clone().unwrap_or_default(),
        detail: item.detail.clone().unwrap_or_default(),
    }
}

/// MalwareLog → MalwareLogEntry。
fn map_malware(item: &storage::model::MalwareLog) -> MalwareLogEntry {
    MalwareLogEntry {
        id: item.id,
        scan_time: item.scan_time,
        device_sn: item.device_sn.clone().unwrap_or_default(),
        device_name: item.device_name.clone().unwrap_or_default(),
        file_path: item.file_path.clone().unwrap_or_default(),
        scan_result: common::mapping::scan_result_int_to_str(item.scan_result)
            .unwrap_or("unknown")
            .to_string(),
        virus_name: item.virus_name.clone().unwrap_or_default(),
        virus_db_version: item.virus_db_version.clone().unwrap_or_default(),
        process_result: item
            .process_result
            .map(|r| r.to_string())
            .unwrap_or_default(),
        fail_reason: item.fail_reason.clone().unwrap_or_default(),
        detail: item.detail.clone().unwrap_or_default(),
    }
}

/// OperationLog → OperationLogEntry。
fn map_operation(item: &storage::model::OperationLog) -> OperationLogEntry {
    OperationLogEntry {
        id: item.id,
        op_time: item.op_time,
        username: item.username.clone(),
        role: common::mapping::role_int_to_str(item.role)
            .unwrap_or("unknown")
            .to_string(),
        log_category: item.log_type.clone(),
        action_type: item.action_type.clone().unwrap_or_default(),
        target: item.target.clone().unwrap_or_default(),
        related_file: item.related_file.clone().unwrap_or_default(),
        related_version: item.related_version.clone().unwrap_or_default(),
        result: item.result.to_string(),
        fail_reason: item.fail_reason.clone().unwrap_or_default(),
        detail: item.detail.clone().unwrap_or_default(),
        source_ip: item.source_ip.clone().unwrap_or_default(),
        app_version: item.app_version.clone().unwrap_or_default(),
        session_id: item.session_id.clone().unwrap_or_default(),
        request_id: item.request_id.clone().unwrap_or_default(),
        before_value: item.before_value.clone().unwrap_or_default(),
        after_value: item.after_value.clone().unwrap_or_default(),
    }
}

/// 生成 USB 审计日志 CSV。
fn generate_usb_audit_csv(items: &[storage::model::UsbAuditLog]) -> String {
    let mut csv = String::from(
        "ID,事件时间,设备序列号,设备名称,设备类型,接口类型,事件类型,权限,容量,文件路径,匹配策略,结果,失败原因,详情\n",
    );
    for item in items {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            item.id,
            item.event_time,
            item.device_sn.as_deref().unwrap_or(""),
            escape_csv(item.device_name.as_deref().unwrap_or("")),
            item.device_type.as_deref().unwrap_or(""),
            item.interface_type.as_deref().unwrap_or(""),
            item.event_type,
            item.permission.unwrap_or(0),
            item.capacity_bytes.unwrap_or(0),
            escape_csv(item.file_path.as_deref().unwrap_or("")),
            escape_csv(item.matched_policy.as_deref().unwrap_or("")),
            item.result,
            escape_csv(item.fail_reason.as_deref().unwrap_or("")),
            escape_csv(item.detail.as_deref().unwrap_or("")),
        ));
    }
    csv
}

/// 生成恶意代码检测日志 CSV。
fn generate_malware_csv(items: &[storage::model::MalwareLog]) -> String {
    let mut csv = String::from(
        "ID,扫描时间,设备序列号,设备名称,文件路径,扫描结果,病毒名称,病毒库版本,处理结果,失败原因,详情\n",
    );
    for item in items {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{}\n",
            item.id,
            item.scan_time,
            item.device_sn.as_deref().unwrap_or(""),
            escape_csv(item.device_name.as_deref().unwrap_or("")),
            escape_csv(item.file_path.as_deref().unwrap_or("")),
            item.scan_result,
            escape_csv(item.virus_name.as_deref().unwrap_or("")),
            item.virus_db_version.as_deref().unwrap_or(""),
            item.process_result.unwrap_or(0),
            escape_csv(item.fail_reason.as_deref().unwrap_or("")),
            escape_csv(item.detail.as_deref().unwrap_or("")),
        ));
    }
    csv
}

/// 生成操作日志 CSV。
fn generate_operation_csv(items: &[storage::model::OperationLog]) -> String {
    let mut csv = String::from(
        "ID,操作时间,用户名,角色,日志分类,操作类型,操作目标,结果,失败原因,来源IP,详情\n",
    );
    for item in items {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{}\n",
            item.id,
            item.op_time,
            item.username,
            item.role,
            item.log_type,
            item.action_type.as_deref().unwrap_or(""),
            escape_csv(item.target.as_deref().unwrap_or("")),
            item.result,
            escape_csv(item.fail_reason.as_deref().unwrap_or("")),
            item.source_ip.as_deref().unwrap_or(""),
            escape_csv(item.detail.as_deref().unwrap_or("")),
        ));
    }
    csv
}

/// CSV 字段转义：包含逗号、引号、换行时用双引号包裹；
/// 以公式字符开头时加前缀单引号防止注入。
fn escape_csv(value: &str) -> String {
    let sanitized = if value.starts_with('=')
        || value.starts_with('+')
        || value.starts_with('-')
        || value.starts_with('@')
    {
        format!("'{value}")
    } else {
        value.to_string()
    };
    if sanitized.contains(',')
        || sanitized.contains('"')
        || sanitized.contains('\n')
        || sanitized.contains('\r')
    {
        format!("\"{}\"", sanitized.replace('"', "\"\""))
    } else {
        sanitized
    }
}

/// 构造 RspQueryLogs 错误响应。
fn query_error(seq_id: u32, code: ResultCode, msg: &str) -> Vec<u8> {
    let rsp = RspQueryLogs {
        success: false,
        usb_audit_entries: Vec::new(),
        malware_entries: Vec::new(),
        operation_entries: Vec::new(),
        total: 0,
        page: 0,
        page_size: 0,
        result_code: code.as_u16() as i32,
        error_message: msg.to_string(),
    };
    codec::encode_frame(RSP_QUERY_LOGS, seq_id, &rsp.encode_to_vec()).unwrap_or_default()
}

/// 构造 RspExportLogs 错误响应。
fn export_error(seq_id: u32, code: ResultCode, msg: &str) -> Vec<u8> {
    let rsp = RspExportLogs {
        success: false,
        zip_data: Vec::new(),
        suggested_filename: String::new(),
        result_code: code.as_u16() as i32,
        error_message: msg.to_string(),
    };
    codec::encode_frame(RSP_EXPORT_LOGS, seq_id, &rsp.encode_to_vec()).unwrap_or_default()
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
