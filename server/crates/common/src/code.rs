//! 装置端统一结果码。
//!
//! 与 `docs/architecture/统一结果码表.md` 全表对齐。每个变体的 `as u16`
//! 取值必须保持与结果码表完全一致；新增码必须先更新结果码表，再回填本枚举。
//!
//! ResultCode 在协议层通过 proto 中 `int32` 字段传输；管理端按数字码查
//! 提示文案，未匹配时退回 `error_message` 原文。

use std::fmt;

/// 全部装置端结果码。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum ResultCode {
    /// 操作成功。
    Success = 0x0000,

    // ===== 全局 0x0001-0x000F =====
    /// 会话无效或已过期。
    Unauthenticated = 0x0001,
    /// 装置未授权。
    Unauthorized = 0x0002,
    /// 当前角色无权限。
    PermissionDenied = 0x0003,
    /// 账号已锁定。
    AccountLocked = 0x0004,
    /// 请求参数校验失败。
    ValidationFailed = 0x0005,
    /// 装置正在执行其他操作。
    DeviceBusy = 0x0006,
    /// 装置内部错误。
    InternalError = 0x0007,

    // ===== 登录鉴权 0x0101-0x01FF =====
    /// 用户名或密码错误。
    UserOrPasswordError = 0x0101,
    /// 装置未授权，需进入授权页（登录场景下的语义）。
    DeviceUnauthorized = 0x0102,
    /// 授权文件格式错误。
    LicenseFormatError = 0x0103,
    /// 授权文件校验失败（签名/匹配/有效期任一不通过）。
    LicenseVerifyFailed = 0x0104,
    /// 上传的授权文件已过有效期。
    LicenseExpired = 0x0105,

    // ===== 白名单 0x0201-0x02FF =====
    /// 序列号为空。
    SerialNumberEmpty = 0x0201,
    /// 序列号已存在。
    AlreadyExists = 0x0202,
    /// 目标设备不存在。
    NotFound = 0x0203,
    /// 非存储设备不可添加。
    DeviceNotStorage = 0x0204,
    /// 疑似伪装设备（描述符与接口能力不一致）。
    DeviceSpoofSuspected = 0x0205,
    /// 设备类型不支持。
    DeviceUnsupported = 0x0206,

    // ===== 文件访问控制策略 0x0301-0x03FF =====
    /// 策略开关标识不合法。
    PolicyKeyInvalid = 0x0301,
    /// 后缀格式错误。
    ExtensionFormatError = 0x0302,
    /// 后缀已存在。
    ExtensionExists = 0x0303,
    /// 指定后缀不在黑名单中。
    ExtensionNotFound = 0x0304,
    /// 内置默认后缀不可删除。
    DefaultExtensionNoDelete = 0x0305,

    // ===== 策略导入导出 0x0401-0x04FF =====
    /// 策略数据读取或加密失败（导出）。
    PolicyExportFailed = 0x0401,
    /// 策略文件格式错误（魔数不匹配）。
    PolicyFormatError = 0x0402,
    /// SM2 签名校验失败。
    PolicySignatureError = 0x0403,
    /// SM3 摘要校验失败。
    PolicyDigestError = 0x0404,
    /// SM4 解密失败。
    PolicyDecryptError = 0x0405,
    /// 策略文件版本不兼容。
    PolicyVersionIncompatible = 0x0406,
    /// 策略写入数据库失败（已回滚）。
    PolicyImportFailed = 0x0407,

    // ===== 日志管理 0x0501-0x05FF =====
    /// 半年内日志不可清理。
    LogRetentionViolation = 0x0501,
    /// 日志类型参数不合法。
    LogTypeInvalid = 0x0502,
    /// 日志查询失败（数据库异常）。
    LogQueryFailed = 0x0503,
    /// 日志导出生成失败。
    LogExportFailed = 0x0504,

    // ===== 系统管理 0x0601-0x06FF =====
    /// 升级包版本低于当前版本。
    VersionTooLow = 0x0601,
    /// 升级包文件格式错误。
    UpgradeFormatError = 0x0602,
    /// SHA256 校验和不匹配。
    UpgradeChecksumError = 0x0603,
    /// 系统升级安装失败（已回滚至原版本）。
    UpgradeApplyFailed = 0x0604,
    /// 病毒库版本号命中逢 4 跳过规则。
    VersionNumberForbidden = 0x0605,
    /// 病毒库文件完整性校验失败。
    VirusdbIntegrityError = 0x0606,
    /// clamd 重新加载失败（已回滚）。
    ClamdReloadFailed = 0x0607,
    /// 病毒库文件替换失败。
    VirusdbApplyFailed = 0x0608,
    /// 设备描述格式不合法。
    DeviceDescFormatError = 0x0609,

    // ===== 用户管理 0x0701-0x07FF =====
    /// 用户名已存在。
    UsernameExists = 0x0701,
    /// 已删除用户名不可复用。
    UsernameDeletedReuse = 0x0702,
    /// 目标用户不存在。
    UserNotFound = 0x0703,
    /// 密码复杂度不符合要求。
    PasswordComplexityError = 0x0704,
    /// 两次密码不一致。
    PasswordConfirmMismatch = 0x0705,
    /// 旧密码错误。
    OldPasswordError = 0x0706,
    /// 内置用户不可删除。
    BuiltinUserNoDelete = 0x0707,
    /// 不允许删除当前登录用户。
    SelfDeleteForbidden = 0x0708,
}

