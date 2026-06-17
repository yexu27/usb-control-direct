use std::fs;

use file_access::file_tree::build_file_tree;
use file_access::types::ExecFileType;

/// 创建临时目录结构用于测试。
fn create_test_usb(dir: &std::path::Path) {
    // 普通文本文件
    fs::write(dir.join("readme.txt"), b"Hello").unwrap();

    // PE 可执行文件
    let mut pe_data = vec![0u8; 256];
    pe_data[0] = b'M';
    pe_data[1] = b'Z';
    pe_data[0x3C] = 0x80;
    pe_data[0x80] = b'P';
    pe_data[0x81] = b'E';
    pe_data[0x82] = 0x00;
    pe_data[0x83] = 0x00;
    fs::write(dir.join("setup.exe"), &pe_data).unwrap();

    // ELF 文件
    let mut elf_data = vec![0u8; 64];
    elf_data[0] = 0x7F;
    elf_data[1] = b'E';
    elf_data[2] = b'L';
    elf_data[3] = b'F';
    fs::write(dir.join("app.bin"), &elf_data).unwrap();

    // 子目录
    fs::create_dir(dir.join("docs")).unwrap();
    fs::write(dir.join("docs").join("manual.pdf"), b"PDF data").unwrap();

    // autorun.inf
    fs::write(
        dir.join("autorun.inf"),
        "[autorun]\r\nopen=setup.exe\r\n",
    )
    .unwrap();

    // shell 脚本
    fs::write(dir.join("run.sh"), "#!/bin/bash\necho hi").unwrap();
}

#[test]
fn build_tree_detects_exec_types() {
    let tmp = tempfile::tempdir().unwrap();
    create_test_usb(tmp.path());

    let infected_files: Vec<String> = vec![];
    let tree = build_file_tree(tmp.path(), &infected_files);

    let setup = tree.iter().find(|e| e.virtual_name == "setup.exe").unwrap();
    assert_eq!(setup.exec_type, Some(ExecFileType::Pe));
    assert!(!setup.is_virus);
}

#[test]
fn build_tree_marks_virus_files() {
    let tmp = tempfile::tempdir().unwrap();
    create_test_usb(tmp.path());

    let infected_files = vec!["setup.exe".to_string()];
    let tree = build_file_tree(tmp.path(), &infected_files);

    let setup = tree.iter().find(|e| e.virtual_name.contains("setup.exe")).unwrap();
    assert!(setup.is_virus);
    assert!(setup.virtual_name.starts_with("[病毒禁止访问]"));
}

#[test]
fn build_tree_parses_autorun_targets() {
    let tmp = tempfile::tempdir().unwrap();
    create_test_usb(tmp.path());

    let infected_files: Vec<String> = vec![];
    let tree = build_file_tree(tmp.path(), &infected_files);

    let autorun = tree.iter().find(|e| e.virtual_name == "autorun.inf").unwrap();
    assert!(autorun.is_autorun_inf);

    let setup = tree.iter().find(|e| e.virtual_name == "setup.exe").unwrap();
    assert!(setup.is_autorun_target);
}

#[test]
fn build_tree_marks_root_shell_scripts() {
    let tmp = tempfile::tempdir().unwrap();
    create_test_usb(tmp.path());

    let infected_files: Vec<String> = vec![];
    let tree = build_file_tree(tmp.path(), &infected_files);

    let sh = tree.iter().find(|e| e.virtual_name == "run.sh").unwrap();
    assert!(sh.is_root_shell_script);
}

#[test]
fn build_tree_includes_subdirectories() {
    let tmp = tempfile::tempdir().unwrap();
    create_test_usb(tmp.path());

    let infected_files: Vec<String> = vec![];
    let tree = build_file_tree(tmp.path(), &infected_files);

    let docs = tree.iter().find(|e| e.virtual_name == "docs").unwrap();
    assert!(docs.is_dir);
    assert_eq!(docs.children.len(), 1);
    assert_eq!(docs.children[0].virtual_name, "manual.pdf");
}

#[test]
fn build_tree_extracts_extensions() {
    let tmp = tempfile::tempdir().unwrap();
    create_test_usb(tmp.path());

    let infected_files: Vec<String> = vec![];
    let tree = build_file_tree(tmp.path(), &infected_files);

    let readme = tree.iter().find(|e| e.virtual_name == "readme.txt").unwrap();
    assert_eq!(readme.extension, "txt");

    let setup = tree.iter().find(|e| e.virtual_name == "setup.exe").unwrap();
    assert_eq!(setup.extension, "exe");
}

#[test]
fn build_tree_empty_directory() {
    let tmp = tempfile::tempdir().unwrap();
    let infected_files: Vec<String> = vec![];
    let tree = build_file_tree(tmp.path(), &infected_files);
    assert!(tree.is_empty());
}
