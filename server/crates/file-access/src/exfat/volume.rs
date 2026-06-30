//! 虚拟 exFAT 卷 — 扇区级读写接口。
//!
//! 整合 boot / FAT / bitmap / upcase / dir_entry，提供按扇区号读取内容的接口。
//! 元数据扇区预生成在内存中，文件数据扇区映射到真实文件路径。

use std::collections::HashMap;
use std::path::PathBuf;

use crate::exfat::bitmap::generate_bitmap;
use crate::exfat::boot::{generate_boot_region, generate_mbr};
use crate::exfat::dir_entry::*;
use crate::exfat::fat::FatBuilder;
use crate::exfat::layout::*;
use crate::exfat::upcase::generate_upcase_table;
use crate::policy::evaluate_access;
use crate::types::{AccessDecision, ControlledEntry, PolicySnapshot, SectorContent};

/// 虚拟 exFAT 卷。
pub struct VirtualVolume {
    /// 预生成的元数据扇区（扇区号 → 数据）。
    metadata_sectors: HashMap<u64, Vec<u8>>,
    /// 文件数据扇区映射（扇区号 → FileData 信息）。
    file_data_sectors: HashMap<u64, FileDataMapping>,
    /// 磁盘布局。
    layout: DiskLayout,
    /// 文件名到其数据扇区范围的映射（用于测试辅助）。
    file_sector_map: HashMap<String, Vec<u64>>,
}

/// 文件数据映射。
#[derive(Debug, Clone)]
struct FileDataMapping {
    real_path: PathBuf,
    offset: u64,
    valid_bytes: u32,
    blocked: bool,
}

impl VirtualVolume {
    /// 构建虚拟 exFAT 卷。
    ///
    /// 参数:
    ///   - tree: 受控文件树。
    ///   - snapshot: 策略快照。
    pub fn build(tree: &[ControlledEntry], snapshot: &PolicySnapshot) -> Self {
        Self::build_with_capacity(tree, snapshot, 0)
    }

    /// 使用指定容量目标构建虚拟 exFAT 卷。
    pub fn build_with_capacity(
        tree: &[ControlledEntry],
        snapshot: &PolicySnapshot,
        source_size_bytes: u64,
    ) -> Self {
        let mut builder = VolumeBuilder::new(snapshot, source_size_bytes);
        builder.allocate_metadata();
        builder.allocate_files(tree, &[]);
        builder.generate()
    }

    /// 读取指定扇区。
    pub fn read_sector(&self, sector: u64) -> SectorContent {
        if sector >= self.layout.total_sectors {
            return SectorContent::Zero;
        }

        if let Some(data) = self.metadata_sectors.get(&sector) {
            return SectorContent::Metadata(data.clone());
        }

        if let Some(mapping) = self.file_data_sectors.get(&sector) {
            return SectorContent::FileData {
                real_path: mapping.real_path.clone(),
                offset: mapping.offset,
                valid_bytes: mapping.valid_bytes,
                blocked: mapping.blocked,
            };
        }

        SectorContent::Zero
    }

    /// 总扇区数。
    pub fn total_sectors(&self) -> u64 {
        self.layout.total_sectors
    }

    /// 磁盘布局引用。
    pub fn layout(&self) -> &DiskLayout {
        &self.layout
    }

    /// 查找指定文件名的数据扇区列表（测试辅助）。
    pub fn find_file_data_sectors(&self, name: &str) -> Vec<u64> {
        self.file_sector_map.get(name).cloned().unwrap_or_default()
    }

    /// 判断扇区是否为目录数据扇区。
    pub fn is_directory_sector(&self, sector: u64) -> bool {
        self.metadata_sectors.contains_key(&sector)
            && self.layout.sector_to_cluster(sector).is_some()
    }
}

/// 卷构建器。
struct VolumeBuilder<'a> {
    snapshot: &'a PolicySnapshot,
    /// 虚拟块设备对外暴露的最小总容量。
    min_total_bytes: u64,
    /// 下一个可分配的簇号。
    next_cluster: u32,
    /// 簇分配记录。
    cluster_allocations: Vec<ClusterAllocation>,
    /// 根目录项数据。
    root_dir_entries: Vec<u8>,
    /// 文件数据映射。
    file_mappings: Vec<FileMapping>,
    /// 文件扇区映射（名称 → 扇区列表）。
    #[allow(dead_code)]
    file_sector_map: HashMap<String, Vec<u64>>,
}

