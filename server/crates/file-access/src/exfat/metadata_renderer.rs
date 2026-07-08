//! Render committed exFAT metadata from authoritative runtime state.

use std::collections::BTreeMap;

use crate::exfat::dir_entry::build_file_entry_set;
use crate::exfat::layout::{
    DiskLayout, FAT_END_OF_CHAIN, FAT_ENTRY_SIZE, FAT_MEDIA_TYPE, FIRST_CLUSTER,
    PARTITION_OFFSET_SECTORS, SECTOR_SIZE,
};
use crate::exfat::metadata_state::ExfatMetadataState;
use crate::exfat::transaction::CommittedMetadataUpdate;
use crate::vfs::{VfsIndex, VfsNodeKind};

#[derive(Debug, Default, Clone)]
pub struct MetadataRenderer;

impl MetadataRenderer {
    pub fn render_directory(
        &self,
        path: &str,
        index: &VfsIndex,
        metadata: &ExfatMetadataState,
        layout: &DiskLayout,
    ) -> Result<Vec<CommittedMetadataUpdate>, std::io::Error> {
        let clusters = metadata.directory_clusters(path).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "directory metadata missing")
        })?;
        let node_id = index.lookup_path(path).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "directory node missing")
        })?;
        let node = index.node(node_id).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "directory node missing")
        })?;
        let mut directory_bytes = Vec::new();
        for child_id in &node.children {
            let child = index.node(*child_id).ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, "child node missing")
            })?;
            let is_dir = matches!(child.kind, VfsNodeKind::Directory);
            let data_length = if is_dir {
                metadata
                    .directory_clusters(&child.virtual_path)
                    .map(|clusters| clusters.len() as u64 * layout.cluster_size() as u64)
                    .unwrap_or(0)
            } else {
                child.size
            };
            directory_bytes.extend_from_slice(&build_file_entry_set(
                &child.name,
                is_dir,
                child.first_cluster.unwrap_or(0),
                data_length,
                child.is_blocked_placeholder(),
            ));
        }

        let total_len = clusters.len() * layout.cluster_size() as usize;
        directory_bytes.resize(total_len, 0);

        let mut updates = Vec::new();
        for (cluster_index, cluster) in clusters.iter().enumerate() {
            let cluster_start = cluster_index * layout.cluster_size() as usize;
            for sector_in_cluster in 0..layout.sectors_per_cluster() as usize {
                let byte_start = cluster_start + sector_in_cluster * SECTOR_SIZE as usize;
                let byte_end = byte_start + SECTOR_SIZE as usize;
                let sector = layout.cluster_to_sector(*cluster) + sector_in_cluster as u64;
                updates.push(CommittedMetadataUpdate::Sector {
                    sector,
                    data: directory_bytes[byte_start..byte_end].to_vec(),
                });
            }
        }
        Ok(updates)
    }

    pub fn render_fat_and_bitmap(
        &self,
        metadata: &ExfatMetadataState,
        layout: &DiskLayout,
    ) -> Vec<CommittedMetadataUpdate> {
        let mut updates = Vec::new();
        updates.extend(render_fat(metadata, layout));
        updates.extend(render_bitmap(metadata, layout));
        updates
    }

    pub fn merge_updates(
        &self,
        updates: Vec<CommittedMetadataUpdate>,
    ) -> Vec<CommittedMetadataUpdate> {
        let mut sectors = BTreeMap::<u64, Vec<u8>>::new();
        for update in updates {
            let CommittedMetadataUpdate::Sector { sector, data } = update;
            sectors.insert(sector, data);
        }
        sectors
            .into_iter()
            .map(|(sector, data)| CommittedMetadataUpdate::Sector { sector, data })
            .collect()
    }
}

fn render_fat(metadata: &ExfatMetadataState, layout: &DiskLayout) -> Vec<CommittedMetadataUpdate> {
    let size = layout.fat_length_sectors as usize * SECTOR_SIZE as usize;
    let mut data = vec![0u8; size];
    write_fat_entry(&mut data, 0, FAT_MEDIA_TYPE);
    write_fat_entry(&mut data, 1, FAT_END_OF_CHAIN);

    for cluster in FIRST_CLUSTER..FIRST_CLUSTER + metadata.fat_cluster_count() {
        if let Some(entry) = metadata.fat_entry_for(cluster) {
            write_fat_entry(&mut data, cluster, entry);
        }
    }

    let start_sector = PARTITION_OFFSET_SECTORS + layout.fat_offset_sectors;
    data.chunks(SECTOR_SIZE as usize)
        .enumerate()
        .map(|(index, chunk)| {
            let mut sector = vec![0u8; SECTOR_SIZE as usize];
            sector[..chunk.len()].copy_from_slice(chunk);
            CommittedMetadataUpdate::Sector {
                sector: start_sector + index as u64,
                data: sector,
            }
        })
        .collect()
}

fn write_fat_entry(data: &mut [u8], cluster: u32, entry: u32) {
    let offset = cluster as usize * FAT_ENTRY_SIZE as usize;
    if offset + FAT_ENTRY_SIZE as usize <= data.len() {
        data[offset..offset + FAT_ENTRY_SIZE as usize].copy_from_slice(&entry.to_le_bytes());
    }
}

fn render_bitmap(
    metadata: &ExfatMetadataState,
    layout: &DiskLayout,
) -> Vec<CommittedMetadataUpdate> {
    let bitmap_bytes = (metadata.bitmap_cluster_count() as usize).div_ceil(8);
    let bitmap_cluster_bytes = bitmap_bytes.div_ceil(layout.cluster_size() as usize).max(1)
        * layout.cluster_size() as usize;
    let mut data = vec![0u8; bitmap_cluster_bytes];

    for cluster in metadata.allocated_clusters() {
        if cluster < FIRST_CLUSTER {
            continue;
        }
        let bit = (cluster - FIRST_CLUSTER) as usize;
        let byte_index = bit / 8;
        let bit_index = bit % 8;
        if byte_index < data.len() {
            data[byte_index] |= 1 << bit_index;
        }
    }

    let bitmap_start_cluster = FIRST_CLUSTER + 1;
    data.chunks(SECTOR_SIZE as usize)
        .enumerate()
        .map(|(index, chunk)| {
            let mut sector = vec![0u8; SECTOR_SIZE as usize];
            sector[..chunk.len()].copy_from_slice(chunk);
            CommittedMetadataUpdate::Sector {
                sector: layout.cluster_to_sector(bitmap_start_cluster) + index as u64,
                data: sector,
            }
        })
        .collect()
}
