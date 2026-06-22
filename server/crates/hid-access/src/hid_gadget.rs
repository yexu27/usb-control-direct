//! HID gadget configfs 配置与 hidg 节点发现。
//!
//! 在 USB gadget 的 functions/ 目录下创建并配置 hid.usbN 实例，
//! 服务启动时执行一次，设备接入时只通过 /dev/hidgX 读写。

use std::fs;
use std::path::{Path, PathBuf};

use tracing::info;

use crate::error::HidAccessError;

/// HID function 名称常量。
pub const KEYBOARD_FUNCTION: &str = "hid.usb0";
pub const MOUSE_FUNCTION: &str = "hid.usb1";

/// 配置单个 HID function。
///
/// 参数:
///   - `function_dir`: function 的 configfs 目录路径。
///   - `protocol`: USB HID protocol（1=keyboard, 2=mouse）。
///   - `report_desc`: HID report descriptor 字节序列。
///   - `report_len`: HID report 长度（字节）。
pub fn configure_hid_function(
    function_dir: &Path,
    protocol: u8,
    report_desc: &[u8],
    report_len: usize,
) -> Result<(), HidAccessError> {
    fs::create_dir_all(function_dir).map_err(|e| {
        HidAccessError::Internal(format!(
            "创建 HID function 目录 {} 失败: {}",
            function_dir.display(),
            e
        ))
    })?;

    info!(
        function = %function_dir.display(),
        protocol,
        report_len,
        report_desc_len = report_desc.len(),
        "配置 HID function"
    );

    write_configfs_attr(function_dir, "protocol", &protocol.to_string())?;
    write_configfs_attr(function_dir, "subclass", "1")?;
    write_configfs_attr(function_dir, "report_length", &report_len.to_string())?;
    write_configfs_attr(function_dir, "report_desc", report_desc)?;

    Ok(())
}

/// 写单个 configfs 属性文件。
fn write_configfs_attr(
    dir: &Path,
    filename: &str,
    value: impl AsRef<[u8]>,
) -> Result<(), HidAccessError> {
    let path = dir.join(filename);
    fs::write(&path, value).map_err(|e| {
        HidAccessError::Internal(format!(
            "写 configfs {} 失败: {}",
            path.display(),
            e
        ))
    })
}

/// 已发现的 HID gadget 设备节点。
#[derive(Debug, Clone)]
pub struct HidgNodes {
    pub keyboard: PathBuf,
    pub mouse: PathBuf,
}

/// 扫描 /dev/hidg* 并确定键盘/鼠标对应关系。
///
/// configfs 按 function 链接顺序创建 hidg 节点，
/// 第一个发现的节点分配给键盘（hid.usb0），第二个分配给鼠标（hid.usb1）。
pub fn discover_hidg_nodes() -> Result<HidgNodes, HidAccessError> {
    let mut nodes: Vec<PathBuf> = Vec::new();

    let dev_dir = fs::read_dir("/dev").map_err(|e| {
        HidAccessError::Internal(format!("读取 /dev 目录失败: {}", e))
    })?;

    for entry in dev_dir {
        let entry = entry.map_err(|e| {
            HidAccessError::Internal(format!("读取 /dev 条目失败: {}", e))
        })?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("hidg") {
            nodes.push(entry.path());
        }
    }
    nodes.sort();

    if nodes.len() < 2 {
        return Err(HidAccessError::Internal(format!(
            "未找到足够的 hidg 节点（需要至少 2 个，实际 {} 个）",
            nodes.len()
        )));
    }

    let mapping = HidgNodes {
        keyboard: nodes[0].clone(),
        mouse: nodes[1].clone(),
    };

    info!(?mapping, "发现 hidg 节点");

    Ok(mapping)
}
