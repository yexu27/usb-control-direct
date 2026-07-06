//! 块设备后端抽象。
//!
//! NBD 层只依赖该 trait，不理解 exFAT、策略、病毒文件或真实文件写回语义。

use std::io;

/// 为 NBD 提供按字节偏移读写的块后端。
///
/// 参数:
/// - `offset`: 字节偏移。
/// - `len`: 读取长度。
/// - `data`: 写入数据。
///
/// 返回:
/// - 成功时完成对应块操作。
/// - 失败时返回 `io::Error`，由 NBD request loop 转换成协议错误码。
pub trait BlockBackend: Send + Sync + 'static {
    fn read_at(&self, offset: u64, len: usize) -> io::Result<Vec<u8>>;
    fn write_at(&self, offset: u64, data: &[u8]) -> io::Result<()>;
    fn flush(&self) -> io::Result<()>;

    fn shutdown(&self) -> io::Result<()> {
        self.flush()
    }
}
