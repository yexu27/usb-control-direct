#!/usr/bin/env bash
set -euo pipefail

fail() {
  echo "build-deb.sh: $*" >&2
  exit 1
}

test "$#" -eq 1 || fail 'usage: server/deploy/build-deb.sh <major.minor.patch>'
VERSION="$1"
printf '%s' "$VERSION" | grep -Eq '^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$' || fail "invalid release version: $VERSION"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SERVER_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_DIR="$(cd "$SERVER_DIR/.." && pwd)"
ASSETS="$REPO_DIR/deploy/assets"
: "${SYSROOT:?SYSROOT is required}"
: "${TOOLCHAIN:?TOOLCHAIN is required}"
test -d "$SYSROOT" || fail "SYSROOT not found: $SYSROOT"
test -d "$TOOLCHAIN" || fail "TOOLCHAIN not found: $TOOLCHAIN"
LINKER="$TOOLCHAIN/aarch64-buildroot-linux-gnu-gcc"
READELF="$TOOLCHAIN/aarch64-buildroot-linux-gnu-readelf"
test -x "$LINKER" || fail "cross linker not found: $LINKER"
test -x "$READELF" || fail "cross readelf not found: $READELF"
for command in cargo dpkg-deb sha256sum openssl python3; do
  command -v "$command" >/dev/null 2>&1 || fail "$command not found"
done

TLS_CERT="$ASSETS/tls/server.crt"
TLS_KEY="$ASSETS/tls/server.key"
TLS_CERT_SHA256="$ASSETS/tls/server.crt.sha256"
LICENSE_PUBKEY="$ASSETS/keys/license_verify.pub"
SM4_POLICY_KEY="$ASSETS/keys/sm4_policy.key"
SM2_POLICY_KEY="$ASSETS/keys/sm2_policy.key"
SM2_POLICY_PUB="$ASSETS/keys/sm2_policy.pub"
UPGRADE_PUBKEY="$ASSETS/keys/upgrade_verify.pub"
UPGRADE_KEY_ID_FILE="$ASSETS/keys/upgrade_verify.id"
for path in "$TLS_CERT" "$TLS_KEY" "$TLS_CERT_SHA256" "$LICENSE_PUBKEY" "$SM4_POLICY_KEY" "$SM2_POLICY_KEY" "$SM2_POLICY_PUB" "$UPGRADE_PUBKEY" "$UPGRADE_KEY_ID_FILE"; do
  test -r "$path" || fail "missing release input: $path"
done

openssl x509 -in "$TLS_CERT" -noout >/dev/null || fail 'invalid TLS certificate'
openssl pkey -in "$TLS_KEY" -noout >/dev/null || fail 'invalid TLS private key'
CERT_PUBKEY_SHA256="$(openssl x509 -in "$TLS_CERT" -pubkey -noout | openssl pkey -pubin -outform der | sha256sum | awk '{print $1}')"
KEY_PUBKEY_SHA256="$(openssl pkey -in "$TLS_KEY" -pubout -outform der | sha256sum | awk '{print $1}')"
test "$CERT_PUBKEY_SHA256" = "$KEY_PUBKEY_SHA256" || fail 'TLS certificate and private key mismatch'
TLS_SHA256="$(openssl x509 -in "$TLS_CERT" -outform der | sha256sum | awk '{print tolower($1)}')"
test "$(tr -d '[:space:]' <"$TLS_CERT_SHA256")" = "$TLS_SHA256" || fail 'TLS fingerprint file mismatch'
test "$(wc -c <"$SM4_POLICY_KEY")" -eq 16 || fail 'SM4 policy key must be exactly 16 bytes'
tr -d '[:space:]' <"$SM2_POLICY_KEY" | grep -Eq '^[0-9A-Fa-f]{64}$' || fail 'invalid SM2 policy private key'
tr -d '[:space:]' <"$SM2_POLICY_PUB" | grep -Eq '^[0-9A-Fa-f]{128}$' || fail 'invalid SM2 policy public key'
UPGRADE_KEY_ID="$(tr -d '\r\n' <"$UPGRADE_KEY_ID_FILE")"
printf '%s' "$UPGRADE_KEY_ID" | grep -Eq '^[a-z0-9][a-z0-9-]{0,63}$' || fail 'invalid upgrade key id'
test "$(tr -d '\r\n' <"$UPGRADE_PUBKEY" | wc -c)" -eq 128 || fail 'upgrade public key must be 128 hex characters'
tr -d '\r\n' <"$UPGRADE_PUBKEY" | grep -Eq '^[0-9A-Fa-f]{128}$' || fail 'invalid upgrade public key'

