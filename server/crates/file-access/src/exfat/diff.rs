use std::collections::HashSet;

use crate::exfat::dir_snapshot::DirectorySnapshot;
use crate::exfat::directory_parser::ParsedDirectoryEntry;
use crate::exfat::layout::CLUSTER_SIZE;
use crate::vfs::mutation::{ClusterChain, FsMutation, NodeKind};

pub fn diff_directory_snapshots(
    old: &DirectorySnapshot,
    new: &DirectorySnapshot,
) -> Result<Vec<FsMutation>, std::io::Error> {
    let parent = old.virtual_path.clone();
    let mut mutations = Vec::new();
    let mut renamed_old_offsets = HashSet::new();
    let mut renamed_new_offsets = HashSet::new();

    for old_entry in old.entries() {
        if new.get_by_name(&old_entry.name).is_some() {
            continue;
        }
        for new_entry in new.entries() {
            if old.get_by_name(&new_entry.name).is_some() {
                continue;
            }
            if same_entry_identity(old_entry, new_entry) {
                let kind = if old_entry.is_dir {
                    NodeKind::Directory
                } else {
                    NodeKind::File
                };
                mutations.push(FsMutation::Rename {
                    from: join_virtual_path(&parent, &old_entry.name),
                    to: join_virtual_path(&parent, &new_entry.name),
                    kind,
                });
                if !new_entry.is_dir && old_entry.data_length != new_entry.data_length {
                    mutations.push(FsMutation::Truncate {
                        virtual_path: join_virtual_path(&parent, &new_entry.name),
                        len: new_entry.data_length,
                    });
                }
                renamed_old_offsets.insert(old_entry.entry_offset);
                renamed_new_offsets.insert(new_entry.entry_offset);
                break;
            }
        }
    }

    for new_entry in new.entries() {
        if let Some(old_entry) = old.get_by_name(&new_entry.name) {
            if !new_entry.is_dir && !old_entry.is_dir {
                let virtual_path = join_virtual_path(&parent, &new_entry.name);
                if old_entry.first_cluster != new_entry.first_cluster {
                    mutations.push(FsMutation::RewriteFile {
                        virtual_path,
                        size: new_entry.data_length,
                        valid_data_len: new_entry.valid_data_length,
                        chain: cluster_chain_from_entry(new_entry),
                        data_patches: Vec::new(),
                    });
                } else if old_entry.data_length != new_entry.data_length {
                    mutations.push(FsMutation::Truncate {
                        virtual_path,
                        len: new_entry.data_length,
                    });
                }
            }
        }
        if renamed_new_offsets.contains(&new_entry.entry_offset) {
            continue;
        }
        if old.get_by_name(&new_entry.name).is_none() {
            if new_entry.is_dir {
                mutations.push(FsMutation::CreateDir {
                    parent: parent.clone(),
                    name: new_entry.name.clone(),
                    chain: cluster_chain_from_entry(new_entry),
                });
            } else {
                mutations.push(FsMutation::CreateFile {
                    parent: parent.clone(),
                    name: new_entry.name.clone(),
                    size: new_entry.data_length,
                    valid_data_len: new_entry.valid_data_length,
                    chain: cluster_chain_from_entry(new_entry),
                    data_patches: Vec::new(),
                });
            }
        }
    }

    for old_entry in old.entries() {
        if renamed_old_offsets.contains(&old_entry.entry_offset) {
            continue;
        }
        if new.get_by_name(&old_entry.name).is_none() {
            let kind = if old_entry.is_dir {
                NodeKind::Directory
            } else {
                NodeKind::File
            };
            mutations.push(FsMutation::Delete {
                virtual_path: join_virtual_path(&parent, &old_entry.name),
                kind,
            });
        }
    }

    Ok(mutations)
}

fn same_entry_identity(
    old_entry: &ParsedDirectoryEntry,
    new_entry: &ParsedDirectoryEntry,
) -> bool {
    old_entry.is_dir == new_entry.is_dir
        && old_entry.entry_offset == new_entry.entry_offset
        && old_entry.first_cluster != 0
        && old_entry.first_cluster == new_entry.first_cluster
}

fn cluster_chain_from_entry(entry: &ParsedDirectoryEntry) -> Option<ClusterChain> {
    if entry.first_cluster == 0 {
        None
    } else {
        let cluster_count = entry
            .data_length
            .div_ceil(CLUSTER_SIZE as u64)
            .max(1) as u32;
        Some(ClusterChain {
            first_cluster: entry.first_cluster,
            clusters: (0..cluster_count)
                .map(|offset| entry.first_cluster + offset)
                .collect(),
        })
    }
}

fn join_virtual_path(parent: &str, name: &str) -> String {
    if parent == "/" {
        format!("/{}", name)
    } else {
        format!("{}/{}", parent, name)
    }
}
