//! 块设备后端抽象。
//!
//! NBD 层只依赖该 trait，不理解 exFAT、策略、病毒文件或真实文件写回语义。

use std::io;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockWriteOutcome {
    Committed,
    PolicyRejectedAndRestored { reason: String },
}

impl BlockWriteOutcome {
    pub fn is_success_for_block_device(&self) -> bool {
        matches!(
            self,
            Self::Committed | Self::PolicyRejectedAndRestored { .. }
        )
    }
}

/// 为 NBD 提供按字节偏移读写的块后端。
///
/// 参数:
/// - `offset`: 字节偏移。
/// - `len`: 读取长度。
/// - `data`: 写入数据。
///
/// 返回:
/// - `Committed`: 写入已提交。
/// - `PolicyRejectedAndRestored`: 写入代表策略拒绝，后端已恢复 canonical metadata；块设备层应视为成功。
/// - `io::Error`: 真实块设备失败、解析损坏、越界、只读权限等应反馈给 NBD 的错误。
pub trait BlockBackend: Send + Sync + 'static {
    fn read_at(&self, offset: u64, len: usize) -> io::Result<Vec<u8>>;
    fn write_at(&self, offset: u64, data: &[u8]) -> io::Result<BlockWriteOutcome>;
    fn flush(&self) -> io::Result<()>;

    fn shutdown(&self) -> io::Result<()> {
        self.flush()
    }
}