echo '==> cross-build three ARM64 release binaries'
(
  cd "$SERVER_DIR"
  USB_CONTROL_RELEASE_VERSION="$VERSION" \
  CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="$LINKER" \
  PKG_CONFIG_ALLOW_CROSS=1 \
  PKG_CONFIG_SYSROOT_DIR="$SYSROOT" \
  PKG_CONFIG_PATH="$SYSROOT/usr/lib/pkgconfig" \
  RUSTFLAGS="-C link-args=--sysroot=$SYSROOT" \
  cargo build --release --target aarch64-unknown-linux-gnu \
    --bin usb-control --bin usb-control-updater --bin usb-control-db-migrate
)

TARGET_DIR="$SERVER_DIR/target/aarch64-unknown-linux-gnu/release"
BINARIES=(usb-control usb-control-updater usb-control-db-migrate)
declare -A RUNTIME_PACKAGES=()

version_not_newer_than() {
  test "$(printf '%s\n%s\n' "$1" "$2" | sort -V | tail -n 1)" = "$2"
}

for binary in "${BINARIES[@]}"; do
  path="$TARGET_DIR/$binary"
  test -x "$path" || fail "missing built binary: $path"
  "$READELF" -hW "$path" | grep -Fq 'Class:                             ELF64' || fail "$binary is not ELF64"
  "$READELF" -hW "$path" | grep -Fq 'Machine:                           AArch64' || fail "$binary is not AArch64"
  "$READELF" -lW "$path" | grep -Eq 'Requesting program interpreter: /lib/ld-linux-aarch64\.so\.1' || fail "$binary has unsupported interpreter"
  while read -r soname; do
    case "$soname" in
      libc.so.6|libm.so.6|libdl.so.2|libpthread.so.0|librt.so.1) package='libc6' ;;
      libgcc_s.so.1) package='libgcc-s1' ;;
      libudev.so.1) package='libudev1' ;;
      libstdc++.so.6) package='libstdc++6' ;;
      *) fail "$binary has unmapped DT_NEEDED: $soname" ;;
    esac
    RUNTIME_PACKAGES["$package"]=1
  done < <("$READELF" -dW "$path" | sed -n 's/.*Shared library: \[\([^]]*\)\].*/\1/p')
  while read -r symbol_version; do
    family="${symbol_version%%_*}"
    value="${symbol_version#*_}"
    case "$family" in
      GLIBC) maximum='2.35' ;;
      GLIBCXX) maximum='3.4.30' ;;
      CXXABI) maximum='1.3.13' ;;
      *) continue ;;
    esac
    version_not_newer_than "$value" "$maximum" || fail "$binary requires unsupported $symbol_version"
  done < <("$READELF" --version-info "$path" | grep -Eo '(GLIBC|GLIBCXX|CXXABI)_[0-9]+(\.[0-9]+)+' | sort -u)
done

RUNTIME_DEPENDS=''
while read -r package; do
  if [ -z "$RUNTIME_DEPENDS" ]; then
    RUNTIME_DEPENDS="$package"
  else
    RUNTIME_DEPENDS="$RUNTIME_DEPENDS, $package"
  fi
done < <(printf '%s\n' "${!RUNTIME_PACKAGES[@]}" | sort)
test -n "$RUNTIME_DEPENDS" || fail 'no runtime dependencies discovered'

