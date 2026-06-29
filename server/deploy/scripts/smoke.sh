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
DB_PATH="/var/lib/usb-control/device.db"
DB_MIGRATE="/opt/usb-control/bin/usb-control-db-migrate"
DB_MIGRATE_SQL="/opt/usb-control/db/migrations/0001_init.sql"
DB_SEED_SQL="/opt/usb-control/db/seeds/0001_default_data.sql"

systemctl is-active --quiet "$SERVICE_NAME" || fail "systemd service is not active: $SERVICE_NAME"
pass "service active"

test -x "$BINARY" || fail "binary missing or not executable: $BINARY"
pass "binary executable"

test -x "$DB_MIGRATE" || fail "db migrate binary missing or not executable: $DB_MIGRATE"
test -r "$DB_MIGRATE_SQL" || fail "db migration SQL missing: $DB_MIGRATE_SQL"
test -r "$DB_SEED_SQL" || fail "db seed SQL missing: $DB_SEED_SQL"
pass "database migration resources present"

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

test -s "$DB_PATH" || fail "database missing or empty: $DB_PATH"
db_mode="$(stat -c '%a' "$DB_PATH")"
test "$db_mode" = "600" || fail "database mode must be 600, got $db_mode"
pass "database present and mode is 600"

if command -v sqlite3 >/dev/null 2>&1; then
  db_user_version="$(sqlite3 "$DB_PATH" "PRAGMA user_version;")"
  test "$db_user_version" = "1" || fail "database user_version must be 1, got $db_user_version"
  db_system_version="$(sqlite3 "$DB_PATH" "SELECT config_value FROM system_config WHERE config_key='system_version';")"
  package_version="$(tr -d '[:space:]' < /opt/usb-control/install-meta/VERSION)"
  test "$db_system_version" = "$package_version" || fail "database system_version mismatch: db=$db_system_version package=$package_version"
  pass "database schema and system version verified"

  clamav_version_output="$(/usr/bin/clamscan --version)"
  clamav_db_version="$(printf '%s\n' "$clamav_version_output" | awk -F/ '{print $2}')"
  clamav_db_time="$(printf '%s\n' "$clamav_version_output" | awk -F/ '{print $3}')"
  test -n "$clamav_db_version" || fail "failed to parse ClamAV database version from: $clamav_version_output"
  test -n "$clamav_db_time" || fail "failed to parse ClamAV database time from: $clamav_version_output"

  db_virus_version="$(sqlite3 "$DB_PATH" "SELECT config_value FROM system_config WHERE config_key='virus_db_version';")"
  db_virus_updated_at="$(sqlite3 "$DB_PATH" "SELECT config_value FROM system_config WHERE config_key='virus_db_updated_at';")"
  test "$db_virus_version" = "$clamav_db_version" || fail "database virus_db_version mismatch: db=$db_virus_version clamav=$clamav_db_version"
  test "$db_virus_updated_at" != "0" || fail "database virus_db_updated_at must not be 0"
  pass "database virus database status verified"
else
  echo "[WARN] sqlite3 not found, skip database content check"
fi

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
