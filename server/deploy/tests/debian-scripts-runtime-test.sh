#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"

if [ "${USB_CONTROL_TEST_MOUNT_NS:-0}" != '1' ]; then
  test "$(id -u)" = '0' || {
    echo 'debian-scripts-runtime-test: root is required for mount namespace test' >&2
    exit 1
  }
  exec unshare --mount --fork env USB_CONTROL_TEST_MOUNT_NS=1 REPO_ROOT="$ROOT" bash "$0"
fi

mount --make-rprivate /
WORK="$(mktemp -d)"
FAKE_BIN="$WORK/fake-bin"
LOG="$WORK/calls.log"
mkdir -p "$FAKE_BIN" "$WORK/opt" "$WORK/etc" "$WORK/lib" "$WORK/log" "$WORK/run-updater"
: >"$LOG"

cleanup() {
  for target in /run/usb-control-updater /var/log/usb-control /var/lib/usb-control /etc/usb-control /opt/usb-control; do
    mountpoint -q "$target" && umount "$target" || true
  done
  rm -rf "$WORK"
}
trap cleanup EXIT

for target in /opt/usb-control /etc/usb-control /var/lib/usb-control /var/log/usb-control /run/usb-control-updater; do
  mkdir -p "$target"
done
mount --bind "$WORK/opt" /opt/usb-control
mount --bind "$WORK/etc" /etc/usb-control
mount --bind "$WORK/lib" /var/lib/usb-control
mount --bind "$WORK/log" /var/log/usb-control
mount --bind "$WORK/run-updater" /run/usb-control-updater

cat >"$FAKE_BIN/systemctl" <<EOF
#!/usr/bin/env bash
printf 'systemctl %s\n' "\$*" >>'$LOG'
test ! -e '$WORK/fail-systemctl'
EOF
chmod 0755 "$FAKE_BIN/systemctl"
export PATH="$FAKE_BIN:/usr/sbin:/usr/bin:/sbin:/bin"

DEFAULTS="$WORK/opt/defaults/etc/usb-control"
mkdir -p "$DEFAULTS/tls" "$DEFAULTS/keys" "$WORK/opt/bin"
printf '#!/usr/bin/env bash\nexit 0\n' >"$WORK/opt/bin/usb-control"
printf '#!/usr/bin/env bash\nexit 0\n' >"$WORK/opt/bin/usb-control-db-migrate"
chmod 0755 "$WORK/opt/bin/usb-control" "$WORK/opt/bin/usb-control-db-migrate"
printf 'config-default\n' >"$DEFAULTS/usb-control.toml"
printf 'tls-cert-default\n' >"$DEFAULTS/tls/server.crt"
printf 'tls-key-default\n' >"$DEFAULTS/tls/server.key"
printf 'tls-sha-default\n' >"$DEFAULTS/tls/server.crt.sha256"
for name in license_verify.pub sm4_policy.key sm2_policy.key sm2_policy.pub; do
  printf '%s-default\n' "$name" >"$DEFAULTS/keys/$name"
done
printf 'upgrade-pub-v1\n' >"$DEFAULTS/keys/upgrade_verify.pub"
printf 'upgrade-id-v1\n' >"$DEFAULTS/keys/upgrade_verify.id"

cat >"$WORK/opt/bin/usb-control-updater" <<EOF
#!/usr/bin/env bash
printf 'updater %s\n' "\$*" >>'$LOG'
test ! -e '$WORK/fail-finalizer'
EOF
chmod 0755 "$WORK/opt/bin/usb-control-updater"

POSTINST="$REPO_ROOT/server/deploy/debian/postinst"
PRERM="$REPO_ROOT/server/deploy/debian/prerm"
POSTRM="$REPO_ROOT/server/deploy/debian/postrm"

reset_log() { : >"$LOG"; }
count_log() { grep -Fc "$1" "$LOG" || true; }
assert_no_clamav_calls() {
  if grep -Eqi 'clam|fresh|/var/lib/clamav' "$LOG"; then
    echo 'maintainer script unexpectedly managed ClamAV' >&2
    exit 1
  fi
}

reset_log
bash "$POSTINST" configure
test "$(count_log 'updater finalize-install')" = '1'
assert_no_clamav_calls
test "$(cat "$WORK/etc/usb-control.toml")" = 'config-default'
test "$(cat "$WORK/etc/tls/server.key")" = 'tls-key-default'
test "$(cat "$WORK/etc/keys/license_verify.pub")" = 'license_verify.pub-default'

printf 'site-config\n' >"$WORK/etc/usb-control.toml"
printf 'site-tls-key\n' >"$WORK/etc/tls/server.key"
printf 'site-license\n' >"$WORK/etc/keys/license_verify.pub"
printf 'site-policy\n' >"$WORK/etc/keys/sm4_policy.key"
printf 'upgrade-pub-v2\n' >"$DEFAULTS/keys/upgrade_verify.pub"
printf 'upgrade-id-v2\n' >"$DEFAULTS/keys/upgrade_verify.id"
touch "$WORK/run-updater/managed"
reset_log
bash "$POSTINST" configure
test "$(count_log 'updater finalize-install')" = '0'
assert_no_clamav_calls
test "$(cat "$WORK/etc/usb-control.toml")" = 'site-config'
test "$(cat "$WORK/etc/tls/server.key")" = 'site-tls-key'
test "$(cat "$WORK/etc/keys/license_verify.pub")" = 'site-license'
test "$(cat "$WORK/etc/keys/sm4_policy.key")" = 'site-policy'
test "$(cat "$WORK/etc/keys/upgrade_verify.pub")" = 'upgrade-pub-v2'
test "$(cat "$WORK/etc/keys/upgrade_verify.id")" = 'upgrade-id-v2'

reset_log
bash "$PRERM" upgrade
test "$(count_log 'systemctl stop')" = '0'
rm -f "$WORK/run-updater/managed"
reset_log
bash "$PRERM" upgrade
test "$(count_log 'systemctl stop usb-control.service')" = '1'

printf 'db\n' >"$WORK/lib/device.db"
printf 'log\n' >"$WORK/log/service.log"
bash "$POSTRM" remove
test -f "$WORK/etc/usb-control.toml"
test -f "$WORK/lib/device.db"
test -f "$WORK/log/service.log"
bash "$POSTRM" remove

bash "$POSTRM" purge
test -z "$(find "$WORK/etc" -mindepth 1 -print -quit)"
test -z "$(find "$WORK/lib" -mindepth 1 -print -quit)"
test -z "$(find "$WORK/log" -mindepth 1 -print -quit)"
bash "$POSTRM" purge

mkdir -p "$WORK/etc" "$WORK/lib" "$WORK/log"
rm -f "$WORK/run-updater/managed"
touch "$WORK/fail-finalizer"
if bash "$POSTINST" configure; then
  echo 'postinst unexpectedly ignored finalizer failure' >&2
  exit 1
fi
rm -f "$WORK/fail-finalizer"

echo 'debian-scripts-runtime-test: PASS'