BUILD_DIR="$SCRIPT_DIR/build"
ROOT_DIR="$BUILD_DIR/deb-root"
OUT_DIR="$BUILD_DIR/out"
DEB_PATH="$OUT_DIR/usb-control_V${VERSION}_arm64.deb"
rm -rf "$ROOT_DIR"
mkdir -p \
  "$ROOT_DIR/DEBIAN" \
  "$ROOT_DIR/opt/usb-control/bin" \
  "$ROOT_DIR/opt/usb-control/db/migrations" \
  "$ROOT_DIR/opt/usb-control/db/seeds" \
  "$ROOT_DIR/opt/usb-control/install-meta" \
  "$ROOT_DIR/opt/usb-control/defaults/etc/usb-control/tls" \
  "$ROOT_DIR/opt/usb-control/defaults/etc/usb-control/keys" \
  "$ROOT_DIR/lib/systemd/system" \
  "$OUT_DIR"

for binary in "${BINARIES[@]}"; do
  install -m 0755 "$TARGET_DIR/$binary" "$ROOT_DIR/opt/usb-control/bin/$binary"
done
install -m 0644 "$SCRIPT_DIR/db/migrations/0001_init.sql" "$ROOT_DIR/opt/usb-control/db/migrations/0001_init.sql"
install -m 0644 "$SCRIPT_DIR/db/seeds/0001_default_data.sql" "$ROOT_DIR/opt/usb-control/db/seeds/0001_default_data.sql"
DEFAULTS="$ROOT_DIR/opt/usb-control/defaults/etc/usb-control"
install -m 0640 "$SCRIPT_DIR/config/usb-control.toml" "$DEFAULTS/usb-control.toml"
install -m 0644 "$TLS_CERT" "$DEFAULTS/tls/server.crt"
install -m 0600 "$TLS_KEY" "$DEFAULTS/tls/server.key"
install -m 0644 "$TLS_CERT_SHA256" "$DEFAULTS/tls/server.crt.sha256"
install -m 0644 "$LICENSE_PUBKEY" "$DEFAULTS/keys/license_verify.pub"
install -m 0600 "$SM4_POLICY_KEY" "$DEFAULTS/keys/sm4_policy.key"
install -m 0600 "$SM2_POLICY_KEY" "$DEFAULTS/keys/sm2_policy.key"
install -m 0644 "$SM2_POLICY_PUB" "$DEFAULTS/keys/sm2_policy.pub"
install -m 0644 "$UPGRADE_PUBKEY" "$DEFAULTS/keys/upgrade_verify.pub"
install -m 0644 "$UPGRADE_KEY_ID_FILE" "$DEFAULTS/keys/upgrade_verify.id"
install -m 0644 "$SCRIPT_DIR/usb-control.service" "$ROOT_DIR/lib/systemd/system/usb-control.service"
install -m 0644 "$SCRIPT_DIR/usb-control-updater.service" "$ROOT_DIR/lib/systemd/system/usb-control-updater.service"

printf '{"format_version":1,"product":"usb-control","version":"%s","architecture":"arm64","supported_schema_min":1,"supported_schema_max":1,"tls_cert_sha256":"%s","upgrade_signing_key_id":"%s"}\n' \
  "$VERSION" "$TLS_SHA256" "$UPGRADE_KEY_ID" >"$ROOT_DIR/opt/usb-control/install-meta/release.json"

sed -e "s/@VERSION@/$VERSION/g" -e "s/@RUNTIME_DEPENDS@/$RUNTIME_DEPENDS/g" \
  "$SCRIPT_DIR/debian/control.template" >"$ROOT_DIR/DEBIAN/control"
for script in preinst postinst prerm postrm; do
  install -m 0755 "$SCRIPT_DIR/debian/$script" "$ROOT_DIR/DEBIAN/$script"
done
(
  cd "$ROOT_DIR"
  find . -path './DEBIAN' -prune -o -type f -print | LC_ALL=C sort | sed 's#^\./##' | xargs md5sum >DEBIAN/md5sums
)
chmod 0644 "$ROOT_DIR/DEBIAN/control" "$ROOT_DIR/DEBIAN/md5sums"

dpkg-deb --build --root-owner-group "$ROOT_DIR" "$DEB_PATH"
bash "$SCRIPT_DIR/tests/deb-package-test.sh" "$DEB_PATH" "$VERSION"
sha256sum "$DEB_PATH" >"$DEB_PATH.sha256"
echo "DEB ready: $DEB_PATH"
