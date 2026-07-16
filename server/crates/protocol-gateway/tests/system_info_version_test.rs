mod support;

use common::code::ResultCode;
use common::proto::{CmdGetSystemInfo, RspCommon, RspSystemInfo};
use prost::Message;
use protocol_gateway::codec;
use protocol_gateway::handlers::system::handle_get_system_info;

use support::request_fixture;

#[test]
fn system_info_returns_database_business_version() {
    let fixture = request_fixture(501);
    fixture
        .context
        .storage()
        .unwrap()
        .config_set("system_version", "3.0.1")
        .unwrap();
    let frame = handle_get_system_info(
        &fixture.context,
        &CmdGetSystemInfo {
            session_token: "validated".into(),
        }
        .encode_to_vec(),
    );
    let (_, payload, _) = codec::try_decode_frame(&frame)
        .unwrap()
        .expect("complete system info frame");
    let response = RspSystemInfo::decode(payload.as_slice()).unwrap();
    assert_eq!(response.system_version, "3.0.1");
}

#[test]
fn system_info_rejects_missing_or_invalid_database_version() {
    for (seq_id, version) in [(502, ""), (503, "V3.0.1"), (504, "3.0")] {
        let fixture = request_fixture(seq_id);
        if version.is_empty() {
            let conn = rusqlite::Connection::open(&fixture.database).unwrap();
            conn.execute(
                "DELETE FROM system_config WHERE config_key='system_version'",
                [],
            )
            .unwrap();
        } else {
            fixture
                .context
                .storage()
                .unwrap()
                .config_set("system_version", version)
                .unwrap();
        }
        let frame = handle_get_system_info(
            &fixture.context,
            &CmdGetSystemInfo {
                session_token: "validated".into(),
            }
            .encode_to_vec(),
        );
        let (_, payload, _) = codec::try_decode_frame(&frame)
            .unwrap()
            .expect("complete system info error frame");
        let response = RspCommon::decode(payload.as_slice()).unwrap();
        assert!(!response.success);
        assert_eq!(
            response.result_code,
            ResultCode::InternalError.as_u16() as i32
        );
    }
}
