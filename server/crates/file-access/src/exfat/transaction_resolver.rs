//! Resolve closed runtime exFAT write transactions into filesystem mutations.

use crate::exfat::directory_parser::parse_entry_sets;
use crate::exfat::layout::SECTOR_SIZE;
use crate::exfat::runtime_state::ExfatRuntimeState;
use crate::exfat::sector_owner::SectorOwner;
use crate::exfat::transaction::{
    PendingReason, PendingTransaction, ResolveStatus, ResolvedTransaction, TransactionError,
    TransactionWrite,
};
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
    ) -> Result<ResolveStatus, std::io::Error> {
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
                    return Ok(ResolveStatus::Invalid(
                        TransactionError::UnknownDirectoryOwner { sector: *sector },
                    ));
                };
                directory_writes
                    .entry(parent)
                    .or_default()
                    .insert(*sector, data.clone());
            }
        }

        for (parent, sectors) in directory_writes {
            let Some((first_sector, mut data)) = state.directory_image(&parent)? else {
                return Ok(ResolveStatus::Invalid(
                    TransactionError::MissingDirectoryImage { parent },
                ));
            };
            for (sector, sector_data) in sectors {
                if sector < first_sector {
                    return Ok(ResolveStatus::Invalid(
                        TransactionError::DirectoryWriteBeforeStart { parent, sector },
                    ));
                }
                let offset = (sector - first_sector) as usize * SECTOR_SIZE as usize;
                if offset >= data.len() {
                    return Ok(ResolveStatus::Incomplete(
                        PendingReason::WaitingForDirectoryData { sector },
                    ));
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

            let mut skipped_blocked_paths = HashSet::new();
            let mut skipped_cached_names = HashSet::new();
            for missing in &missing_children {
                let (_, virtual_path, _) = missing;
                if !is_blocked_placeholder_path(state, virtual_path) {
                    continue;
                }
                let Some(cached_entry) = new_entries
                    .iter()
                    .find(|entry| is_cached_blocked_rename_entry(state, missing, entry))
                else {
                    continue;
                };
                tracing::warn!(
                    from = %virtual_path,
                    cached_name = %cached_entry.name,
                    "忽略 Windows 缓存的 blocked placeholder 重命名目录项，继续处理同事务内其他写入"
                );
                skipped_blocked_paths.insert(virtual_path.clone());
                skipped_cached_names.insert(cached_entry.name.clone());
            }

            let effective_missing_children = missing_children
                .iter()
                .filter(|(_, virtual_path, _)| !skipped_blocked_paths.contains(virtual_path))
                .cloned()
                .collect::<Vec<_>>();
            let effective_new_entries = new_entries
                .iter()
                .filter(|entry| !skipped_cached_names.contains(&entry.name))
                .cloned()
                .collect::<Vec<_>>();
            let effective_parsed_entries = parsed_entries
                .iter()
                .filter(|entry| !skipped_cached_names.contains(&entry.name))
                .cloned()
                .collect::<Vec<_>>();

            if effective_missing_children.len() == 1
                && effective_new_entries.len() == 1
                && is_rename_candidate(
                    state,
                    &effective_missing_children[0],
                    &effective_new_entries[0],
                )
            {
                let (_, from, kind) = &effective_missing_children[0];
                let to = join_virtual_path(&parent, &effective_new_entries[0].name);
                mutations.push(FsMutation::Rename {
                    from: from.clone(),
                    to: to.clone(),
                    kind: *kind,
                });
                if !effective_new_entries[0].is_dir {
                    let chain = match resolve_entry_chain(
                        state,
                        effective_new_entries[0].first_cluster,
                        effective_new_entries[0].data_length,
                    ) {
                        Ok(chain) => chain,
                        Err(err) => return Ok(ResolveStatus::Invalid(err)),
                    };
                    mutations.push(FsMutation::RewriteFile {
                        virtual_path: to,
                        size: effective_new_entries[0].data_length,
                        valid_data_len: effective_new_entries[0].valid_data_length,
                        chain,
                        data_patches: collect_data_patches(
                            tx,
                            &join_virtual_path(&parent, &effective_new_entries[0].name),
                            effective_new_entries[0].first_cluster,
                            effective_new_entries[0].data_length,
                        ),
                    });
                }
                continue;
            }

            if !skipped_blocked_paths.is_empty()
                && effective_missing_children.is_empty()
                && effective_new_entries.is_empty()
                && !effective_parsed_entries
                    .iter()
                    .any(|entry| existing_entry_has_metadata_change(state, &parent, entry))
            {
                let virtual_path = skipped_blocked_paths
                    .iter()
                    .next()
                    .expect("skipped blocked path exists")
                    .clone();
                return Ok(ResolveStatus::Invalid(
                    TransactionError::BlockedPlaceholderRewrite { virtual_path },
                ));
            }

            if !effective_new_entries.is_empty() {
                for (_, virtual_path, _) in &effective_missing_children {
                    if is_blocked_placeholder_path(state, virtual_path) {
                        return Ok(ResolveStatus::Invalid(
                            TransactionError::BlockedPlaceholderRewrite {
                                virtual_path: virtual_path.clone(),
                            },
                        ));
                    }
                }
            }

            for (_, virtual_path, kind) in effective_missing_children {
                mutations.push(FsMutation::Delete { virtual_path, kind });
            }
            for entry in effective_parsed_entries {
                let virtual_path = join_virtual_path(&parent, &entry.name);
                if let Some(node) = state.lookup_path(&virtual_path) {
                    if entry.is_dir {
                        if entry_first_cluster(entry.first_cluster) != node.first_cluster {
                            return Ok(ResolveStatus::Invalid(
                                TransactionError::UnsupportedDirectoryRewrite {
                                    parent: parent.clone(),
                                },
                            ));
                        }
                        continue;
                    }
                    if !entry.is_dir {
                        let entry_first_cluster = if entry.first_cluster == 0 {
                            None
                        } else {
                            Some(entry.first_cluster)
                        };
                        if entry_first_cluster == node.first_cluster
                            && entry.data_length != node.size
                        {
                            mutations.push(FsMutation::Truncate {
                                virtual_path,
                                len: entry.data_length,
                            });
                        } else if entry_first_cluster != node.first_cluster {
                            let chain = match resolve_entry_chain(
                                state,
                                entry.first_cluster,
                                entry.data_length,
                            ) {
                                Ok(chain) => chain,
                                Err(err) => return Ok(ResolveStatus::Invalid(err)),
                            };
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
                let chain = match resolve_entry_chain(state, entry.first_cluster, entry.data_length)
                {
                    Ok(chain) => chain,
                    Err(err) => return Ok(ResolveStatus::Invalid(err)),
                };
                if entry.is_dir {
                    mutations.push(FsMutation::CreateDir {
                        parent: parent.clone(),
                        name: entry.name.clone(),
                        chain: chain.clone(),
                    });
                    if let Some(data) = free_cluster_data(tx, entry.first_cluster) {
                        match resolve_new_directory_entries(
                            state,
                            &virtual_path,
                            &data,
                            tx,
                            &mut mutations,
                        ) {
                            Ok(()) => {}
                            Err(err) => return Ok(ResolveStatus::Invalid(err)),
                        }
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
        Ok(ResolveStatus::Complete(ResolvedTransaction { mutations }))
    }
}

fn resolve_new_directory_entries(
    state: &ExfatRuntimeState,
    parent: &str,
    data: &[u8],
    tx: &PendingTransaction,
    mutations: &mut Vec<FsMutation>,
) -> Result<(), TransactionError> {
    for entry in
        parse_entry_sets(data).map_err(|_| TransactionError::UnsupportedDirectoryRewrite {
            parent: parent.to_string(),
        })?
    {
        let chain = resolve_entry_chain(state, entry.first_cluster, entry.data_length)?;
        let virtual_path = join_virtual_path(parent, &entry.name);
        if entry.is_dir {
            mutations.push(FsMutation::CreateDir {
                parent: parent.to_string(),
                name: entry.name.clone(),
                chain: chain.clone(),
            });
            if let Some(data) = free_cluster_data(tx, entry.first_cluster) {
                resolve_new_directory_entries(state, &virtual_path, &data, tx, mutations)?;
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

fn is_blocked_placeholder_path(state: &ExfatRuntimeState, virtual_path: &str) -> bool {
    state
        .lookup_path(virtual_path)
        .map(|node| node.is_blocked_placeholder())
        .unwrap_or(false)
}

fn is_cached_blocked_rename_entry(
    state: &ExfatRuntimeState,
    missing: &(String, String, NodeKind),
    new_entry: &crate::exfat::directory_parser::ParsedDirectoryEntry,
) -> bool {
    if !is_rename_candidate(state, missing, new_entry) {
        return false;
    }
    let (_, virtual_path, _) = missing;
    let Some(node) = state.lookup_path(virtual_path) else {
        return false;
    };
    node.is_blocked_placeholder() && node.size == new_entry.data_length
}

fn existing_entry_has_metadata_change(
    state: &ExfatRuntimeState,
    parent: &str,
    entry: &crate::exfat::directory_parser::ParsedDirectoryEntry,
) -> bool {
    let virtual_path = join_virtual_path(parent, &entry.name);
    let Some(node) = state.lookup_path(&virtual_path) else {
        return false;
    };
    if entry.is_dir {
        return entry_first_cluster(entry.first_cluster) != node.first_cluster;
    }
    let entry_first_cluster = entry_first_cluster(entry.first_cluster);
    entry_first_cluster != node.first_cluster || entry.data_length != node.size
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

fn resolve_entry_chain(
    state: &ExfatRuntimeState,
    first_cluster: u32,
    data_length: u64,
) -> Result<Option<ClusterChain>, TransactionError> {
    let Some(first_cluster) = entry_first_cluster(first_cluster) else {
        return Ok(None);
    };
    let cluster_size = state.cluster_size() as u64;
    let required_clusters = data_length.div_ceil(cluster_size).max(1) as usize;
    if required_clusters == 1 {
        return Ok(Some(ClusterChain {
            first_cluster,
            clusters: vec![first_cluster],
        }));
    }
    let chain = state.metadata_chain_from(first_cluster).map_err(|_| {
        TransactionError::UnresolvedClusterChain {
            first_cluster,
            data_length,
        }
    })?;
    if chain.len() < required_clusters {
        return Err(TransactionError::UnresolvedClusterChain {
            first_cluster,
            data_length,
        });
    }
    Ok(Some(ClusterChain {
        first_cluster,
        clusters: chain.into_iter().take(required_clusters).collect(),
    }))
}

fn free_cluster_data(tx: &PendingTransaction, cluster: u32) -> Option<Vec<u8>> {
    if cluster == 0 {
        return None;
    }
    let mut sectors = BTreeMap::<u64, Vec<u8>>::new();
    for write in tx.writes() {
        if let TransactionWrite::FreeCluster {
            sector,
            owner: SectorOwner::FreeCluster {
                cluster: write_cluster,
            },
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