/// 簇分配类型。
#[derive(Debug)]
enum ClusterAllocation {
    /// 元数据（bitmap / upcase / 目录）。
    Metadata { start: u32, count: u32, data: Vec<u8> },
    /// 文件数据。
    #[allow(dead_code)]
    FileData { start: u32, count: u32, name: String },
}

/// 文件映射。
#[derive(Debug)]
struct FileMapping {
    name: String,
    real_path: PathBuf,
    start_cluster: u32,
    file_size: u64,
    blocked: bool,
}

/// 计算文件所需簇数（安全处理大文件）。
fn file_clusters(file_size: u64) -> u32 {
    let clusters = file_size.div_ceil(CLUSTER_SIZE as u64);
    clusters.min(u32::MAX as u64) as u32
}

impl<'a> VolumeBuilder<'a> {
    fn new(snapshot: &'a PolicySnapshot, source_size_bytes: u64) -> Self {
        VolumeBuilder {
            snapshot,
            min_total_bytes: source_size_bytes.max(MIN_VIRTUAL_VOLUME_BYTES),
            next_cluster: FIRST_CLUSTER,
            cluster_allocations: Vec::new(),
            root_dir_entries: Vec::new(),
            file_mappings: Vec::new(),
            file_sector_map: HashMap::new(),
        }
    }

    /// 分配一个或多个连续簇。
    fn allocate_clusters(&mut self, count: u32) -> u32 {
        let start = self.next_cluster;
        self.next_cluster += count;
        start
    }

    /// 分配元数据区域（root dir / bitmap / upcase）。
    fn allocate_metadata(&mut self) {
        // Cluster 2: Root Directory（初始 1 簇，后续可扩展）
        let root_cluster = self.allocate_clusters(1);
        assert_eq!(root_cluster, 2);

        // Cluster 3: Allocation Bitmap（1 簇）
        let bitmap_cluster = self.allocate_clusters(1);

        // Cluster 4+: Upcase Table
        let (upcase_data, upcase_checksum) = generate_upcase_table();
        let upcase_clusters = (upcase_data.len() as u32).div_ceil(CLUSTER_SIZE);
        let upcase_cluster = self.allocate_clusters(upcase_clusters);

        // 根目录项：Volume Label + Bitmap + Upcase
        self.root_dir_entries.extend(build_volume_label_entry("USB_CTRL"));
        self.root_dir_entries.extend(build_bitmap_entry(bitmap_cluster, 0)); // 大小后面更新
        self.root_dir_entries.extend(build_upcase_entry(
            upcase_cluster,
            upcase_data.len() as u64,
            upcase_checksum,
        ));

        // 保存 upcase 分配
        self.cluster_allocations.push(ClusterAllocation::Metadata {
            start: upcase_cluster,
            count: upcase_clusters,
            data: upcase_data,
        });
    }

