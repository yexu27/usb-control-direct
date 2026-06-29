# USB Control Database Resources

The SQL files in this directory are the release source of truth for device-side SQLite schema and seed data.

Runtime service code must not create tables or insert default data. The deb `postinst` script runs `/opt/usb-control/bin/usb-control-db-migrate.sh`, which initializes or migrates `/var/lib/usb-control/device.db` before `usb-control.service` starts.

Execution order for a new database:

1. `migrations/0001_init.sql`
2. `seeds/0001_default_data.sql`
3. sync `system_config.system_version` from `/opt/usb-control/install-meta/VERSION`

`0001_default_data.sql` may contain the baseline `system_version = 1.0.0`; packaging scripts must not edit this SQL per release.
