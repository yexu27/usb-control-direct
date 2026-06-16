//! 装置端公共类型枚举。
//!
//! 所有跨 crate 共享的类型集中在本模块；新增枚举值必须同步更新映射函数与
//! 协议层定义，避免出现各模块各写一份。

/// USB 设备类型。与 proto `ConnectedDevice.device_type`、T05 `device_type`
/// 字段对齐。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceType {
    /// 普通大容量存储设备。
    Storage,
    /// 键盘。
    Keyboard,
    /// 鼠标。
    Mouse,
    /// 已识别但不支持的设备类型。
    Unsupported,
    /// 无法识别的设备。
    Unknown,
}

/// 用户角色。协议层使用字符串 admin/operator/auditor，数据库层使用整数
/// 0/1/2。详见 `mapping::role_str_to_int` / `mapping::role_int_to_str`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    /// 系统管理员。
    Admin,
    /// 操作员。
    Operator,
    /// 审计员。
    Auditor,
}

/// 白名单设备访问权限。
///
/// 协议层使用字符串 `readonly` / `readwrite`，T01 数据库层使用整数
/// `0` / `1`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    /// 只读。
    ReadOnly,
    /// 读写。
    ReadWrite,
}

/// 操作日志分类。对应 T10 `log_type` 字段与 PRD 操作日志页面分类筛选。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogType {
    /// 登录认证。
    LoginAuth,
    /// 用户管理。
    UserManagement,
    /// 安全配置变更。
    SecurityConfig,
    /// 授权管理。
    AuthManagement,
    /// 系统管理。
    SystemManagement,
    /// 程序升级。
    ProgramUpgrade,
    /// 日志管理。
    LogManagement,
}

/// U 盘设备状态机状态。对应架构 06 §1。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UsbDeviceState {
    /// 已检测到 USB 插入事件。
    Detected,
    /// 正在读取描述符并识别类型。
    Classifying,
    /// 默认拒绝映射（非存储/未知/伪装等）。
    Rejected,
    /// 已识别为存储但不在白名单。
    Blocked,
    /// 正在执行病毒扫描。
    Scanning,
    /// 扫描失败或病毒库不可用。
    ScanFailed,
    /// 扫描完成未发现病毒。
    Clean,
    /// 扫描完成发现病毒。
    Infected,
    /// NBD + OTG 映射成功。
    Mapped,
    /// NBD 启动或 OTG 映射失败。
    MapFailed,
    /// USB 已拔出，资源已清理。
    Removed,
}

/// 键盘设备状态机状态。对应架构 06 §2。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyboardState {
    /// 已检测为键盘。
    KbDetected,
    /// 等待用户顺序输入 1234。
    KbWaiting,
    /// 验证通过，键盘已映射。
    KbMapped,
    /// HID 接管失败，禁止映射。
    KbRejected,
    /// 键盘已拔出。
    KbRemoved,
}

/// 鼠标设备状态机状态。对应架构 06 §3。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseState {
    /// 已检测为鼠标。
    MouseDetected,
    /// 鼠标已自动映射。
    MouseMapped,
    /// 映射失败。
    MouseRejected,
    /// 鼠标已拔出。
    MouseRemoved,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_type_equality() {
        assert_eq!(DeviceType::Storage, DeviceType::Storage);
        assert_ne!(DeviceType::Storage, DeviceType::Keyboard);
    }

    #[test]
    fn role_three_variants() {
        let all = [Role::Admin, Role::Operator, Role::Auditor];
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn usb_device_state_variant_count_matches_diagram() {
        let all = [
            UsbDeviceState::Detected,
            UsbDeviceState::Classifying,
            UsbDeviceState::Rejected,
            UsbDeviceState::Blocked,
            UsbDeviceState::Scanning,
            UsbDeviceState::ScanFailed,
            UsbDeviceState::Clean,
            UsbDeviceState::Infected,
            UsbDeviceState::Mapped,
            UsbDeviceState::MapFailed,
            UsbDeviceState::Removed,
        ];
        assert_eq!(all.len(), 11);
    }

    #[test]
    fn keyboard_state_variant_count() {
        let all = [
            KeyboardState::KbDetected,
            KeyboardState::KbWaiting,
            KeyboardState::KbMapped,
            KeyboardState::KbRejected,
            KeyboardState::KbRemoved,
        ];
        assert_eq!(all.len(), 5);
    }

    #[test]
    fn mouse_state_variant_count() {
        let all = [
            MouseState::MouseDetected,
            MouseState::MouseMapped,
            MouseState::MouseRejected,
            MouseState::MouseRemoved,
        ];
        assert_eq!(all.len(), 4);
    }

    #[test]
    fn log_type_variant_count() {
        let all = [
            LogType::LoginAuth,
            LogType::UserManagement,
            LogType::SecurityConfig,
            LogType::AuthManagement,
            LogType::SystemManagement,
            LogType::ProgramUpgrade,
            LogType::LogManagement,
        ];
        assert_eq!(all.len(), 7);
    }
}