    /// 分配文件和子目录。
    fn allocate_files(&mut self, entries: &[ControlledEntry], _parent_path: &[String]) {
        for entry in entries {
            let decision = evaluate_access(entry, self.snapshot);
            let blocked = matches!(decision, AccessDecision::Deny(_));

            if entry.is_dir {
                // 目录：按实际子目录项大小动态分配簇数
                let mut child_dir_data = Vec::new();
                if !entry.children.is_empty() {
                    for child in &entry.children {
                        let child_decision = evaluate_access(child, self.snapshot);
                        let child_blocked = matches!(child_decision, AccessDecision::Deny(_));

                        if child.is_dir {
                            // 子目录的簇号会在递归中分配
                            let child_cluster = self.allocate_clusters(1);
                            let child_entry = build_file_entry_set(
                                &child.virtual_name,
                                true,
                                child_cluster,
                                0,
                                false,
                            );
                            child_dir_data.extend(child_entry);

                            self.cluster_allocations.push(ClusterAllocation::Metadata {
                                start: child_cluster,
                                count: 1,
                                data: vec![0u8; CLUSTER_SIZE as usize],
                            });
                        } else {
                            let (file_cluster, file_clusters) = if child.is_virus || child.file_size == 0 {
                                (0, 0)
                            } else {
                                let clusters = file_clusters(child.file_size);
                                let start = self.allocate_clusters(clusters);
                                (start, clusters)
                            };

                            let child_entry = build_file_entry_set(
                                &child.virtual_name,
                                false,
                                file_cluster,
                                child.file_size,
                                child.is_virus,
                            );
                            child_dir_data.extend(child_entry);

                            if file_clusters > 0 {
                                self.file_mappings.push(FileMapping {
                                    name: child.virtual_name.clone(),
                                    real_path: child.real_path.clone(),
                                    start_cluster: file_cluster,
                                    file_size: child.file_size,
                                    blocked: child_blocked,
                                });
                            }
                        }
                    }
                }

                // 按实际子目录项大小动态分配簇
                let dir_clusters_needed = if child_dir_data.is_empty() {
                    1
                } else {
                    (child_dir_data.len() as u32).div_ceil(CLUSTER_SIZE)
                };
                let dir_cluster = self.allocate_clusters(dir_clusters_needed);

                // 生成目录项（放入父目录，即 root_dir_entries）
                let dir_entry_data = build_file_entry_set(
                    &entry.virtual_name,
                    true,
                    dir_cluster,
                    0,
                    false,
                );
                self.root_dir_entries.extend(dir_entry_data);

                // 填充到簇对齐大小
                child_dir_data.resize((dir_clusters_needed as usize) * CLUSTER_SIZE as usize, 0);
                self.cluster_allocations.push(ClusterAllocation::Metadata {
                    start: dir_cluster,
                    count: dir_clusters_needed,
                    data: child_dir_data,
                });
            } else {
                // 文件
                let (file_cluster, file_clusters) = if entry.is_virus || entry.file_size == 0 {
                    (0, 0)
                } else {
                    let clusters = file_clusters(entry.file_size);
                    let start = self.allocate_clusters(clusters);
                    (start, clusters)
                };

                let file_entry = build_file_entry_set(
                    &entry.virtual_name,
                    false,
                    file_cluster,
                    entry.file_size,
                    entry.is_virus,
                );
                self.root_dir_entries.extend(file_entry);

                if file_clusters > 0 {
                    self.file_mappings.push(FileMapping {
                        name: entry.virtual_name.clone(),
                        real_path: entry.real_path.clone(),
                        start_cluster: file_cluster,
                        file_size: entry.file_size,
                        blocked,
                    });
                }
            }
        }
    }

