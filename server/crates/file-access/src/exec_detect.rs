//! 可执行文件 magic bytes 检测。
//!
//! 通过预读文件头部字节判断是否为 PE（MZ+PE\0\0）或 ELF（\x7fELF）。
//! 用于 L2 可执行控制策略。

use std::fs::File;
use std::io::Read;
use std::path::Path;

use tracing::{debug, trace};

use crate::types::ExecFileType;

/// PE 特征字段中 IMAGE_FILE_DLL 标志位。
const IMAGE_FILE_DLL: u16 = 0x2000;

/// 检测文件的可执行类型。
///
/// 参数:
///   - path: 文件路径（真实 U 盘挂载路径下的文件）。
///
/// 返回:
///   - Some(ExecFileType) 表示检测到可执行类型，None 表示非可执行文件。
pub fn detect_exec_type(path: &Path) -> Option<ExecFileType> {
    let mut file = File::open(path).ok()?;
    let mut header = [0u8; 4];
    let n = file.read(&mut header).ok()?;
    if n < 4 {
        return None;
    }

    // ELF: \x7fELF
    if header[0] == 0x7F && header[1] == b'E' && header[2] == b'L' && header[3] == b'F' {
        debug!(file = %path.display(), exec_type = "ELF", "检测到可执行文件");
        return Some(ExecFileType::Elf);
    }

    // PE: MZ 头
    if header[0] != b'M' || header[1] != b'Z' {
        trace!(file = %path.display(), "文件非可执行类型");
        return None;
    }

    let result = detect_pe(&mut file);
    match &result {
        Some(exec_type) => {
            debug!(file = %path.display(), exec_type = ?exec_type, "检测到可执行文件");
        }
        None => {
            trace!(file = %path.display(), "文件非可执行类型");
        }
    }
    result
}

/// 从 MZ 头继续检测 PE 签名和 DLL 标志。
fn detect_pe(file: &mut File) -> Option<ExecFileType> {
    use std::io::Seek;

    // 读取 e_lfanew（offset 0x3C，4 字节小端）
    file.seek(std::io::SeekFrom::Start(0x3C)).ok()?;
    let mut lfanew_buf = [0u8; 4];
    file.read_exact(&mut lfanew_buf).ok()?;
    let pe_offset = u32::from_le_bytes(lfanew_buf) as u64;

    // 读取 PE 签名（4 字节）
    file.seek(std::io::SeekFrom::Start(pe_offset)).ok()?;
    let mut pe_sig = [0u8; 4];
    file.read_exact(&mut pe_sig).ok()?;
    if pe_sig != [b'P', b'E', 0x00, 0x00] {
        return None;
    }

    // COFF Header 紧随 PE 签名后，Characteristics 在 COFF Header 偏移 +18
    // COFF Header 起始 = pe_offset + 4
    // Characteristics = pe_offset + 4 + 18 = pe_offset + 22
    let characteristics_offset = pe_offset + 22;
    file.seek(std::io::SeekFrom::Start(characteristics_offset)).ok()?;
    let mut chars_buf = [0u8; 2];
    file.read_exact(&mut chars_buf).ok()?;
    let characteristics = u16::from_le_bytes(chars_buf);

    if characteristics & IMAGE_FILE_DLL != 0 {
        Some(ExecFileType::Dll)
    } else {
        Some(ExecFileType::Pe)
    }
}
