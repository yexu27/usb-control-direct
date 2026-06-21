use storage::{Storage, StorageError};

/// 装置当前授权快照。
pub(crate) struct LicenseSnapshot {
    pub authorized: bool,
    pub status: String,
    pub expire_time: i64,
    pub device_description: String,
}

/// 从装置配置读取并解释当前授权状态。
pub(crate) fn read_license_snapshot(
    storage: &Storage,
    now: i64,
) -> Result<LicenseSnapshot, StorageError> {
    let auth_status = config_value(storage, "auth_status")?;
    let expire_time = config_value(storage, "auth_expire_time")?
        .parse::<i64>()
        .unwrap_or(0);
    let device_description = config_value(storage, "device_description")?;

    Ok(derive_license_snapshot(
        &auth_status,
        expire_time,
        &device_description,
        now,
    ))
}

fn config_value(storage: &Storage, key: &str) -> Result<String, StorageError> {
    Ok(storage
        .config_get(key)?
        .and_then(|config| config.config_value)
        .unwrap_or_default())
}

fn derive_license_snapshot(
    configured_status: &str,
    expire_time: i64,
    device_description: &str,
    now: i64,
) -> LicenseSnapshot {
    let status = if configured_status != "authorized" {
        "unauthorized"
    } else if expire_time > 0 && expire_time <= now {
        "expired"
    } else {
        "authorized"
    };

    LicenseSnapshot {
        authorized: status == "authorized",
        status: status.to_string(),
        expire_time,
        device_description: device_description.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::derive_license_snapshot;

    #[test]
    fn reports_authorized_state_before_expiry() {
        let snapshot = derive_license_snapshot(
            "authorized",
            1_893_455_999,
            "USB_DEVICE_01",
            1_800_000_000,
        );

        assert!(snapshot.authorized);
        assert_eq!(snapshot.status, "authorized");
        assert_eq!(snapshot.expire_time, 1_893_455_999);
        assert_eq!(snapshot.device_description, "USB_DEVICE_01");
    }

    #[test]
    fn reports_unauthorized_state_from_device_config() {
        let snapshot = derive_license_snapshot("unauthorized", 0, "", 1_800_000_000);

        assert!(!snapshot.authorized);
        assert_eq!(snapshot.status, "unauthorized");
    }

    #[test]
    fn reports_expired_state_at_expiry_boundary() {
        let snapshot = derive_license_snapshot(
            "authorized",
            1_800_000_000,
            "USB_DEVICE_01",
            1_800_000_000,
        );

        assert!(!snapshot.authorized);
        assert_eq!(snapshot.status, "expired");
    }

    #[test]
    fn treats_zero_expiry_as_non_expiring_authorization() {
        let snapshot = derive_license_snapshot("authorized", 0, "USB_DEVICE_01", 1_800_000_000);

        assert!(snapshot.authorized);
        assert_eq!(snapshot.status, "authorized");
    }
}
