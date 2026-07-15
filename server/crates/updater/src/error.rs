//! updater 内部错误分类，不映射管理协议结果码。

use thiserror::Error;

#[derive(Debug, Error)]
pub enum UpdaterError {
    #[error("升级任务无效: {0}")]
    TaskInvalid(String),
    #[error("{stage} 无法启动命令 {program}: {source}")]
    CommandSpawn {
        stage: String,
        program: String,
        #[source]
        source: std::io::Error,
    },
    #[error("{stage} 命令 {program} 执行超时")]
    CommandTimeout { stage: String, program: String },
    #[error("{stage} 命令 {program} 执行失败，退出状态 {status:?}")]
    CommandFailed {
        stage: String,
        program: String,
        status: Option<i32>,
    },
    #[error("数据库迁移失败: {0}")]
    MigrationFailed(String),
    #[error("健康检查失败: {0}")]
    HealthFailed(String),
    #[error("自动回滚失败；原始错误: {original}；回滚错误: {rollback}")]
    RollbackFailed { original: String, rollback: String },
    #[error("系统升级领域错误: {0}")]
    Domain(#[from] system_upgrade::UpgradeError),
    #[error("updater IO 失败: {0}")]
    Io(#[from] std::io::Error),
    #[error("updater 状态序列化失败: {0}")]
    Json(#[from] serde_json::Error),
}
