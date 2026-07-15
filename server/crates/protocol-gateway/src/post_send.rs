//! 响应完整发送后的动作契约。
//!
//! 网关只负责保证动作与响应发送结果之间的时序，不了解动作背后的升级协调、
//! 日志或进程管理实现。

use crate::GatewayError;

/// 只能在响应完整写入并刷新后执行的动作。
#[derive(Debug, PartialEq, Eq)]
pub enum PostSendAction {
    /// 启动已完成受理的系统升级任务。
    StartSystemUpgrade { upgrade_id: String },
}

/// handler 的响应以及可选的发送后动作。
pub struct HandlerOutcome {
    pub response: Vec<u8>,
    pub post_send_action: Option<PostSendAction>,
}

impl HandlerOutcome {
    /// 构造不包含发送后动作的普通响应。
    pub fn response(response: Vec<u8>) -> Self {
        Self {
            response,
            post_send_action: None,
        }
    }
}

/// 发送后动作的应用层执行端口。
pub trait PostSendActionExecutor: Send + Sync {
    /// 响应已完整发送时消费并执行动作。
    fn execute(&self, action: PostSendAction) -> Result<(), GatewayError>;

    /// 响应发送失败时取消尚未执行的动作。
    fn cancel(&self, action: &PostSendAction);
}
