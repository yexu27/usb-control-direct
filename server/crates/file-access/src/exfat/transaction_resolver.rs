//! Resolve closed runtime exFAT write transactions into filesystem mutations.

use crate::exfat::directory_parser::parse_entry_sets;
use crate::exfat::runtime_state::ExfatRuntimeState;
use crate::exfat::sector_owner::SectorOwner;
use crate::exfat::transaction::{PendingTransaction, TransactionWrite};
use crate::exfat::layout::SECTOR_SIZE;
use crate::vfs::mutation::{ClusterChain, FileDataPatch, FsMutation, NodeKind};
use std::collections::{BTreeMap, HashSet};

#[derive(Debug, Default, Clone)]
pub struct TransactionResolver;

impl TransactionResolver {
    pub fn new() -> Self {
        Self
    }

    pub fn resolve_closed(
        &self,
        tx: &PendingTransaction,
        state: &ExfatRuntimeState,
    ) -> Result<Vec<FsMutation>, std::io::Error> {
        let mut mutations = Vec::new();
        let mut directory_writes = BTreeMap::<String, BTreeMap<u64, Vec<u8>>>::new();
        for write in tx.writes() {
            if let TransactionWrite::Directory {
                sector,
                owner,
                data,
            } = write
            {
                let Some(parent) = state.parent_path_for_directory_owner(owner) else {
                    continue;
                };
                directory_writes
                    .entry(parent)
                    .or_default()
                    .insert(*sector, data.clone());
            }
        }

        for (parent, sectors) in directory_writes {
            let Some((first_sector, mut data)) = state.directory_image(&parent)? else {
                continue;
            };
            for (sector, sector_data) in sectors {
                if sector < first_sector {
                    continue;
                }
                let offset = (sector - first_sector) as usize * SECTOR_SIZE as usize;
                if offset >= data.len() {
                    continue;
                }
                let end = (offset + SECTOR_SIZE as usize).min(data.len());
                data[offset..end].copy_from_slice(&sector_data[..end - offset]);
            }
            if data.is_empty() {
                continue;
            }
            let parsed_entries = parse_entry_sets(&data)?;
            let parsed_names = parsed_entries
                .iter()
                .map(|entry| entry.name.clone())
                .collect::<HashSet<_>>();
            let existing_children = state.immediate_children(&parent);
            let missing_children = existing_children
                .iter()
                .filter(|(name, _, _)| !parsed_names.contains(name))
                .cloned()
                .collect::<Vec<_>>();
            let new_entries = parsed_entries
                .iter()
                .filter(|entry| {
                    let virtual_path = join_virtual_path(&parent, &entry.name);
                    state.lookup_path(&virtual_path).is_none()
                })
                .cloned()
                .collect::<Vec<_>>();

            if missing_children.len() == 1
                && new_entries.len() == 1
                && is_rename_candidate(state, &missing_children[0], &new_entries[0])
            {
                let (_, from, kind) = &missing_children[0];
                let to = join_virtual_path(&parent, &new_entries[0].name);
                mutations.push(FsMutation::Rename {
                    from: from.clone(),
                    to: to.clone(),
                    kind: *kind,
                });
                if !new_entries[0].is_dir {
                    let chain = entry_chain(new_entries[0].first_cluster);
                    mutations.push(FsMutation::RewriteFile {
                        virtual_path: to,
                        size: new_entries[0].data_length,
                        valid_data_len: new_entries[0].valid_data_length,
                        chain,
                        data_patches: collect_data_patches(
                            tx,
                            &join_virtual_path(&parent, &new_entries[0].name),
                            new_entries[0].first_cluster,
                            new_entries[0].data_length,
                        ),
                    });
                }
                continue;
            }

            for (_, virtual_path, kind) in missing_children {
                mutations.push(FsMutation::Delete { virtual_path, kind });
            }
            for entry in parsed_entries {
                let virtual_path = join_virtual_path(&parent, &entry.name);
                if let Some(node) = state.lookup_path(&virtual_path) {
                    if !entry.is_dir {
                        let entry_first_cluster = if entry.first_cluster == 0 {
                            None
                        } else {
                            Some(entry.first_cluster)
                        };
                        if entry_first_cluster == node.first_cluster && entry.data_length != node.size
                        {
                            mutations.push(FsMutation::Truncate {
                                virtual_path,
                                len: entry.data_length,
                            });
                        } else if entry_first_cluster != node.first_cluster {
                            let chain = entry_first_cluster.map(|first_cluster| ClusterChain {
                                first_cluster,
                                clusters: vec![first_cluster],
                            });
                            let data_patches = collect_data_patches(
                                tx,
                                &virtual_path,
                                entry.first_cluster,
                                entry.data_length,
                            );
                            mutations.push(FsMutation::RewriteFile {
                                virtual_path,
                                size: entry.data_length,
                                valid_data_len: entry.valid_data_length,
                                chain,
                                data_patches,
                            });
                        }
                    }
                    continue;
                }
                let chain = entry_chain(entry.first_cluster);
                if entry.is_dir {
                    mutations.push(FsMutation::CreateDir {
                        parent: parent.clone(),
                        name: entry.name.clone(),
                        chain: chain.clone(),
                    });
                    if let Some(data) = free_cluster_data(tx, entry.first_cluster) {
                        resolve_new_directory_entries(
                            &virtual_path,
                            &data,
                            tx,
                            &mut mutations,
                        )?;
                    }
                } else {
                    let data_patches = collect_data_patches(
                        tx,
                        &virtual_path,
                        entry.first_cluster,
                        entry.data_length,
                    );
                    mutations.push(FsMutation::CreateFile {
                        parent: parent.clone(),
                        name: entry.name,
                        size: entry.data_length,
                        valid_data_len: entry.valid_data_length,
                        chain,
                        data_patches,
                    });
                }
            }
        }
        Ok(mutations)
    }
}

