#!/usr/bin/env bash
set -euo pipefail

fail() {
  echo "build-deb.sh: $*" >&2
  exit 1
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SERVER_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_DIR="$(cd "$SERVER_DIR/.." && pwd)"
SHARED_ASSETS="$REPO_DIR/deploy/assets"

REQUESTED_VERSION="${1:-}"
REQUESTED_VERSION="$(printf '%s' "$REQUESTED_VERSION" | tr -d '\r')"
test -n "$REQUESTED_VERSION" || fail "usage: server/deploy/build-deb.sh <version>"
VERSION="${REQUESTED_VERSION#V}"
VERSION="${VERSION#v}"
DISPLAY_VERSION="V${VERSION}"

WORKSPACE_VERSION="$(
  sed -n '/^\[workspace.package\]/,/^\[/p' "$SERVER_DIR/Cargo.toml" |
    sed -n 's/^version[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' |
    head -n 1
)"

test "$WORKSPACE_VERSION" = "$VERSION" || fail "version mismatch: arg=$REQUESTED_VERSION normalized=$VERSION workspace=$WORKSPACE_VERSION"

command -v cargo >/dev/null 2>&1 || fail "cargo not found"
command -v dpkg-deb >/dev/null 2>&1 || fail "dpkg-deb not found"
command -v sha256sum >/dev/null 2>&1 || fail "sha256sum not found"
command -v openssl >/dev/null 2>&1 || fail "openssl not found"

TLS_CERT="$SHARED_ASSETS/tls/server.crt"
TLS_KEY="$SHARED_ASSETS/tls/server.key"
TLS_CERT_SHA256="$SHARED_ASSETS/tls/server.crt.sha256"
LICENSE_PUBKEY="$SHARED_ASSETS/keys/license_verify.pub"
SM4_POLICY_KEY="$SHARED_ASSETS/keys/sm4_policy.key"
SM2_POLICY_KEY="$SHARED_ASSETS/keys/sm2_policy.key"
SM2_POLICY_PUB="$SHARED_ASSETS/keys/sm2_policy.pub"
CLAMAV_CONTRACT="$SHARED_ASSETS/clamav/clamav-contract.toml"
test -r "$TLS_CERT" || fail "missing TLS cert: $TLS_CERT"
test -r "$TLS_KEY" || fail "missing TLS key: $TLS_KEY"
test -r "$TLS_CERT_SHA256" || fail "missing TLS cert sha256 file: $TLS_CERT_SHA256"
test -r "$LICENSE_PUBKEY" || fail "missing license public key: $LICENSE_PUBKEY"
test -r "$SM4_POLICY_KEY" || fail "missing SM4 policy key: $SM4_POLICY_KEY"
test -r "$SM2_POLICY_KEY" || fail "missing SM2 policy private key: $SM2_POLICY_KEY"
test -r "$SM2_POLICY_PUB" || fail "missing SM2 policy public key: $SM2_POLICY_PUB"
test -r "$CLAMAV_CONTRACT" || fail "missing ClamAV contract: $CLAMAV_CONTRACT"
openssl x509 -in "$TLS_CERT" -noout >/dev/null || fail "invalid TLS cert: $TLS_CERT"
openssl pkey -in "$TLS_KEY" -noout >/dev/null || fail "invalid TLS key: $TLS_KEY"

CERT_PUBKEY_SHA256="$(openssl x509 -in "$TLS_CERT" -pubkey -noout | openssl pkey -pubin -outform der | sha256sum | awk '{print $1}')"
KEY_PUBKEY_SHA256="$(openssl pkey -in "$TLS_KEY" -pubout -outform der | sha256sum | awk '{print $1}')"
test "$CERT_PUBKEY_SHA256" = "$KEY_PUBKEY_SHA256" || fail "TLS cert and key do not match"

CLIENT_CERT_FP="$(openssl x509 -in "$TLS_CERT" -outform der | sha256sum | awk '{print tolower($1)}')"
EXPECTED_CLIENT_CERT_FP="$(tr -d '[:space:]' < "$TLS_CERT_SHA256")"
printf '%s' "$EXPECTED_CLIENT_CERT_FP" | grep -Eq '^[0-9a-f]{64}$' || fail "TLS client fingerprint must be 64 lowercase hex characters"
test "$CLIENT_CERT_FP" = "$EXPECTED_CLIENT_CERT_FP" || fail "TLS fingerprint mismatch: generated=$CLIENT_CERT_FP file=$EXPECTED_CLIENT_CERT_FP"
test "$(wc -c < "$SM4_POLICY_KEY")" -eq 16 || fail "SM4 policy key must be 16 bytes: $SM4_POLICY_KEY"
tr -d '[:space:]' < "$SM2_POLICY_KEY" | grep -Eq '^[0-9A-Fa-f]{64}$' || fail "SM2 policy private key must be 64 hex characters"
tr -d '[:space:]' < "$SM2_POLICY_PUB" | grep -Eq '^[0-9A-Fa-f]{128}$' || fail "SM2 policy public key must be 128 hex characters"

BUILD_DIR="$SCRIPT_DIR/build"
ROOT_DIR="$BUILD_DIR/deb-root"
OUT_DIR="$BUILD_DIR/out"
PACKAGE_NAME="usb-control_${DISPLAY_VERSION}_arm64"
DEB_PATH="$OUT_DIR/${PACKAGE_NAME}.deb"

