//! 进程退出信号处理。

/// 等待服务退出信号。
pub async fn wait_for_shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let mut interrupt = signal(SignalKind::interrupt()).expect("SIGINT 监听初始化失败");
        let mut terminate = signal(SignalKind::terminate()).expect("SIGTERM 监听初始化失败");

        tokio::select! {
            _ = interrupt.recv() => {}
            _ = terminate.recv() => {}
        }
    }

    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c().await.expect("Ctrl-C 监听失败");
    }
}
