//! 受控文件树构建器。
//!
//! 遍历真实 U 盘挂载目录，对每个文件进行：
//! - L1: 根据 scan_result 标记病毒文件（加 [病毒禁止访问] 前缀，size=0）
//! - L2: 预读 magic bytes 检测可执行文件类型
//! - L4: 解析根目录 autorun.inf 提取引用的可执行文件路径

use std::collections::HashSet;
use std::path::Path;

use tracing::warn;

use crate::autorun;
use crate::exec_detect;
use crate::types::ControlledEntry;

/// 病毒文件名前缀。
pub const VIRUS_PREFIX: &str = "[病毒禁止访问]";

/// 构建受控文件树。
///
/// 参数:
///   - mount_path: U 盘挂载路径（如 /mnt/usb_raw）。
///   - infected_files: 病毒文件相对路径列表（来自 ScanResult.infected_files）。
///
/// 返回:
///   - 根目录下的受控文件节点列表。
pub fn build_file_tree(mount_path: &Path, infected_files: &[String]) -> Vec<ControlledEntry> {
    let infected_set: HashSet<String> = infected_files
        .iter()
        .map(|p| normalize_path(p))
        .collect();

    // 先解析 autorun.inf（如果存在）
    let autorun_targets = parse_root_autorun(mount_path);

    build_directory(mount_path, mount_path, &infected_set, &autorun_targets, true)
}

/// 递归构建目录。
fn build_directory(
    mount_path: &Path,
    dir_path: &Path,
    infected_set: &HashSet<String>,
    autorun_targets: &HashSet<String>,
    is_root: bool,
) -> Vec<ControlledEntry> {
    let read_dir = match std::fs::read_dir(dir_path) {
        Ok(rd) => rd,
        Err(e) => {
            warn!("读取目录失败 {}: {}", dir_path.display(), e);
            return Vec::new();
        }
    };

    let mut entries = Vec::new();

    for entry in read_dir {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                warn!("读取目录项失败: {}", e);
                continue;
            }
        };

        // 跳过符号链接，防止路径穿越
        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(e) => {
                warn!("读取文件类型失败 {}: {}", entry.path().display(), e);
                continue;
            }
        };
        if file_type.is_symlink() {
            warn!("跳过符号链接: {}", entry.path().display());
            continue;
        }

        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(e) => {
                warn!("读取元数据失败 {}: {}", entry.path().display(), e);
                continue;
            }
        };

        let file_name = entry.file_name().to_string_lossy().to_string();
        let real_path = entry.path();
        let relative = relative_path(&real_path, mount_path);
        let is_dir = metadata.is_dir();

        let is_virus = infected_set.contains(&normalize_path(&relative));
        let file_size = if is_virus { 0 } else { metadata.len() };

        let virtual_name = if is_virus {
            format!("{}{}", VIRUS_PREFIX, file_name)
        } else {
            file_name.clone()
        };

        let exec_type = if !is_dir && !is_virus {
            exec_detect::detect_exec_type(&real_path)
        } else {
            None
        };

        let extension = if is_dir {
            String::new()
        } else {
            extract_extension(&file_name)
        };

        let is_autorun_inf = is_root && file_name.eq_ignore_ascii_case("autorun.inf");

        let is_autorun_target = if !is_dir {
            let rel_lower = relative.replace('\\', "/").to_lowercase();
            autorun_targets.contains(&rel_lower)
        } else {
            false
        };

        let is_root_shell_script = is_root && is_shell_script(&file_name);

        let children = if is_dir {
            build_directory(mount_path, &real_path, infected_set, autorun_targets, false)
        } else {
            Vec::new()
        };

        entries.push(ControlledEntry {
            real_path,
            virtual_name,
            file_size,
            is_dir,
            is_virus,
            exec_type,
            extension,
            is_autorun_target,
            is_autorun_inf,
            is_root_shell_script,
            children,
        });
    }

    // 目录按名称排序，保持稳定顺序
    entries.sort_by(|a, b| a.virtual_name.cmp(&b.virtual_name));
    entries
}

/// 解析根目录 autorun.inf 中引用的可执行文件路径。
fn parse_root_autorun(mount_path: &Path) -> HashSet<String> {
    let autorun_path = mount_path.join("autorun.inf");
    if !autorun_path.exists() {
        // 尝试大小写变体
        let autorun_path_upper = mount_path.join("AUTORUN.INF");
        if !autorun_path_upper.exists() {
            return HashSet::new();
        }
        return parse_autorun_file(&autorun_path_upper);
    }
    parse_autorun_file(&autorun_path)
}

/// 解析 autorun.inf 文件。
fn parse_autorun_file(path: &Path) -> HashSet<String> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            warn!("读取 autorun.inf 失败: {}", e);
            return HashSet::new();
        }
    };

    autorun::parse_autorun_targets(&content)
        .into_iter()
        .map(|p| {
            // 统一分隔符并转小写，保留完整相对路径
            p.replace('\\', "/").to_lowercase()
        })
        .collect()
}

/// 计算相对路径。
fn relative_path(full_path: &Path, mount_path: &Path) -> String {
    full_path
        .strip_prefix(mount_path)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| full_path.to_string_lossy().to_string())
}

/// 规范化路径（统一斜杠、去掉前导斜杠）。
fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
        .trim_start_matches('/')
        .to_string()
}

/// 提取文件后缀（小写，不含点号）。
fn extract_extension(file_name: &str) -> String {
    file_name
        .rsplit('.')
        .next()
        .filter(|ext| ext.len() < file_name.len())
        .map(|ext| ext.to_lowercase())
        .unwrap_or_default()
}

/// 判断是否为 shell 脚本。
fn is_shell_script(file_name: &str) -> bool {
    let lower = file_name.to_lowercase();
    lower.ends_with(".sh") || lower.ends_with(".bash")
}