impl ResultCode {
    /// 返回结果码的数字值，供协议层 `int32` 字段使用。
    pub const fn as_u16(self) -> u16 {
        self as u16
    }
}

impl fmt::Display for ResultCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:04X}", self.as_u16())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 逐项验证结果码的数字编码值，与 `统一结果码表.md` 严格对齐。
    #[test]
    fn result_code_numeric_values() {
        assert_eq!(ResultCode::Success.as_u16(), 0x0000);
        assert_eq!(ResultCode::Unauthenticated.as_u16(), 0x0001);
        assert_eq!(ResultCode::Unauthorized.as_u16(), 0x0002);
        assert_eq!(ResultCode::PermissionDenied.as_u16(), 0x0003);
        assert_eq!(ResultCode::AccountLocked.as_u16(), 0x0004);
        assert_eq!(ResultCode::ValidationFailed.as_u16(), 0x0005);
        assert_eq!(ResultCode::DeviceBusy.as_u16(), 0x0006);
        assert_eq!(ResultCode::InternalError.as_u16(), 0x0007);
        assert_eq!(ResultCode::UserOrPasswordError.as_u16(), 0x0101);
        assert_eq!(ResultCode::DeviceUnauthorized.as_u16(), 0x0102);
        assert_eq!(ResultCode::LicenseFormatError.as_u16(), 0x0103);
        assert_eq!(ResultCode::LicenseVerifyFailed.as_u16(), 0x0104);
        assert_eq!(ResultCode::LicenseExpired.as_u16(), 0x0105);
        assert_eq!(ResultCode::SerialNumberEmpty.as_u16(), 0x0201);
        assert_eq!(ResultCode::AlreadyExists.as_u16(), 0x0202);
        assert_eq!(ResultCode::NotFound.as_u16(), 0x0203);
        assert_eq!(ResultCode::DeviceNotStorage.as_u16(), 0x0204);
        assert_eq!(ResultCode::DeviceSpoofSuspected.as_u16(), 0x0205);
        assert_eq!(ResultCode::DeviceUnsupported.as_u16(), 0x0206);
        assert_eq!(ResultCode::PolicyKeyInvalid.as_u16(), 0x0301);
        assert_eq!(ResultCode::ExtensionFormatError.as_u16(), 0x0302);
        assert_eq!(ResultCode::ExtensionExists.as_u16(), 0x0303);
        assert_eq!(ResultCode::ExtensionNotFound.as_u16(), 0x0304);
        assert_eq!(ResultCode::DefaultExtensionNoDelete.as_u16(), 0x0305);
        assert_eq!(ResultCode::PolicyExportFailed.as_u16(), 0x0401);
        assert_eq!(ResultCode::PolicyFormatError.as_u16(), 0x0402);
        assert_eq!(ResultCode::PolicySignatureError.as_u16(), 0x0403);
        assert_eq!(ResultCode::PolicyDigestError.as_u16(), 0x0404);
        assert_eq!(ResultCode::PolicyDecryptError.as_u16(), 0x0405);
        assert_eq!(ResultCode::PolicyVersionIncompatible.as_u16(), 0x0406);
        assert_eq!(ResultCode::PolicyImportFailed.as_u16(), 0x0407);
        assert_eq!(ResultCode::LogRetentionViolation.as_u16(), 0x0501);
        assert_eq!(ResultCode::LogTypeInvalid.as_u16(), 0x0502);
        assert_eq!(ResultCode::LogQueryFailed.as_u16(), 0x0503);
        assert_eq!(ResultCode::LogExportFailed.as_u16(), 0x0504);
        assert_eq!(ResultCode::VersionTooLow.as_u16(), 0x0601);
        assert_eq!(ResultCode::UpgradeFormatError.as_u16(), 0x0602);
        assert_eq!(ResultCode::UpgradeChecksumError.as_u16(), 0x0603);
        assert_eq!(ResultCode::UpgradeApplyFailed.as_u16(), 0x0604);
        assert_eq!(ResultCode::VersionNumberForbidden.as_u16(), 0x0605);
        assert_eq!(ResultCode::VirusdbIntegrityError.as_u16(), 0x0606);
        assert_eq!(ResultCode::ClamdReloadFailed.as_u16(), 0x0607);
        assert_eq!(ResultCode::VirusdbApplyFailed.as_u16(), 0x0608);
        assert_eq!(ResultCode::DeviceDescFormatError.as_u16(), 0x0609);
        assert_eq!(ResultCode::UsernameExists.as_u16(), 0x0701);
        assert_eq!(ResultCode::UsernameDeletedReuse.as_u16(), 0x0702);
        assert_eq!(ResultCode::UserNotFound.as_u16(), 0x0703);
        assert_eq!(ResultCode::PasswordComplexityError.as_u16(), 0x0704);
        assert_eq!(ResultCode::PasswordConfirmMismatch.as_u16(), 0x0705);
        assert_eq!(ResultCode::OldPasswordError.as_u16(), 0x0706);
        assert_eq!(ResultCode::BuiltinUserNoDelete.as_u16(), 0x0707);
        assert_eq!(ResultCode::SelfDeleteForbidden.as_u16(), 0x0708);
    }

    #[test]
    fn result_code_display_uppercase_hex() {
        assert_eq!(format!("{}", ResultCode::Success), "0x0000");
    }
}
