//! 严格三段式系统版本值对象。

use std::fmt::{Display, Formatter};
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::UpgradeError;

/// 严格的 `major.minor.patch` 系统版本。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SystemVersion {
    major: u64,
    minor: u64,
    patch: u64,
}

impl SystemVersion {
    /// 解析严格三段式系统版本。
    ///
    /// 参数:
    /// - `value`: 只包含十进制数字和两个点号的版本字符串。
    ///
    /// 返回:
    /// - 成功时返回系统版本；格式非法或数值溢出时返回错误。
    pub fn parse(value: &str) -> Result<Self, UpgradeError> {
        let mut parts = value.split('.');
        let major = parse_part(parts.next(), value)?;
        let minor = parse_part(parts.next(), value)?;
        let patch = parse_part(parts.next(), value)?;

        if parts.next().is_some() {
            return Err(UpgradeError::InvalidVersion(value.to_string()));
        }

        Ok(Self {
            major,
            minor,
            patch,
        })
    }
}

impl Display for SystemVersion {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl FromStr for SystemVersion {
    type Err = UpgradeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl Serialize for SystemVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for SystemVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value).map_err(serde::de::Error::custom)
    }
}

fn parse_part(part: Option<&str>, value: &str) -> Result<u64, UpgradeError> {
    let part = part.ok_or_else(|| UpgradeError::InvalidVersion(value.to_string()))?;
    let has_leading_zero = part.len() > 1 && part.starts_with('0');
    let contains_only_ascii_digits = part.bytes().all(|byte| byte.is_ascii_digit());
    if part.is_empty() || has_leading_zero || !contains_only_ascii_digits {
        return Err(UpgradeError::InvalidVersion(value.to_string()));
    }

    part.parse()
        .map_err(|_| UpgradeError::InvalidVersion(value.to_string()))
}
