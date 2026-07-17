#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
DEBIAN="$ROOT/server/deploy/debian"

fail() {
  echo "debian-scripts-test: $*" >&2
  exit 1
}

for script in preinst postinst prerm postrm; do
  bash -n "$DEBIAN/$script"
done

grep -Fq '/run/usb-control-updater/managed' "$DEBIAN/postinst" || fail 'postinst missing managed marker check'
grep -Fq '/opt/usb-control/bin/usb-control-updater finalize-install' "$DEBIAN/postinst" || fail 'postinst missing finalizer'
if grep -Eq '^[[:space:]]*/opt/usb-control/bin/usb-control-db-migrate([[:space:]]|$)' "$DEBIAN/postinst"; then
  fail 'postinst must not invoke db migrator directly'
fi
if grep -Eq 'freshclam|clamdscan|/var/lib/clamav|/run/clamav|clamav-daemon|systemctl .*clam' "$DEBIAN/postinst"; then
  fail 'postinst must not manage ClamAV or virus databases'
fi
grep -Fq '/run/usb-control-updater/managed' "$DEBIAN/prerm" || fail 'prerm missing managed marker check'
grep -Eq 'systemctl stop usb-control(\.service)?' "$DEBIAN/prerm" || fail 'prerm missing main service stop'

grep -Eq '^Depends:.*clamav([ ,]|$)' "$DEBIAN/control.template" || fail 'control missing clamav dependency'
grep -Eq '^Depends:.*clamav-daemon([ ,]|$)' "$DEBIAN/control.template" || fail 'control missing clamav-daemon dependency'
grep -Eq '^Depends:.*clamav-freshclam([ ,]|$)' "$DEBIAN/control.template" || fail 'control missing clamav-freshclam dependency'

grep -Fq 'RuntimeDirectory=usb-control-updater' "$ROOT/server/deploy/usb-control-updater.service" || fail 'updater unit missing RuntimeDirectory'
grep -Fq 'RuntimeDirectoryMode=0700' "$ROOT/server/deploy/usb-control-updater.service" || fail 'updater unit missing private RuntimeDirectoryMode'

if grep -Eq 'rm -rf /opt/usb-control|rm -f /etc/systemd/system/usb-control' "$DEBIAN/postrm"; then
  fail 'postrm must not delete dpkg-owned files'
fi
for path in /etc/usb-control /var/lib/usb-control /var/log/usb-control; do
  grep -Fq "$path" "$DEBIAN/postrm" || fail "postrm missing purge path $path"
done

echo 'debian-scripts-test: PASS'
