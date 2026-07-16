#!/usr/bin/env bash
set -euo pipefail

fail() {
  echo "build-bin.sh: $*" >&2
  exit 1
}

test "$#" -eq 3 || fail 'usage: server/deploy/build-bin.sh <target-version> <minimum-current-version> <schema-from>'
TARGET_VERSION="$1"
MINIMUM_VERSION="$2"
SCHEMA_FROM="$3"
for version in "$TARGET_VERSION" "$MINIMUM_VERSION"; do
  printf '%s' "$version" | grep -Eq '^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$' || fail "invalid version: $version"
done
printf '%s' "$SCHEMA_FROM" | grep -Eq '^[0-9]+$' || fail 'schema-from must be an unsigned integer'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SERVER_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_DIR="$(cd "$SERVER_DIR/.." && pwd)"
KEY_DIR="$REPO_DIR/deploy/assets/keys"
DEB="$SCRIPT_DIR/build/out/usb-control_V${TARGET_VERSION}_arm64.deb"
BIN="$SCRIPT_DIR/build/out/usb-control_V${TARGET_VERSION}_arm64.bin"
test -f "$DEB" || fail "target DEB not found: $DEB"
test "$(dpkg-deb --field "$DEB" Version)" = "$TARGET_VERSION" || fail 'DEB version mismatch'
for key in upgrade_sign.key upgrade_verify.pub upgrade_verify.id; do
  test -r "$KEY_DIR/$key" || fail "missing upgrade key material: $key"
done
rm -f "$BIN" "$BIN.sha256"

(
  cd "$SERVER_DIR"
  source "$HOME/.cargo/env" 2>/dev/null || true
  cargo run -p usb-control-release-tool --release -- build-bin \
    --deb "$DEB" \
    --key-dir "$KEY_DIR" \
    --output "$BIN" \
    --minimum-current-version "$MINIMUM_VERSION" \
    --schema-from "$SCHEMA_FROM"
  cargo run -p usb-control-release-tool --release -- verify-bin \
    --bin "$BIN" \
    --key-dir "$KEY_DIR"
)
bash "$SCRIPT_DIR/tests/bin-package-test.sh" "$BIN" "$DEB" "$TARGET_VERSION" "$SCHEMA_FROM"
sha256sum "$BIN" >"$BIN.sha256"
echo "BIN ready: $BIN"
