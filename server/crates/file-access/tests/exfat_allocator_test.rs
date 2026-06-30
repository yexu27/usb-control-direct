use std::path::PathBuf;

use file_access::exfat::allocator::ExfatAllocator;
use file_access::types::ControlledEntry;
use file_access::vfs::VfsIndex;

fn file(name: &str, size: u64) -> ControlledEntry {
    ControlledEntry {
        real_path: PathBuf::from(format!("/mnt/usb_raw/{name}")),
        virtual_name: name.to_string(),
        file_size: size,
        is_dir: false,
        is_virus: false,
        exec_type: None,
        extension: String::new(),
        is_autorun_target: false,
        is_autorun_inf: false,
        is_root_shell_script: false,
        children: vec![],
    }
}

#[test]
fn allocator_does_not_allocate_source_sized_image() {
    let source_size = 128_u64 * 1024 * 1024 * 1024;
    let index =
        VfsIndex::from_controlled_tree(&PathBuf::from("/mnt/usb_raw"), &[file("a.bin", 4096)])
            .unwrap();
    let allocator = ExfatAllocator::build(&index, source_size).unwrap();

    assert_eq!(allocator.total_sectors() * 512, source_size);
    assert!(allocator.estimated_memory_bytes() < 16 * 1024 * 1024);
}
