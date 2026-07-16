mod support;

use common::proto::{CmdGetSystemInfo, RspSystemInfo};
use prost::Message;
use protocol_gateway::codec;
use protocol_gateway::handlers::system::handle_get_system_info;
use system_upgrade::{ActiveRelease, ActiveReleaseStore, SystemVersion, UpgradeStateLock};

use support::request_fixture;

#[test]
fn system_info_reads_committed_release_not_candidate_version() {
    let fixture = request_fixture(501);
    fixture
        .context
        .storage()
        .unwrap()
        .config_set("system_version", "3.0.1")
        .unwrap();
    let releases = ActiveReleaseStore::new(fixture.upgrade_root.path().to_path_buf()).unwrap();
    let guard = UpgradeStateLock::acquire(fixture.upgrade_root.path()).unwrap();
    releases
        .commit(
            &guard,
            &ActiveRelease {
                format_version: 1,
                version: SystemVersion::parse("3.0.2").unwrap(),
                schema_version: 1,
                committed_at: 200,
                online_upgrade_id: None,
            },
        )
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
    assert_eq!(response.system_version, "3.0.2");
}