fn resolve_new_directory_entries(
    parent: &str,
    data: &[u8],
    tx: &PendingTransaction,
    mutations: &mut Vec<FsMutation>,
) -> Result<(), std::io::Error> {
    for entry in parse_entry_sets(data)? {
        let chain = entry_chain(entry.first_cluster);
        let virtual_path = join_virtual_path(parent, &entry.name);
        if entry.is_dir {
            mutations.push(FsMutation::CreateDir {
                parent: parent.to_string(),
                name: entry.name.clone(),
                chain: chain.clone(),
            });
            if let Some(data) = free_cluster_data(tx, entry.first_cluster) {
                resolve_new_directory_entries(&virtual_path, &data, tx, mutations)?;
            }
        } else {
            let data_patches =
                collect_data_patches(tx, &virtual_path, entry.first_cluster, entry.data_length);
            mutations.push(FsMutation::CreateFile {
                parent: parent.to_string(),
                name: entry.name,
                size: entry.data_length,
                valid_data_len: entry.valid_data_length,
                chain,
                data_patches,
            });
        }
    }
    Ok(())
}

fn is_rename_candidate(
    state: &ExfatRuntimeState,
    missing: &(String, String, NodeKind),
    new_entry: &crate::exfat::directory_parser::ParsedDirectoryEntry,
) -> bool {
    let (_, from, kind) = missing;
    if *kind != entry_kind(new_entry) {
        return false;
    }
    let Some(node) = state.lookup_path(from) else {
        return false;
    };
    node.first_cluster == entry_first_cluster(new_entry.first_cluster)
}

fn entry_kind(entry: &crate::exfat::directory_parser::ParsedDirectoryEntry) -> NodeKind {
    if entry.is_dir {
        NodeKind::Directory
    } else {
        NodeKind::File
    }
}

fn entry_first_cluster(first_cluster: u32) -> Option<u32> {
    if first_cluster == 0 {
        None
    } else {
        Some(first_cluster)
    }
}

fn entry_chain(first_cluster: u32) -> Option<ClusterChain> {
    entry_first_cluster(first_cluster).map(|first_cluster| ClusterChain {
        first_cluster,
        clusters: vec![first_cluster],
    })
}

fn free_cluster_data(tx: &PendingTransaction, cluster: u32) -> Option<Vec<u8>> {
    if cluster == 0 {
        return None;
    }
    let mut sectors = BTreeMap::<u64, Vec<u8>>::new();
    for write in tx.writes() {
        if let TransactionWrite::FreeCluster {
            sector,
            owner: SectorOwner::FreeCluster { cluster: write_cluster },
            data,
        } = write
        {
            if *write_cluster == cluster {
                sectors.insert(*sector, data.clone());
            }
        }
    }
    if sectors.is_empty() {
        return None;
    }
    let mut out = Vec::new();
    for data in sectors.values() {
        out.extend_from_slice(data);
    }
    Some(out)
}

fn collect_data_patches(
    tx: &PendingTransaction,
    virtual_path: &str,
    first_cluster: u32,
    data_length: u64,
) -> Vec<FileDataPatch> {
    if first_cluster == 0 || data_length == 0 {
        return Vec::new();
    }
    let mut out = Vec::new();
    for write in tx.writes() {
        match write {
            TransactionWrite::FreeCluster {
                owner: SectorOwner::FreeCluster { cluster },
                data,
                ..
            } if *cluster == first_cluster => {
                let take = (data_length as usize).min(data.len());
                out.push(FileDataPatch {
                    virtual_path: virtual_path.to_string(),
                    offset: 0,
                    data: data[..take].to_vec(),
                });
            }
            _ => {}
        }
    }
    if out.is_empty() {
        Vec::new()
    } else {
        let total = out.iter().map(|patch| patch.data.len()).sum::<usize>();
        if total > data_length as usize {
            out.truncate(data_length.div_ceil(SECTOR_SIZE as u64) as usize);
        }
        out
    }
}

fn join_virtual_path(parent: &str, name: &str) -> String {
    if parent == "/" {
        format!("/{name}")
    } else {
        format!("{parent}/{name}")
    }
}
