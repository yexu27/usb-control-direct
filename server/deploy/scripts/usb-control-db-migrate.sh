#!/usr/bin/env bash
set -euo pipefail

fail() {
  echo "usb-control-db-migrate.sh: $*" >&2
  exit 1
}

test "$#" -eq 2 || fail "usage: $0 <database-path> <sql-root>"

DB_PATH="$1"
SQL_ROOT="$2"

test -x /opt/usb-control/bin/usb-control-db-migrate || fail "missing /opt/usb-control/bin/usb-control-db-migrate"
test -r "$SQL_ROOT/migrations/0001_init.sql" || fail "missing migration SQL: $SQL_ROOT/migrations/0001_init.sql"
test -r "$SQL_ROOT/seeds/0001_default_data.sql" || fail "missing seed SQL: $SQL_ROOT/seeds/0001_default_data.sql"
install -d -m 0700 -o root -g root "$(dirname "$DB_PATH")"
/opt/usb-control/bin/usb-control-db-migrate "$DB_PATH" "$SQL_ROOT"
chown root:root "$DB_PATH"
chmod 0600 "$DB_PATH"
