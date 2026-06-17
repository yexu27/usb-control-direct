use std::io::Write;

use file_access::exec_detect::detect_exec_type;
use file_access::types::ExecFileType;

#[test]
fn detect_pe_exe_from_valid_mz_header() {
    let mut data = vec![0u8; 256];
    // MZ 头
    data[0] = b'M';
    data[1] = b'Z';
    // e_lfanew 偏移在 0x3C，指向 PE 签名位置
    data[0x3C] = 0x80;
    data[0x3D] = 0x00;
    data[0x3E] = 0x00;
    data[0x3F] = 0x00;
    // PE 签名在 offset 0x80
    data[0x80] = b'P';
    data[0x81] = b'E';
    data[0x82] = 0x00;
    data[0x83] = 0x00;
    // COFF 特征字段在 PE 签名后 22 字节 (offset 0x80 + 4 + 18 = 0x96)
    // IMAGE_FILE_DLL = 0x2000，不设置则为 EXE
    data[0x96] = 0x00;
    data[0x97] = 0x00;

    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), &data).unwrap();

    let result = detect_exec_type(tmp.path());
    assert_eq!(result, Some(ExecFileType::Pe));
}

#[test]
fn detect_pe_dll_from_characteristics() {
    let mut data = vec![0u8; 256];
    data[0] = b'M';
    data[1] = b'Z';
    data[0x3C] = 0x80;
    data[0x80] = b'P';
    data[0x81] = b'E';
    data[0x82] = 0x00;
    data[0x83] = 0x00;
    // IMAGE_FILE_DLL = 0x2000
    data[0x96] = 0x00;
    data[0x97] = 0x20;

    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), &data).unwrap();

    let result = detect_exec_type(tmp.path());
    assert_eq!(result, Some(ExecFileType::Dll));
}

#[test]
fn detect_elf_executable() {
    let mut data = vec![0u8; 64];
    data[0] = 0x7F;
    data[1] = b'E';
    data[2] = b'L';
    data[3] = b'F';

    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), &data).unwrap();

    let result = detect_exec_type(tmp.path());
    assert_eq!(result, Some(ExecFileType::Elf));
}

#[test]
fn detect_non_executable_returns_none() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), b"Hello, world!").unwrap();

    let result = detect_exec_type(tmp.path());
    assert_eq!(result, None);
}

#[test]
fn detect_empty_file_returns_none() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let result = detect_exec_type(tmp.path());
    assert_eq!(result, None);
}

#[test]
fn detect_mz_without_pe_signature_returns_none() {
    let mut data = vec![0u8; 256];
    data[0] = b'M';
    data[1] = b'Z';
    data[0x3C] = 0x80;
    // PE 签名位置填入垃圾数据
    data[0x80] = 0xFF;
    data[0x81] = 0xFF;

    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), &data).unwrap();

    let result = detect_exec_type(tmp.path());
    assert_eq!(result, None);
}

#[test]
fn detect_nonexistent_file_returns_none() {
    let result = detect_exec_type(std::path::Path::new("/nonexistent/path/file.exe"));
    assert_eq!(result, None);
}
