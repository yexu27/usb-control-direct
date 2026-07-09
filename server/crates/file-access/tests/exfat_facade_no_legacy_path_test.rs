use std::fs;
use std::path::Path;

fn source_file(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!("failed to read {}: {err}", path.display());
    })
}

fn function_body<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source
        .find(signature)
        .unwrap_or_else(|| panic!("missing function signature: {signature}"));
    let after_signature = &source[start..];
    let brace = after_signature
        .find('{')
        .unwrap_or_else(|| panic!("missing function body: {signature}"));
    let body_start = start + brace;
    let bytes = source.as_bytes();
    let mut depth = 0usize;
    for index in body_start..source.len() {
        match bytes[index] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return &source[body_start..=index];
                }
            }
            _ => {}
        }
    }
    panic!("unterminated function body: {signature}");
}

#[test]
fn virtual_exfat_facade_write_path_does_not_use_legacy_overlay_or_diff() {
    let fs_rs = source_file("src/exfat/fs.rs");
    let write_at = function_body(
        &fs_rs,
        "pub fn write_at(&self, offset: u64, data: &[u8]) -> Result<BlockWriteOutcome, std::io::Error>",
    );
    for forbidden in [
        "metadata_overlay",
        "data_overlay",
        "dirty_metadata_sectors",
        "commit_overlay_mutations",
        "diff_directory_snapshots",
        "WriteJournal",
    ] {
        assert!(
            !write_at.contains(forbidden),
            "VirtualExfatFs::write_at must not use legacy path marker `{forbidden}`"
        );
    }
    assert!(
        !write_at.contains("try_commit_closed_transaction"),
        "VirtualExfatFs::write_at must not route through removed low-level transaction API"
    );
    assert!(
        write_at.contains(".write_at(offset, data)"),
        "VirtualExfatFs::write_at must delegate to runtime block write path"
    );
}

#[test]
fn virtual_exfat_facade_flush_path_does_not_use_directory_snapshot_diff() {
    let fs_rs = source_file("src/exfat/fs.rs");
    let flush = function_body(&fs_rs, "pub fn flush(&self)");
    for forbidden in [
        "commit_overlay_mutations",
        "collect_dirty_directory_snapshots",
        "diff_directory_snapshots",
        "metadata_overlay",
        "data_overlay",
        "WriteJournal",
    ] {
        assert!(
            !flush.contains(forbidden),
            "VirtualExfatFs::flush must not use legacy path marker `{forbidden}`"
        );
    }
}

#[test]
fn production_exfat_module_does_not_import_directory_snapshot_diff() {
    let fs_rs = source_file("src/exfat/fs.rs");
    let mod_rs = source_file("src/exfat/mod.rs");
    assert!(
        !fs_rs.contains("crate::exfat::diff::diff_directory_snapshots"),
        "fs.rs must not import diff_directory_snapshots"
    );
    assert!(
        !mod_rs.contains("pub mod diff") && !mod_rs.contains("mod diff"),
        "production exfat module must not expose diff.rs"
    );
}

#[test]
fn virtual_exfat_facade_does_not_use_legacy_allocator() {
    let fs_rs = source_file("src/exfat/fs.rs");

    assert!(!fs_rs.contains("ExfatAllocator"));
    assert!(!fs_rs.contains("exfat::allocator"));
}
