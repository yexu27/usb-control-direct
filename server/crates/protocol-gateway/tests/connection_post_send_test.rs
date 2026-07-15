mod support;

use std::sync::Arc;
use std::{io, time::Duration};

use protocol_gateway::connection::{send_handler_outcome, ConnectionManager};
use protocol_gateway::post_send::{HandlerOutcome, PostSendAction};
use protocol_gateway::GatewayError;
use tokio::io::{duplex, AsyncReadExt};

use support::{ExecutorEvent, FlushFailWriter, RecordingExecutor};

#[tokio::test]
async fn executes_action_only_after_response_is_written() {
    let response = vec![0x5a; 32];
    let expected = response.clone();
    let (mut server, mut client) = duplex(4);
    let executor = Arc::new(RecordingExecutor::default());
    let observed = Arc::clone(&executor);
    let mut task = tokio::spawn(async move {
        send_handler_outcome(
            &mut server,
            executor.as_ref(),
            HandlerOutcome {
                response,
                post_send_action: Some(PostSendAction::StartSystemUpgrade {
                    upgrade_id: "upgrade-42".into(),
                }),
            },
        )
        .await
    });

    assert!(
        tokio::time::timeout(Duration::from_millis(20), &mut task)
            .await
            .is_err(),
        "send must remain pending while the bounded duplex buffer is full"
    );
    assert!(observed.events().is_empty());

    let mut received = Vec::new();
    // 始终至少保留两个缓冲区容量，确保每次中间断言时 write_all 仍被阻塞。
    while received.len() + 8 < expected.len() {
        let mut chunk = [0_u8; 4];
        client
            .read_exact(&mut chunk)
            .await
            .expect("read response chunk");
        received.extend_from_slice(&chunk);
        tokio::task::yield_now().await;
        assert!(
            observed.events().is_empty(),
            "action cannot execute while write_all is still blocked"
        );
    }
    let mut final_chunk = vec![0_u8; expected.len() - received.len()];
    client
        .read_exact(&mut final_chunk)
        .await
        .expect("read final response chunk");
    received.extend_from_slice(&final_chunk);
    assert_eq!(received, expected);
    task.await
        .expect("send task panicked")
        .expect("send outcome");
    assert_eq!(
        observed.events(),
        vec![ExecutorEvent::Executed("upgrade-42".into())]
    );
}

#[tokio::test]
async fn cancels_prepared_task_when_flush_fails() {
    let mut writer = FlushFailWriter::new(io::ErrorKind::BrokenPipe);
    let executor = RecordingExecutor::default();

    let result = send_handler_outcome(
        &mut writer,
        &executor,
        HandlerOutcome {
            response: b"accepted".to_vec(),
            post_send_action: Some(PostSendAction::StartSystemUpgrade {
                upgrade_id: "upgrade-flush".into(),
            }),
        },
    )
    .await;

    assert_eq!(writer.written(), b"accepted");
    match result {
        Err(GatewayError::Io(error)) => assert_eq!(error.kind(), io::ErrorKind::BrokenPipe),
        other => panic!("expected injected IO error, got {other:?}"),
    }
    assert_eq!(
        executor.events(),
        vec![ExecutorEvent::Cancelled("upgrade-flush".into())]
    );
}

#[tokio::test]
async fn no_action_never_calls_executor_on_success_or_write_failure() {
    let success_executor = RecordingExecutor::default();
    let (mut success_stream, mut peer) = duplex(16);
    send_handler_outcome(
        &mut success_stream,
        &success_executor,
        HandlerOutcome {
            response: b"ok".to_vec(),
            post_send_action: None,
        },
    )
    .await
    .expect("send response without action");
    let mut response = [0_u8; 2];
    peer.read_exact(&mut response)
        .await
        .expect("read successful response");
    assert_eq!(&response, b"ok");
    assert!(success_executor.events().is_empty());

    let failure_executor = RecordingExecutor::default();
    let (mut failed_stream, failed_peer) = duplex(16);
    drop(failed_peer);
    let result = send_handler_outcome(
        &mut failed_stream,
        &failure_executor,
        HandlerOutcome {
            response: b"not written".to_vec(),
            post_send_action: None,
        },
    )
    .await;
    assert!(matches!(result, Err(GatewayError::Io(_))));
    assert!(failure_executor.events().is_empty());
}

#[tokio::test]
async fn execute_error_is_returned_without_cancellation() {
    let (mut stream, _peer) = duplex(32);
    let executor = RecordingExecutor::failing_execute("executor failed");

    let result = send_handler_outcome(
        &mut stream,
        &executor,
        HandlerOutcome {
            response: b"accepted".to_vec(),
            post_send_action: Some(PostSendAction::StartSystemUpgrade {
                upgrade_id: "upgrade-execute-error".into(),
            }),
        },
    )
    .await;

    assert!(matches!(
        result,
        Err(GatewayError::TlsConfig(message)) if message == "executor failed"
    ));
    assert_eq!(
        executor.events(),
        vec![ExecutorEvent::Executed("upgrade-execute-error".into())]
    );
}

#[tokio::test]
async fn cancels_prepared_task_when_write_fails() {
    let (mut server, client) = duplex(64);
    drop(client);
    let executor = RecordingExecutor::default();

    let result = send_handler_outcome(
        &mut server,
        &executor,
        HandlerOutcome {
            response: b"accepted".to_vec(),
            post_send_action: Some(PostSendAction::StartSystemUpgrade {
                upgrade_id: "upgrade-43".into(),
            }),
        },
    )
    .await;

    assert!(result.is_err());
    assert_eq!(
        executor.events(),
        vec![ExecutorEvent::Cancelled("upgrade-43".into())]
    );
}

#[test]
fn connection_manager_allows_only_one_connection() {
    let manager = ConnectionManager::new();
    let guard = manager.try_acquire().expect("first connection");
    assert!(manager.try_acquire().is_err());
    drop(guard);
    assert!(manager.try_acquire().is_ok());
}

#[test]
fn connection_guard_releases_slot_on_drop() {
    let manager = ConnectionManager::new();
    {
        let _guard = manager.try_acquire().expect("first connection");
    }
    let _guard = manager.try_acquire().expect("slot released after drop");
}