rm -rf "$ROOT_DIR"
mkdir -p "$ROOT_DIR/DEBIAN" "$OUT_DIR"
mkdir -p "$ROOT_DIR/opt/usb-control/bin"
mkdir -p "$ROOT_DIR/opt/usb-control/install-meta"
mkdir -p "$ROOT_DIR/etc/usb-control/tls"
mkdir -p "$ROOT_DIR/etc/usb-control/keys"
mkdir -p "$ROOT_DIR/etc/systemd/system"
mkdir -p "$ROOT_DIR/usr/local/bin"

echo "==> build rust release for aarch64"
(
  cd "$SERVER_DIR"
  cargo build --release --target aarch64-unknown-linux-gnu -p usb-control-app
)

BINARY="$SERVER_DIR/target/aarch64-unknown-linux-gnu/release/usb-control"
test -x "$BINARY" || fail "binary not found: $BINARY"

install -m 0755 "$BINARY" "$ROOT_DIR/opt/usb-control/bin/usb-control"
install -m 0644 "$SCRIPT_DIR/config/usb-control.toml" "$ROOT_DIR/etc/usb-control/usb-control.toml"
install -m 0644 "$TLS_CERT" "$ROOT_DIR/etc/usb-control/tls/server.crt"
install -m 0600 "$TLS_KEY" "$ROOT_DIR/etc/usb-control/tls/server.key"
install -m 0644 "$LICENSE_PUBKEY" "$ROOT_DIR/etc/usb-control/keys/license_verify.pub"
install -m 0600 "$SM4_POLICY_KEY" "$ROOT_DIR/etc/usb-control/keys/sm4_policy.key"
install -m 0600 "$SM2_POLICY_KEY" "$ROOT_DIR/etc/usb-control/keys/sm2_policy.key"
install -m 0644 "$SM2_POLICY_PUB" "$ROOT_DIR/etc/usb-control/keys/sm2_policy.pub"
install -m 0644 "$SCRIPT_DIR/usb-control.service" "$ROOT_DIR/etc/systemd/system/usb-control.service"
install -m 0755 "$SCRIPT_DIR/scripts/usb-control-otg-init.sh" "$ROOT_DIR/usr/local/bin/usb-control-otg-init.sh"
install -m 0755 "$SCRIPT_DIR/scripts/smoke.sh" "$ROOT_DIR/opt/usb-control/smoke.sh"
install -m 0644 "$CLAMAV_CONTRACT" "$ROOT_DIR/opt/usb-control/install-meta/clamav-contract.toml"

printf '%s\n' "$DISPLAY_VERSION" > "$ROOT_DIR/opt/usb-control/install-meta/VERSION"

CERT_FP="$(openssl x509 -in "$TLS_CERT" -outform der | sha256sum | awk '{print toupper($1)}' | sed 's/../&:/g; s/:$//')"
{
  echo "package=usb-control"
  echo "version=$DISPLAY_VERSION"
  echo "debian_version=$VERSION"
  echo "architecture=arm64"
  echo "rust_binary_sha256=$(sha256sum "$BINARY" | awk '{print $1}')"
  echo "tls_cert_sha256_fingerprint=$CERT_FP"
  echo "tls_client_fingerprint=$CLIENT_CERT_FP"
  echo "license_pubkey_sha256=$(sha256sum "$LICENSE_PUBKEY" | awk '{print $1}')"
  echo "sm4_policy_key_sha256=$(sha256sum "$SM4_POLICY_KEY" | awk '{print $1}')"
  echo "sm2_policy_key_sha256=$(sha256sum "$SM2_POLICY_KEY" | awk '{print $1}')"
  echo "sm2_policy_pub_sha256=$(sha256sum "$SM2_POLICY_PUB" | awk '{print $1}')"
  echo "clamav_contract_sha256=$(sha256sum "$CLAMAV_CONTRACT" | awk '{print $1}')"
  echo "stage=2"
  echo "clamav_bundle=not-included"
} > "$ROOT_DIR/opt/usb-control/install-meta/component-lock.txt"

sed "s/@VERSION@/$VERSION/g" "$SCRIPT_DIR/debian/control.template" > "$ROOT_DIR/DEBIAN/control"
install -m 0755 "$SCRIPT_DIR/debian/preinst" "$ROOT_DIR/DEBIAN/preinst"
install -m 0755 "$SCRIPT_DIR/debian/postinst" "$ROOT_DIR/DEBIAN/postinst"
install -m 0755 "$SCRIPT_DIR/debian/prerm" "$ROOT_DIR/DEBIAN/prerm"
install -m 0755 "$SCRIPT_DIR/debian/postrm" "$ROOT_DIR/DEBIAN/postrm"

find "$ROOT_DIR" -type d -exec chmod 0755 {} \;
chmod 0755 "$ROOT_DIR/DEBIAN/preinst" "$ROOT_DIR/DEBIAN/postinst" "$ROOT_DIR/DEBIAN/prerm" "$ROOT_DIR/DEBIAN/postrm"
chmod 0600 "$ROOT_DIR/etc/usb-control/tls/server.key"
chmod 0600 "$ROOT_DIR/etc/usb-control/keys/sm4_policy.key" "$ROOT_DIR/etc/usb-control/keys/sm2_policy.key"
chmod 0644 "$ROOT_DIR/etc/usb-control/keys/license_verify.pub" "$ROOT_DIR/etc/usb-control/keys/sm2_policy.pub"

dpkg-deb --build --root-owner-group "$ROOT_DIR" "$DEB_PATH"
sha256sum "$DEB_PATH" > "$DEB_PATH.sha256"

echo "Deb: $DEB_PATH"
cat "$DEB_PATH.sha256"
echo "TLS cert SHA256 fingerprint: $CERT_FP"
