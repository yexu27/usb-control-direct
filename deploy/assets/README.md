# Shared Packaging Assets

This directory stores fixed release inputs shared by server deb packaging and client NSIS packaging.

Committed layout:

```text
deploy/assets/tls/server.crt
deploy/assets/tls/server.key
deploy/assets/tls/server.crt.sha256
deploy/assets/keys/license_verify.pub
deploy/assets/keys/sm4_policy.key
deploy/assets/keys/sm2_policy.key
deploy/assets/keys/sm2_policy.pub
deploy/assets/clamav/clamav-contract.toml
```

`server.crt` and `server.key` are installed by the server deb to `/etc/usb-control/tls/`.
`server.crt.sha256` is the client certificate pinning input. Its format is 64 lowercase hex characters without colons.
`license_verify.pub` is the raw production license verification public key consumed by `ProductionLicenseValidator::from_key_file`.
`sm4_policy.key`, `sm2_policy.key`, and `sm2_policy.pub` are consumed by the server policy import/export service through `FileKeyProvider`.

ClamAV is provided by the RK3568 factory image. The server deb does not bundle or install ClamAV packages or virus databases; it only validates the runtime contract in `deploy/assets/clamav/clamav-contract.toml`.

Packaging and installation scripts must not run `apt update`, `apt install`, `freshclam`, `wget`, or `curl` on the target device.
