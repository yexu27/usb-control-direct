//! Resolve closed runtime exFAT write transactions into filesystem mutations.

use crate::exfat::directory_parser::parse_entry_sets;
use crate::exfat::runtime_state::ExfatRuntimeState;
use crate::exfat::sector_owner::SectorOwner;
use crate::exfat::transaction::{PendingTransaction, TransactionWrite};
use crate::exfat::layout::SECTOR_SIZE;
use crate::vfs::mutation::{ClusterChain, FileDataPatch, FsMutation};

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
        for write in tx.writes() {
            let TransactionWrite::Directory { owner, data, .. } = write else {
                continue;
            };
            let Some(parent) = state.parent_path_for_directory_owner(owner) else {
                continue;
            };
            for entry in parse_entry_sets(data)? {
                let virtual_path = join_virtual_path(&parent, &entry.name);
                if state.lookup_path(&virtual_path).is_some() {
                    continue;
                }
                let chain = if entry.first_cluster == 0 {
                    None
                } else {
                    Some(ClusterChain {
                        first_cluster: entry.first_cluster,
                        clusters: vec![entry.first_cluster],
                    })
                };
                if entry.is_dir {
                    mutations.push(FsMutation::CreateDir {
                        parent: parent.clone(),
                        name: entry.name,
                        chain,
                    });
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
