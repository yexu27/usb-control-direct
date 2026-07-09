//! Typed errors for recoverable exFAT policy rejections.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RecoverablePolicyRejection {
    pub virtual_path: String,
    pub operation: String,
    pub reason: String,
}

impl RecoverablePolicyRejection {
    pub fn blocked_placeholder(virtual_path: String, operation: &str, reason: String) -> Self {
        Self {
            virtual_path,
            operation: operation.to_string(),
            reason,
        }
    }

    pub fn from_io_error(err: &std::io::Error) -> Option<&Self> {
        err.get_ref()?.downcast_ref::<Self>()
    }

    pub fn to_outcome_reason(&self) -> String {
        format!(
            "blocked placeholder {} rejected for {}: {}",
            self.virtual_path, self.operation, self.reason
        )
    }

    pub fn into_io_error(self) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::PermissionDenied, self)
    }
}

impl fmt::Display for RecoverablePolicyRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "策略命中文件禁止修改: path={} operation={} reason={}",
            self.virtual_path, self.operation, self.reason
        )
    }
}

impl std::error::Error for RecoverablePolicyRejection {}
