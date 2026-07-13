# Offline Bundle Inputs

Stage 2 does not package ClamAV offline dependencies.

ClamAV is installed independently on RK3568. The server deb validates the required executables, service, socket, and virus database files during installation.

Do not place `clamav*.deb`, `main.cvd`, `daily.cvd`, or `bytecode.cvd` here. The server deb must not install ClamAV or update virus databases.
