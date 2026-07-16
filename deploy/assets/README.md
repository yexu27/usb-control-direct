# Shared Packaging Assets

This directory stores fixed release inputs shared by server deb packaging and client NSIS packaging.

Committed layout:

```text
deploy/assets/tls/server.crt
deploy/assets/tls/server.key
deploy/assets/tls/server.crt.sha256
deploy/assets/keys/license_sign.key
deploy/assets/keys/license_verify.pub
deploy/assets/keys/sm4_policy.key
deploy/assets/keys/sm2_policy.key
deploy/assets/keys/sm2_policy.pub
deploy/assets/keys/upgrade_sign.key
deploy/assets/keys/upgrade_verify.pub
deploy/assets/keys/upgrade_verify.id
```

`server.crt` and `server.key` are installed by the server deb to `/etc/usb-control/tls/`.
`server.crt.sha256` is the client certificate pinning input. Its format is 64 lowercase hex characters without colons.
`license_sign.key` is the SM2 private key used by the controlled development signing environment to issue device license files.
`license_verify.pub` is the raw production license verification public key consumed by `ProductionLicenseValidator::from_key_file`.
`sm4_policy.key`, `sm2_policy.key`, and `sm2_policy.pub` are consumed by the server policy import/export service through `FileKeyProvider`.
`upgrade_sign.key` is the SM2 private key used only by the controlled release tool to sign online upgrade containers. It is never included in a DEB or `.bin` file.
`upgrade_verify.pub` and `upgrade_verify.id` are the online upgrade verification trust root installed by the server DEB.
`server/deploy/build-bin.sh` uses these materials to create and immediately verify the three-entry signed online upgrade container.

ClamAV is installed independently on RK3568. The server deb does not bundle or install ClamAV packages or virus databases; its installation script validates the required ClamAV runtime components.

Packaging and installation scripts must not run `apt update`, `apt install`, `freshclam`, `wget`, or `curl` on the target device.
