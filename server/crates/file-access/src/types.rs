//! S04 核心数据结构。

use std::collections::HashSet;
use std::path::PathBuf;

/// 可执行文件类型（L2 检测结果）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecFileType {
    /// Windows PE 可执行文件（.exe）。
    Pe,
    /// Windows DLL。
    Dll,
    /// Linux ELF 可执行文件。
    Elf,
}

/// 受控文件树节点。
#[derive(Debug, Clone)]
pub struct ControlledEntry {
    /// 真实文件路径（/mnt/usb_raw 下）。
    pub real_path: PathBuf,
    /// 虚拟路径（相对于虚拟根）。
    pub virtual_name: String,
    /// 文件大小（字节）。
    pub file_size: u64,
    /// 是否为目录。
    pub is_dir: bool,
    /// 病毒标记（L1）。
    pub is_virus: bool,
    /// 可执行文件类型（L2 检测结果，None 表示非可执行文件）。
    pub exec_type: Option<ExecFileType>,
    /// 文件后缀（小写，不含点号）。
    pub extension: String,
    /// 是否为 autorun.inf 引用的可执行文件（L4）。
    pub is_autorun_target: bool,
    /// 是否为 autorun.inf 文件本身。
    pub is_autorun_inf: bool,
    /// 是否为根目录下的 shell 脚本（.sh / .bash）。
    pub is_root_shell_script: bool,
    /// 子节点（仅目录有效）。
    pub children: Vec<ControlledEntry>,
}

/// 策略快照（U 盘插入时一次性读取 T02/T03/T04）。
#[derive(Debug, Clone)]
pub struct PolicySnapshot {
    /// L2 可执行控制开关。
    pub exec_control_enabled: bool,
    /// L3 文件类型黑名单开关。
    pub file_type_blacklist_enabled: bool,
    /// L4 自运行控制开关。
    pub auto_read_control_enabled: bool,
    /// L3 黑名单后缀集合（带单个前导点，ASCII 小写）。
    pub blacklist_extensions: HashSet<String>,
    /// 权限：0=只读，1=读写。
    pub permission: i32,
}

/// 访问决策。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessDecision {
    /// 允许访问。
    Allow,
    /// 拒绝访问（返回 I/O error），附带命中的策略级别描述。
    Deny(String),
}

/// 虚拟扇区内容类型。
#[derive(Debug, Clone)]
pub enum SectorContent {
    /// 预生成的元数据（boot / FAT / bitmap / upcase / 目录项）。
    Metadata(Vec<u8>),
    /// 文件数据（映射到真实文件路径 + 偏移量）。
    FileData {
        real_path: PathBuf,
        offset: u64,
        /// 该扇区中的有效字节数（最后一个扇区可能不满 512）。
        valid_bytes: u32,
        /// 是否被策略阻断。
        blocked: bool,
    },
    /// 零填充（空闲区域）。
    Zero,
}
