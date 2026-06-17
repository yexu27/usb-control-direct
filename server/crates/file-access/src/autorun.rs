//! autorun.inf 解析器。
//!
//! 解析 INI 格式的 autorun.inf 内容，提取引用的可执行文件路径。
//! 用于 L4 自运行控制策略。

/// 解析 autorun.inf 内容，提取引用的可执行文件路径。
///
/// 识别的键：
///   - `open=` — 直接执行的程序
///   - `shellexecute=` — 通过 ShellExecute 执行的程序
///   - `shell\*\command=` — 自定义 shell 动词命令
///
/// 参数:
///   - content: autorun.inf 文件的文本内容。
///
/// 返回:
///   - 去重后的可执行文件路径列表（保留原始路径大小写和子目录）。
pub fn parse_autorun_targets(content: &str) -> Vec<String> {
    let mut targets = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('[') || trimmed.starts_with(';') {
            continue;
        }

        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };

        let key_lower = key.trim().to_lowercase();
        let value = value.trim();

        let is_target_key = key_lower == "open"
            || key_lower == "shellexecute"
            || is_shell_command_key(&key_lower);

        if !is_target_key || value.is_empty() {
            continue;
        }

        let exe_path = extract_executable_path(value);
        if !exe_path.is_empty() && seen.insert(exe_path.clone()) {
            targets.push(exe_path);
        }
    }

    targets
}

/// 判断键名是否匹配 `shell\*\command` 模式。
fn is_shell_command_key(key: &str) -> bool {
    if !key.starts_with("shell\\") && !key.starts_with("shell/") {
        return false;
    }
    key.ends_with("\\command") || key.ends_with("/command")
}

/// 从值字符串中提取可执行文件路径（去掉参数和引号）。
fn extract_executable_path(value: &str) -> String {
    let value = value.trim();

    // 处理引号包裹的路径
    if let Some(stripped) = value.strip_prefix('"') {
        if let Some(end) = stripped.find('"') {
            return stripped[..end].to_string();
        }
        return stripped.to_string();
    }

    // 无引号时，取第一个空格前的部分作为路径
    match value.find(' ') {
        Some(pos) => value[..pos].to_string(),
        None => value.to_string(),
    }
}