    /// 生成最终的虚拟卷。
    fn generate(mut self) -> VirtualVolume {
        // 根目录可能需要多个簇 — 在最终生成阶段分配额外簇
        let root_data_len = self.root_dir_entries.len().max(CLUSTER_SIZE as usize);
        let root_clusters_needed = (root_data_len as u32).div_ceil(CLUSTER_SIZE);
        let root_extra_clusters = root_clusters_needed - 1; // cluster 2 已在 allocate_metadata 分配

        // 为根目录分配额外簇（不需要连续，FAT 链处理）
        let root_extra_start = if root_extra_clusters > 0 {
            Some(self.allocate_clusters(root_extra_clusters))
        } else {
            None
        };

        let allocated_clusters = self.next_cluster - FIRST_CLUSTER;
        let layout = DiskLayout::new_with_min_total_bytes(allocated_clusters, self.min_total_bytes);
        let total_clusters = layout.cluster_count;

        let mut metadata_sectors = HashMap::new();
        let mut file_data_sectors = HashMap::new();

        // MBR
        let mbr = generate_mbr(&layout);
        metadata_sectors.insert(0, mbr);

        // Boot Region (Main)
        let boot_region = generate_boot_region(&layout);
        for i in 0..12 {
            let sector = PARTITION_OFFSET_SECTORS + i;
            let offset = (i as usize) * SECTOR_SIZE as usize;
            let data = boot_region[offset..offset + SECTOR_SIZE as usize].to_vec();
            metadata_sectors.insert(sector, data);
        }

        // Boot Region (Backup)
        for i in 0..12 {
            let sector = PARTITION_OFFSET_SECTORS + BOOT_REGION_SECTORS + i;
            let offset = (i as usize) * SECTOR_SIZE as usize;
            let data = boot_region[offset..offset + SECTOR_SIZE as usize].to_vec();
            metadata_sectors.insert(sector, data);
        }

        // FAT
        let mut fat_builder = FatBuilder::new(total_clusters);
        // Root directory: cluster 2, possibly chained
        if root_clusters_needed == 1 {
            fat_builder.set_single(2);
        } else {
            // cluster 2 -> root_extra_start -> root_extra_start+1 -> ... -> EOF
            fat_builder.set_chain_from_parts(2, root_extra_start.unwrap(), root_extra_clusters);
        }
        // Bitmap: cluster 3, single
        fat_builder.set_single(3);

        for alloc in &self.cluster_allocations {
            match alloc {
                ClusterAllocation::Metadata { start, count, .. } => {
                    if *count == 1 {
                        fat_builder.set_single(*start);
                    } else {
                        fat_builder.set_chain(*start, *count);
                    }
                }
                ClusterAllocation::FileData { start, count, .. } => {
                    if *count == 1 {
                        fat_builder.set_single(*start);
                    } else {
                        fat_builder.set_chain(*start, *count);
                    }
                }
            }
        }

        for mapping in &self.file_mappings {
            let clusters = file_clusters(mapping.file_size);
            if clusters == 1 {
                fat_builder.set_single(mapping.start_cluster);
            } else {
                fat_builder.set_chain(mapping.start_cluster, clusters);
            }
        }

        let fat_data = fat_builder.build(layout.fat_length_sectors);
        let fat_start_sector = PARTITION_OFFSET_SECTORS + layout.fat_offset_sectors;
        for i in 0..layout.fat_length_sectors {
            let offset = (i as usize) * SECTOR_SIZE as usize;
            let end = (offset + SECTOR_SIZE as usize).min(fat_data.len());
            let mut sector_data = vec![0u8; SECTOR_SIZE as usize];
            let copy_len = end - offset;
            sector_data[..copy_len].copy_from_slice(&fat_data[offset..end]);
            metadata_sectors.insert(fat_start_sector + i, sector_data);
        }

        // Bitmap
        let bitmap_data = generate_bitmap(total_clusters, allocated_clusters);

        // 更新根目录中 Bitmap 条目的 DataLength
        let bitmap_entry_offset = DIR_ENTRY_SIZE as usize;
        if self.root_dir_entries.len() >= bitmap_entry_offset + DIR_ENTRY_SIZE as usize {
            let len_offset = bitmap_entry_offset + 24;
            self.root_dir_entries[len_offset..len_offset + 8]
                .copy_from_slice(&(bitmap_data.len() as u64).to_le_bytes());
        }

        let bitmap_sector_start = layout.cluster_to_sector(3);
        for i in 0..(CLUSTER_SIZE as usize / SECTOR_SIZE as usize) {
            let offset = i * SECTOR_SIZE as usize;
            let mut sector_data = vec![0u8; SECTOR_SIZE as usize];
            let end = (offset + SECTOR_SIZE as usize).min(bitmap_data.len());
            if offset < bitmap_data.len() {
                sector_data[..end - offset].copy_from_slice(&bitmap_data[offset..end]);
            }
            metadata_sectors.insert(bitmap_sector_start + i as u64, sector_data);
        }

        // Root Directory — 按实际大小写入多簇
        let mut root_data = self.root_dir_entries;
        root_data.resize((root_clusters_needed as usize) * CLUSTER_SIZE as usize, 0);

        // 写入 cluster 2（第一个簇）
        let root_sector_start = layout.cluster_to_sector(2);
        for i in 0..(CLUSTER_SIZE as usize / SECTOR_SIZE as usize) {
            let offset = i * SECTOR_SIZE as usize;
            let data = root_data[offset..offset + SECTOR_SIZE as usize].to_vec();
            metadata_sectors.insert(root_sector_start + i as u64, data);
        }

        // 写入额外簇
        if let Some(extra_start) = root_extra_start {
            for c in 0..root_extra_clusters {
                let cluster = extra_start + c;
                let cluster_sector_start = layout.cluster_to_sector(cluster);
                let cluster_data_offset = ((c + 1) as usize) * CLUSTER_SIZE as usize;
                for i in 0..(CLUSTER_SIZE as usize / SECTOR_SIZE as usize) {
                    let offset = cluster_data_offset + i * SECTOR_SIZE as usize;
                    let data = root_data[offset..offset + SECTOR_SIZE as usize].to_vec();
                    metadata_sectors.insert(cluster_sector_start + i as u64, data);
                }
            }
        }

        // 元数据簇（upcase / 子目录）
        for alloc in &self.cluster_allocations {
            if let ClusterAllocation::Metadata { start, count, data } = alloc {
                let cluster_start_sector = layout.cluster_to_sector(*start);
                let total_sectors_for_alloc = *count as u64 * SECTORS_PER_CLUSTER as u64;
                for i in 0..total_sectors_for_alloc {
                    let offset = (i as usize) * SECTOR_SIZE as usize;
                    let mut sector_data = vec![0u8; SECTOR_SIZE as usize];
                    let end = (offset + SECTOR_SIZE as usize).min(data.len());
                    if offset < data.len() {
                        sector_data[..end - offset].copy_from_slice(&data[offset..end]);
                    }
                    metadata_sectors.insert(cluster_start_sector + i, sector_data);
                }
            }
        }

        // 文件数据映射
        let mut file_sector_map = HashMap::new();
        for mapping in &self.file_mappings {
            let cluster_start_sector = layout.cluster_to_sector(mapping.start_cluster);
            let total_data_sectors =
                mapping.file_size.div_ceil(SECTOR_SIZE as u64);
            let mut sectors = Vec::new();

            for i in 0..total_data_sectors {
                let sector = cluster_start_sector + i;
                let offset = i * SECTOR_SIZE as u64;
                let remaining = mapping.file_size - offset;
                let valid_bytes = if remaining >= SECTOR_SIZE as u64 {
                    SECTOR_SIZE
                } else {
                    remaining as u32
                };

                file_data_sectors.insert(
                    sector,
                    FileDataMapping {
                        real_path: mapping.real_path.clone(),
                        offset,
                        valid_bytes,
                        blocked: mapping.blocked,
                    },
                );
                sectors.push(sector);
            }

            file_sector_map.insert(mapping.name.clone(), sectors);
        }

        VirtualVolume {
            metadata_sectors,
            file_data_sectors,
            layout,
            file_sector_map,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::types::{ControlledEntry, ExecFileType, PolicySnapshot, SectorContent};

    #[test]
    fn blocked_file_data_sector_is_marked_blocked() {
        let dir = tempfile::tempdir().unwrap();
        let real = dir.path().join("tool.bin");
        std::fs::write(&real, vec![1u8; 512]).unwrap();
        let entry = ControlledEntry {
            real_path: real,
            virtual_name: "tool.bin".to_string(),
            file_size: 512,
            is_dir: false,
            is_virus: false,
            exec_type: Some(ExecFileType::Elf),
            extension: "bin".to_string(),
            is_autorun_target: false,
            is_autorun_inf: false,
            is_root_shell_script: false,
            children: vec![],
        };
        let snapshot = PolicySnapshot {
            exec_control_enabled: true,
            file_type_blacklist_enabled: false,
            auto_read_control_enabled: false,
            blacklist_extensions: HashSet::new(),
            permission: 1,
        };

        let volume = VirtualVolume::build(&[entry], &snapshot);
        let sectors = volume.find_file_data_sectors("tool.bin");
        assert!(!sectors.is_empty());
        match volume.read_sector(sectors[0]) {
            SectorContent::FileData { blocked, .. } => assert!(blocked),
            other => panic!("expected file data sector, got {other:?}"),
        }
    }
}
