#!/usr/bin/env bash
set -euo pipefail

fail() {
  echo "[FAIL] $*" >&2
  exit 1
}

pass() {
  echo "[ OK ] $*"
}

require_any_readable() {
  label="$1"
  shift

  for path in "$@"; do
    if [ -r "$path" ]; then
      return 0
    fi
  done

  fail "missing readable ClamAV database group: $label ($*)"
}

SERVICE_NAME="usb-control"
CONFIG_FILE="/etc/usb-control/usb-control.toml"
PRIVATE_KEY="/etc/usb-control/tls/server.key"
DATA_DIR="/var/lib/usb-control"
BINARY="/opt/usb-control/bin/usb-control"

systemctl is-active --quiet "$SERVICE_NAME" || fail "systemd service is not active: $SERVICE_NAME"
pass "service active"

test -x "$BINARY" || fail "binary missing or not executable: $BINARY"
pass "binary executable"

test -r "$CONFIG_FILE" || fail "config missing: $CONFIG_FILE"
pass "config readable"

test -f "/etc/usb-control/tls/server.crt" || fail "TLS cert missing"
test -f "$PRIVATE_KEY" || fail "TLS private key missing"

key_mode="$(stat -c '%a' "$PRIVATE_KEY")"
test "$key_mode" = "600" || fail "TLS private key mode must be 600, got $key_mode"
pass "TLS files present and private key mode is 600"

test -r /etc/usb-control/keys/license_verify.pub || fail "license verification public key missing"
test -r /etc/usb-control/keys/sm4_policy.key || fail "SM4 policy key missing"
test -r /etc/usb-control/keys/sm2_policy.key || fail "SM2 policy private key missing"
test -r /etc/usb-control/keys/sm2_policy.pub || fail "SM2 policy public key missing"

sm4_mode="$(stat -c '%a' /etc/usb-control/keys/sm4_policy.key)"
sm2_key_mode="$(stat -c '%a' /etc/usb-control/keys/sm2_policy.key)"
test "$sm4_mode" = "600" || fail "SM4 policy key mode must be 600, got $sm4_mode"
test "$sm2_key_mode" = "600" || fail "SM2 policy private key mode must be 600, got $sm2_key_mode"
test "$(wc -c < /etc/usb-control/keys/sm4_policy.key)" -eq 16 || fail "SM4 policy key must be 16 bytes"
tr -d '[:space:]' < /etc/usb-control/keys/sm2_policy.key | grep -Eq '^[0-9A-Fa-f]{64}$' || fail "SM2 policy private key must be 64 hex characters"
tr -d '[:space:]' < /etc/usb-control/keys/sm2_policy.pub | grep -Eq '^[0-9A-Fa-f]{128}$' || fail "SM2 policy public key must be 128 hex characters"
pass "service keys present and modes are valid"

data_mode="$(stat -c '%a' "$DATA_DIR")"
test "$data_mode" = "700" || fail "data dir mode must be 700, got $data_mode"
pass "data dir mode is 700"

listen_addr="$(awk -F '=' '/^listen_addr/ { gsub(/[ "]/, "", $2); print $2 }' "$CONFIG_FILE")"
listen_port="${listen_addr##*:}"
if command -v ss >/dev/null 2>&1; then
  ss -ltn | grep -q ":${listen_port} " || fail "listen port not found: $listen_port"
  pass "listen port active: $listen_port"
else
  echo "[WARN] ss not found, skip listen port check"
fi

systemctl is-active --quiet clamav-daemon.service || fail "clamav-daemon service is not active"
pass "clamav-daemon active"

test -x /usr/bin/clamdscan || fail "clamdscan missing or not executable"
/usr/bin/clamdscan --version >/dev/null 2>&1 || fail "clamdscan --version failed"
pass "clamdscan available"

test -S /run/clamav/clamd.ctl || fail "ClamAV daemon socket missing: /run/clamav/clamd.ctl"
pass "ClamAV daemon socket present"

require_any_readable "main" /var/lib/clamav/main.cvd /var/lib/clamav/main.cld
require_any_readable "daily" /var/lib/clamav/daily.cvd /var/lib/clamav/daily.cld
require_any_readable "bytecode" /var/lib/clamav/bytecode.cvd /var/lib/clamav/bytecode.cld
pass "ClamAV database groups readable"

eicar_file="$(mktemp /tmp/usb-control-eicar.XXXXXX)"
scan_output="$(mktemp /tmp/usb-control-clamdscan.XXXXXX)"
trap 'rm -f "$eicar_file" "$scan_output"' EXIT
printf '%s\n' 'X5O!P%@AP[4\PZX54(P^)7CC)7}$EICAR-STANDARD-ANTIVIRUS-TEST-FILE!$H+H*' > "$eicar_file"
chmod 0644 "$eicar_file"

set +e
/usr/bin/clamdscan --infected --no-summary "$eicar_file" >"$scan_output" 2>&1
scan_rc="$?"
set -e

if [ "$scan_rc" != "1" ]; then
  cat "$scan_output" >&2 || true
  fail "EICAR was not detected by ClamAV"
fi

grep -Eq 'Eicar|FOUND' "$scan_output" || {
  cat "$scan_output" >&2 || true
  fail "EICAR detection output did not include FOUND"
}
pass "EICAR detected"

echo "Smoke check passed."
