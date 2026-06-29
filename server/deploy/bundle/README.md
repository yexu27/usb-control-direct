# Offline Bundle Inputs

Stage 2 does not package ClamAV offline dependencies.

ClamAV is provided by the RK3568 factory image. The server deb validates the runtime contract from:

```text
deploy/assets/clamav/clamav-contract.toml
```

Do not place `clamav*.deb`, `main.cvd`, `daily.cvd`, or `bytecode.cvd` here. The server deb must not install ClamAV or update virus databases.
