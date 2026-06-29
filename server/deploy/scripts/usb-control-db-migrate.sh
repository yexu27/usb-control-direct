#!/usr/bin/env bash
set -euo pipefail

DB_PATH="${1:-/var/lib/usb-control/device.db}"
SQL_ROOT="${2:-/opt/usb-control/db}"
VERSION_FILE="${3:-/opt/usb-control/install-meta/VERSION}"

fail() {
  echo "usb-control-db-migrate.sh: $*" >&2
  exit 1
}

test -x /opt/usb-control/bin/usb-control-db-migrate || fail "missing /opt/usb-control/bin/usb-control-db-migrate"
test -r "$SQL_ROOT/migrations/0001_init.sql" || fail "missing migration SQL: $SQL_ROOT/migrations/0001_init.sql"
test -r "$SQL_ROOT/seeds/0001_default_data.sql" || fail "missing seed SQL: $SQL_ROOT/seeds/0001_default_data.sql"
test -r "$VERSION_FILE" || fail "missing VERSION file: $VERSION_FILE"

install -d -m 0700 -o root -g root "$(dirname "$DB_PATH")"
/opt/usb-control/bin/usb-control-db-migrate "$DB_PATH" "$SQL_ROOT" "$VERSION_FILE"
chown root:root "$DB_PATH"
chmod 0600 "$DB_PATH"
