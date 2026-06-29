#!/usr/bin/env bash
set -euo pipefail

fail() {
  echo "[FAIL] $*" >&2
  exit 1
}

pass() {
  echo "[ OK ] $*"
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

echo "Smoke check passed."
