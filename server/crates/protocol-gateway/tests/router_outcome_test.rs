mod support;

use common::code::ResultCode;
use prost::Message;
use protocol_gateway::codec;
use protocol_gateway::context::RequestContext;
use protocol_gateway::post_send::{HandlerOutcome, PostSendAction};
use protocol_gateway::router::Router;

use support::request_fixture;

const RSP_COMMON: u32 = 0xFF00;

#[test]
fn ordinary_handler_has_no_post_send_action() {
    let mut router = Router::new();
    router.register(
        0x0001,
        Box::new(|ctx: &RequestContext, _payload| {
            codec::encode_frame(0x0002, ctx.seq_id, b"ok").expect("encode response")
        }),
    );

    let fixture = request_fixture(42);
    let outcome = router.dispatch(&fixture.context, 0x0001, &[]);

    assert!(outcome.post_send_action.is_none());
    let (header, payload, _) = codec::try_decode_frame(&outcome.response)
        .expect("decode response")
        .expect("complete response");
    assert_eq!(header.msg_type, 0x0002);
    assert_eq!(header.seq_id, 42);
    assert_eq!(payload, b"ok");
}

#[test]
fn upgrade_handler_returns_start_upgrade_action() {
    let mut router = Router::new();
    router.register_outcome_with_roles(
        0x0302,
        Box::new(|_ctx, _payload| HandlerOutcome {
            response: b"accepted".to_vec(),
            post_send_action: Some(PostSendAction::StartSystemUpgrade {
                upgrade_id: "upgrade-42".into(),
            }),
        }),
        vec![0],
    );

    let fixture = request_fixture(7);
    let outcome = router.dispatch(&fixture.context, 0x0302, &[]);

    assert_eq!(outcome.response, b"accepted");
    match outcome.post_send_action {
        Some(PostSendAction::StartSystemUpgrade { upgrade_id }) => {
            assert_eq!(upgrade_id, "upgrade-42")
        }
        None => panic!("upgrade outcome must contain a post-send action"),
    }
    assert_eq!(router.allowed_roles(0x0302), &[0]);
}

#[test]
fn unknown_command_has_no_post_send_action() {
    let router = Router::new();
    let fixture = request_fixture(1);
    let outcome = router.dispatch(&fixture.context, 0x9999, &[]);

    assert!(outcome.post_send_action.is_none());
    let (header, payload, _) = codec::try_decode_frame(&outcome.response)
        .expect("decode response")
        .expect("complete response");
    assert_eq!(header.msg_type, RSP_COMMON);
    assert_eq!(header.seq_id, 1);

    let rsp = common::proto::RspCommon::decode(payload.as_slice()).expect("decode protobuf");
    assert!(!rsp.success);
    assert_eq!(
        rsp.result_code,
        ResultCode::ValidationFailed.as_u16() as i32
    );
    assert_eq!(rsp.error_message, "unknown command");
}

#[test]
fn route_registration_preserves_role_metadata() {
    let mut router = Router::new();
    router.register_with_roles(0x0010, Box::new(|_ctx, _payload| Vec::new()), vec![0, 1]);

    assert!(router.is_registered(0x0010));
    assert_eq!(router.allowed_roles(0x0010), &[0, 1]);
    assert!(!router.is_registered(0x9999));
    assert!(router.allowed_roles(0x9999).is_empty());
}
