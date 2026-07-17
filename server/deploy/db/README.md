# USB Control Database Resources

The SQL files in this directory are the release source of truth for device-side SQLite schema and seed data.

Runtime service code must not create tables or insert default data. For a direct DEB install, `postinst` calls `usb-control-updater finalize-install`, which invokes `/opt/usb-control/bin/usb-control-db-migrate` before `usb-control.service` starts. For an online installation, the updater executor invokes the same migrator after installing the DEB and before starting the service.

Execution order for a new database:

1. `migrations/0001_init.sql`
2. `seeds/0001_default_data.sql`

The two files are committed in one transaction and set `PRAGMA user_version = 1`. An existing non-empty database is never reinitialized. Later releases add a sequential migration only when the business schema actually changes; seeds are not rerun during an upgrade. `PRAGMA user_version` is the only schema version marker, and no migration history table is created.

`0001_default_data.sql` may contain the baseline `system_version = 1.0.0`; packaging scripts must not edit this SQL per release.

Database migrations never change `system_config.system_version`. The installed release metadata is stored in `/opt/usb-control/install-meta/release.json`. After the service starts and passes the health check, the direct-install finalizer writes the installed version to `system_config.system_version`; online upgrades commit the target version with compare-and-set after the same health check succeeds.
