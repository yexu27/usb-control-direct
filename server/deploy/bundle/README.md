# Offline Bundle Inputs

Stage 1 does not package full ClamAV offline dependencies.

Stage 2 will collect target RK3568-compatible files into:

```text
server/deploy/bundle/deps/*.deb
server/deploy/bundle/clamav-db/main.cvd
server/deploy/bundle/clamav-db/daily.cvd
server/deploy/bundle/clamav-db/bytecode.cvd
```

These binary release inputs are ignored by git and copied into the deb only when present.
