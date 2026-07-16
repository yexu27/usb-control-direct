#!/usr/bin/env bash
set -euo pipefail

fail() {
  echo "bin-package-test: $*" >&2
  exit 1
}

test "$#" -eq 4 || fail 'usage: bin-package-test.sh <bin> <deb> <target-version> <schema-from>'
BIN="$(realpath "$1")"
DEB="$(realpath "$2")"
TARGET_VERSION="$3"
SCHEMA_FROM="$4"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
KEY_DIR="$ROOT/deploy/assets/keys"
test -f "$BIN" || fail "BIN not found: $BIN"
test -f "$DEB" || fail "DEB not found: $DEB"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
tar -xf "$BIN" -C "$TMP"
cmp -s "$DEB" "$TMP/usb-control_V${TARGET_VERSION}_arm64.deb" || fail 'embedded DEB differs from standalone DEB'
dpkg-deb --extract "$DEB" "$TMP/deb"

for forbidden in \
  var/lib/usb-control/device.db \
  var/lib/usb-control/upgrade/current.json \
  var/lib/usb-control/upgrade/active-release.json \
  var/lib/usb-control/upgrade/history \
  var/lib/usb-control/upgrade/results \
  opt/usb-control/install-meta/VERSION; do
  test ! -e "$TMP/deb/$forbidden" || fail "embedded DEB contains runtime state: $forbidden"
done

python3 - "$BIN" "$DEB" "$TMP/manifest.json" "$TMP/deb/opt/usb-control/install-meta/release.json" "$TARGET_VERSION" "$SCHEMA_FROM" "$KEY_DIR/upgrade_sign.key" <<'PY'
import hashlib, json, pathlib, sys, tarfile
bin_path, deb_path, manifest_path, release_path, target, schema_from, private_path = sys.argv[1:]
with tarfile.open(bin_path, "r:") as archive:
    members = archive.getmembers()
    expected = ["manifest.json", f"usb-control_V{target}_arm64.deb", "signature.sm2"]
    assert [member.name for member in members] == expected
    for member in members:
        assert member.isfile()
        assert member.uid == 0 and member.gid == 0 and member.mtime == 0
        assert "key" not in member.name.lower()
with open(manifest_path, "rb") as source:
    manifest_raw = source.read()
manifest = json.loads(manifest_raw)
with open(release_path, encoding="utf-8") as source:
    release = json.load(source)
deb = pathlib.Path(deb_path).read_bytes()
assert manifest["format_version"] == 1
assert manifest["product"] == "usb-control"
assert manifest["package_version"] == target
assert manifest["architecture"] == "arm64"
assert manifest["deb_file"] == f"usb-control_V{target}_arm64.deb"
assert manifest["deb_size"] == len(deb)
assert manifest["deb_sha256"] == hashlib.sha256(deb).hexdigest()
assert manifest["schema_from"] == int(schema_from)
assert manifest["schema_to"] == release["supported_schema_max"]
assert manifest["tls_cert_sha256"] == release["tls_cert_sha256"]
private = pathlib.Path(private_path).read_bytes().strip()
assert private not in pathlib.Path(bin_path).read_bytes()
assert private not in deb
PY

(
  cd "$ROOT/server"
  source "$HOME/.cargo/env" 2>/dev/null || true
  cargo run -p usb-control-release-tool --release -- verify-bin --bin "$BIN" --key-dir "$KEY_DIR" >/dev/null
)
echo 'bin-package-test: PASS'
