use std::os::unix::fs::PermissionsExt;

use license_upgrade::VirusdbUpgradeManager;

#[test]
fn read_status_returns_real_clamav_status() {
    let tmp = tempfile::tempdir().unwrap();
    let clamscan = tmp.path().join("clamscan");
    std::fs::write(
        &clamscan,
        "#!/bin/sh\nprintf '%s\\n' 'ClamAV 1.4.4/28045/Sun Jun 28 14:26:16 2026'\n",
    )
    .unwrap();
    let mut permissions = std::fs::metadata(&clamscan).unwrap().permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&clamscan, permissions).unwrap();

    let mgr = VirusdbUpgradeManager::new_with_clamscan("/tmp/test-virusdb", &clamscan);
    let status = mgr.read_status().unwrap();

    assert_eq!(status.virus_db_version, "28045");
    assert_eq!(status.virus_db_updated_at, 1_782_656_776);
}
