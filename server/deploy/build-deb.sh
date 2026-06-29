#!/usr/bin/env bash
set -euo pipefail

fail() {
  echo "build-deb.sh: $*" >&2
  exit 1
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SERVER_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

VERSION="${1:-}"
test -n "$VERSION" || fail "usage: server/deploy/build-deb.sh <version>"

WORKSPACE_VERSION="$(
  sed -n '/^\[workspace.package\]/,/^\[/p' "$SERVER_DIR/Cargo.toml" |
    sed -n 's/^version[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' |
    head -n 1
)"

test "$WORKSPACE_VERSION" = "$VERSION" || fail "version mismatch: arg=$VERSION workspace=$WORKSPACE_VERSION"

command -v cargo >/dev/null 2>&1 || fail "cargo not found"
command -v dpkg-deb >/dev/null 2>&1 || fail "dpkg-deb not found"
command -v sha256sum >/dev/null 2>&1 || fail "sha256sum not found"
command -v openssl >/dev/null 2>&1 || fail "openssl not found"

TLS_CERT="$SCRIPT_DIR/release-assets/tls/server.crt"
TLS_KEY="$SCRIPT_DIR/release-assets/tls/server.key"
LICENSE_PUBKEY="$SCRIPT_DIR/release-assets/keys/license_verify.pub"
test -r "$TLS_CERT" || fail "missing TLS cert: $TLS_CERT"
test -r "$TLS_KEY" || fail "missing TLS key: $TLS_KEY"
test -r "$LICENSE_PUBKEY" || fail "missing license public key: $LICENSE_PUBKEY"
openssl x509 -in "$TLS_CERT" -noout >/dev/null || fail "invalid TLS cert: $TLS_CERT"
openssl pkey -in "$TLS_KEY" -noout >/dev/null || fail "invalid TLS key: $TLS_KEY"

BUILD_DIR="$SCRIPT_DIR/build"
ROOT_DIR="$BUILD_DIR/deb-root"
OUT_DIR="$BUILD_DIR/out"
PACKAGE_NAME="usb-control_${VERSION}_arm64"
DEB_PATH="$OUT_DIR/${PACKAGE_NAME}.deb"

rm -rf "$ROOT_DIR"
mkdir -p "$ROOT_DIR/DEBIAN" "$OUT_DIR"
mkdir -p "$ROOT_DIR/opt/usb-control/bin"
mkdir -p "$ROOT_DIR/opt/usb-control/install-meta"
mkdir -p "$ROOT_DIR/opt/usb-control/bundle/deps"
mkdir -p "$ROOT_DIR/opt/usb-control/bundle/clamav-db"
mkdir -p "$ROOT_DIR/etc/usb-control/tls"
mkdir -p "$ROOT_DIR/etc/usb-control/keys"
mkdir -p "$ROOT_DIR/etc/systemd/system"
mkdir -p "$ROOT_DIR/etc/udev/rules.d"
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
install -m 0644 "$SCRIPT_DIR/usb-control.service" "$ROOT_DIR/etc/systemd/system/usb-control.service"
install -m 0644 "$SCRIPT_DIR/scripts/99-usb-control.rules" "$ROOT_DIR/etc/udev/rules.d/99-usb-control.rules"
install -m 0755 "$SCRIPT_DIR/scripts/usb-control-otg-init.sh" "$ROOT_DIR/usr/local/bin/usb-control-otg-init.sh"
install -m 0755 "$SCRIPT_DIR/scripts/smoke.sh" "$ROOT_DIR/opt/usb-control/smoke.sh"

printf '%s\n' "$VERSION" > "$ROOT_DIR/opt/usb-control/install-meta/VERSION"

CERT_FP="$(openssl x509 -in "$TLS_CERT" -outform der | sha256sum | awk '{print toupper($1)}' | sed 's/../&:/g; s/:$//')"
{
  echo "package=usb-control"
  echo "version=$VERSION"
  echo "architecture=arm64"
  echo "rust_binary_sha256=$(sha256sum "$BINARY" | awk '{print $1}')"
  echo "tls_cert_sha256_fingerprint=$CERT_FP"
  echo "stage=1"
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

dpkg-deb --build --root-owner-group "$ROOT_DIR" "$DEB_PATH"
sha256sum "$DEB_PATH" > "$DEB_PATH.sha256"

echo "Deb: $DEB_PATH"
cat "$DEB_PATH.sha256"
echo "TLS cert SHA256 fingerprint: $CERT_FP"
