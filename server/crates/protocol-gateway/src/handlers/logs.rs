//! 日志查询/导出/清理 handler（0x0400/0x0402/0x0404）。

use prost::Message;

use tracing::{debug, error, info, warn};

use common::audit_const::{action_type, log_type};
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
            log_operation(ctx, session, log_type::LOG_MANAGEMENT, action_type::LOG_EXPORT, &cmd.log_type, 0, None);
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
                log_type::LOG_MANAGEMENT,
                action_type::LOG_EXPORT,
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
                log_type::LOG_MANAGEMENT,
                action_type::LOG_CLEAN,
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
                log_type::LOG_MANAGEMENT,
                action_type::LOG_CLEAN,
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
            .and_then(|r| common::mapping::process_result_int_to_str(r).ok())
            .unwrap_or("")
            .to_string(),
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

/// 扫描结果数字转中文（CSV 导出用）。
fn scan_result_to_chinese(value: i32) -> &'static str {
    match value {
        0 => "未发现病毒",
        1 => "发现病毒",
        2 => "扫描失败",
        3 => "扫描中断",
        _ => "未知",
    }
}

/// 处理结果数字转中文（CSV 导出用）。
fn process_result_to_chinese(value: i32) -> &'static str {
    match value {
        0 => "已标记阻断",
        1 => "已阻断",
        2 => "处理失败",
        3 => "无需处理",
        _ => "未知",
    }
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
            scan_result_to_chinese(item.scan_result),
            escape_csv(item.virus_name.as_deref().unwrap_or("")),
            item.virus_db_version.as_deref().unwrap_or(""),
            item.process_result.map(process_result_to_chinese).unwrap_or(""),
            escape_csv(item.fail_reason.as_deref().unwrap_or("")),
            escape_csv(item.detail.as_deref().unwrap_or("")),
        ));
    }
    csv
}

/// 操作结果数字转中文。
fn result_int_to_str(result: i32) -> &'static str {
    match result {
        0 => "成功",
        _ => "失败",
    }
}

/// 角色数字转中文。
fn role_int_to_chinese(role: i32) -> &'static str {
    match role {
        0 => "管理员",
        1 => "操作员",
        2 => "审计员",
        _ => "未知",
    }
}

/// log_type 转 PRD 页面分类中文名。
fn log_type_to_display(log_type: &str) -> &'static str {
    match log_type {
        "login_auth" => "登录认证",
        "user_management" => "用户管理",
        "security_config" => "安全配置变更",
        "auth_management" => "授权管理",
        "system_management" => "系统管理",
        "program_upgrade" => "程序升级",
        "log_management" => "日志管理",
        _ => "未知",
    }
}

/// action_type 转中文动作描述。
fn action_type_to_display(action_type: &str) -> &'static str {
    match action_type {
        "login" => "用户登录",
        "logout" => "用户登出",
        "login_failed" => "用户登录失败",
        "password_change" => "修改密码",
        "password_reset" => "重置密码",
        "user_create" => "新建用户",
        "user_delete" => "删除用户",
        "whitelist_add" => "添加白名单设备",
        "whitelist_remove" => "删除白名单设备",
        "whitelist_update" => "修改白名单",
        "file_policy_update" => "修改文件访问控制策略",
        "blacklist_add" => "添加文件类型黑名单",
        "blacklist_remove" => "删除文件类型黑名单",
        "policy_import" => "导入策略",
        "policy_export" => "导出策略",
        "system_upgrade" => "系统升级",
        "virusdb_upgrade" => "病毒库升级",
        "auth_upload" => "上传授权文件",
        "machine_code_download" => "下载机器码",
        "device_desc_update" => "修改设备描述",
        "log_clean" => "清理日志",
        "log_export" => "导出日志",
        _ => "未知",
    }
}

/// 生成操作日志 CSV。
fn generate_operation_csv(items: &[storage::model::OperationLog]) -> String {
    let mut csv = String::from(
        "ID,操作时间,用户名,角色,日志分类,操作类型,操作目标,操作前值,操作后值,关联文件,关联版本,结果,失败原因,来源IP,详情\n",
    );
    for item in items {
        let role = role_int_to_chinese(item.role);
        let log_type_display = log_type_to_display(&item.log_type);
        let action_display =
            action_type_to_display(item.action_type.as_deref().unwrap_or(""));
        let result_display = result_int_to_str(item.result);

        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            item.id,
            item.op_time,
            item.username,
            role,
            log_type_display,
            action_display,
            escape_csv(item.target.as_deref().unwrap_or("")),
            escape_csv(item.before_value.as_deref().unwrap_or("")),
            escape_csv(item.after_value.as_deref().unwrap_or("")),
            escape_csv(item.related_file.as_deref().unwrap_or("")),
            item.related_version.as_deref().unwrap_or(""),
            result_display,
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
