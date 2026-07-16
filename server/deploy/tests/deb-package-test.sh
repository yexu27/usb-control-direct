#!/usr/bin/env bash
set -euo pipefail

fail() {
  echo "deb-package-test: $*" >&2
  exit 1
}

test "$#" -eq 2 || fail "usage: $0 <deb> <expected-version>"
DEB="$1"
EXPECTED_VERSION="$2"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
test -f "$DEB" || fail "DEB not found: $DEB"

test "$(dpkg-deb --field "$DEB" Package)" = 'usb-control' || fail 'wrong Package'
test "$(dpkg-deb --field "$DEB" Architecture)" = 'arm64' || fail 'wrong Architecture'
test "$(dpkg-deb --field "$DEB" Version)" = "$EXPECTED_VERSION" || fail 'wrong Version'
DEPENDS="$(dpkg-deb --field "$DEB" Depends)"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
dpkg-deb --extract "$DEB" "$TMP/data"
dpkg-deb --control "$DEB" "$TMP/control"

for binary in usb-control usb-control-updater usb-control-db-migrate; do
  path="$TMP/data/opt/usb-control/bin/$binary"
  test -f "$path" || fail "missing binary: $binary"
  readelf -hW "$path" | grep -Fq 'Class:                             ELF64' || fail "$binary is not ELF64"
  readelf -hW "$path" | grep -Fq 'Machine:                           AArch64' || fail "$binary is not AArch64"
  readelf -lW "$path" | grep -Eq 'Requesting program interpreter: /lib/ld-linux-aarch64\.so\.1' || fail "$binary has unexpected PT_INTERP"

  while read -r soname; do
    case "$soname" in
      libc.so.6|libm.so.6|libdl.so.2|libpthread.so.0|librt.so.1) package='libc6' ;;
      libgcc_s.so.1) package='libgcc-s1' ;;
      libudev.so.1) package='libudev1' ;;
      libstdc++.so.6) package='libstdc++6' ;;
      *) fail "$binary has unmapped DT_NEEDED: $soname" ;;
    esac
    printf '%s' "$DEPENDS" | grep -Eq "(^|, )[[:space:]]*$package([[:space:](,]|$)" || fail "Depends missing $package for $soname"
  done < <(readelf -dW "$path" | sed -n 's/.*Shared library: \[\([^]]*\)\].*/\1/p')

  while read -r symbol_version; do
    family="${symbol_version%%_*}"
    value="${symbol_version#*_}"
    case "$family" in
      GLIBC) maximum='2.35' ;;
      GLIBCXX) maximum='3.4.30' ;;
      CXXABI) maximum='1.3.13' ;;
      *) continue ;;
    esac
    test "$(printf '%s\n%s\n' "$value" "$maximum" | sort -V | tail -n 1)" = "$maximum" || fail "$binary requires unsupported $symbol_version"
  done < <(readelf --version-info "$path" | grep -Eo '(GLIBC|GLIBCXX|CXXABI)_[0-9]+(\.[0-9]+)+' | sort -u)
done

for required in \
  lib/systemd/system/usb-control.service \
  lib/systemd/system/usb-control-updater.service \
  opt/usb-control/install-meta/release.json \
  opt/usb-control/db/migrations/0001_init.sql \
  opt/usb-control/db/seeds/0001_default_data.sql \
  opt/usb-control/defaults/etc/usb-control/usb-control.toml \
  opt/usb-control/defaults/etc/usb-control/tls/server.crt \
  opt/usb-control/defaults/etc/usb-control/keys/upgrade_verify.pub \
  opt/usb-control/defaults/etc/usb-control/keys/upgrade_verify.id; do
  test -f "$TMP/data/$required" || fail "missing formal payload: $required"
done

if find "$TMP/data" -type f | grep -E '/(test|tests|fixture|fixtures|testdata)/|/(VERSION|component-lock\.txt|smoke([^/]*)|license_sign\.key|upgrade_sign\.key|[^/]*\.(deb|bin)|device\.db|[^/]*\.log)$' >/dev/null; then
  fail 'DEB contains forbidden release content'
fi

python3 - "$TMP/data/opt/usb-control/install-meta/release.json" "$EXPECTED_VERSION" <<'PY'
import json, sys
with open(sys.argv[1], encoding="utf-8") as source:
    value = json.load(source)
expected_keys = {
    "format_version", "product", "version", "architecture",
    "supported_schema_min", "supported_schema_max", "tls_cert_sha256",
    "upgrade_signing_key_id",
}
assert set(value) == expected_keys
assert value["version"] == sys.argv[2]
assert value["product"] == "usb-control"
assert value["architecture"] == "arm64"
PY

for dependency in clamav clamav-daemon clamav-freshclam; do
  printf '%s' "$DEPENDS" | grep -Eq "(^|, )[[:space:]]*$dependency([[:space:](,]|$)" || fail "Depends missing $dependency"
done

allowed_control='^(control|preinst|postinst|prerm|postrm|md5sums)$'
while read -r entry; do
  entry="${entry#./}"
  test -z "$entry" && continue
  printf '%s\n' "$entry" | grep -Eq "$allowed_control" || fail "unexpected control archive entry: $entry"
done < <(dpkg-deb --ctrl-tarfile "$DEB" | tar -tf -)

for script in preinst postinst prerm postrm; do
  test -f "$TMP/control/$script" || fail "missing maintainer script: $script"
  test "$(stat -c %a "$TMP/control/$script")" = '755' || fail "$script mode is not 0755"
  test "$(sha256sum "$TMP/control/$script" | awk '{print $1}')" = "$(sha256sum "$ROOT/server/deploy/debian/$script" | awk '{print $1}')" || fail "$script differs from repository source"
done

echo 'deb-package-test: PASS'
