//! Allocation Bitmap 生成。
//!
//! 标记已分配的簇。每个 bit 对应一个簇（bit 0 = cluster 2）。

/// 生成 Allocation Bitmap。
///
/// 参数:
///   - cluster_count: 总簇数。
///   - allocated_clusters: 已分配的簇数（从 cluster 2 开始连续分配）。
///
/// 返回:
///   - bitmap 数据（按簇大小对齐）。
pub fn generate_bitmap(cluster_count: u32, allocated_clusters: u32) -> Vec<u8> {
    let bitmap_bits = cluster_count as usize;
    let bitmap_bytes = bitmap_bits.div_ceil(8);
    let mut bitmap = vec![0u8; bitmap_bytes];

    // 标记已分配的簇
    for i in 0..allocated_clusters as usize {
        let byte_idx = i / 8;
        let bit_idx = i % 8;
        if byte_idx < bitmap.len() {
            bitmap[byte_idx] |= 1 << bit_idx;
        }
    }

    bitmap
}
