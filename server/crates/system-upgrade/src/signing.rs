//! 在线升级容器唯一的签名摘要协议。

use crate::UpgradeError;

const SIGNING_DOMAIN: &[u8] = b"USB-CONTROL-UPGRADE-V1\0";

pub fn upgrade_signing_digest(
    manifest_raw: &[u8],
    deb_sha256: &[u8; 32],
) -> Result<[u8; 32], UpgradeError> {
    let mut input = Vec::with_capacity(SIGNING_DOMAIN.len() + 8 + manifest_raw.len() + 32);
    input.extend_from_slice(SIGNING_DOMAIN);
    input.extend_from_slice(&(manifest_raw.len() as u64).to_be_bytes());
    input.extend_from_slice(manifest_raw);
    input.extend_from_slice(deb_sha256);
    let bytes = hex::decode(smcrypto::sm3::sm3_hash(&input))
        .map_err(|error| UpgradeError::SigningDigest(error.to_string()))?;
    bytes
        .try_into()
        .map_err(|_| UpgradeError::InvalidSigningDigestLength)
}
