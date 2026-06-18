//! 日志 ZIP 导出。
//!
//! 将查询结果打包为 .zip 字节流。
//! UTF-8 文件名，兼容 Windows 中文。

use std::io::Write;

use crate::error::AuditError;

/// 生成 ZIP 文件字节流。
///
/// 参数:
/// - `filename`: ZIP 内文件名。
/// - `csv_content`: CSV 内容字符串。
///
/// 返回:
/// - 成功时返回 ZIP 字节序列。
pub fn generate_zip(filename: &str, csv_content: &str) -> Result<Vec<u8>, AuditError> {
    let mut buf = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        zip.start_file(filename, options)
            .map_err(|e| AuditError::ExportFailed(e.to_string()))?;
        zip.write_all(csv_content.as_bytes())
            .map_err(|e| AuditError::ExportFailed(e.to_string()))?;
        zip.finish()
            .map_err(|e| AuditError::ExportFailed(e.to_string()))?;
    }
    Ok(buf)
}

/// 生成导出文件名。
///
/// 参数:
/// - `log_type`: 日志类型标识符。
///
/// 返回:
/// - 带时间戳的 ZIP 文件名。
pub fn generate_filename(log_type: &str) -> String {
    let now = chrono::Local::now();
    let timestamp = now.format("%Y%m%d%H%M%S");
    match log_type {
        "usb_audit" => format!("USBUsageLog{}.zip", timestamp),
        "malware" => format!("MalwareDetectionLog{}.zip", timestamp),
        "operation" => format!("OperationLog{}.zip", timestamp),
        _ => format!("Log{}.zip", timestamp),
    }
}
