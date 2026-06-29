# Release TLS Inputs

Stage 1 deb packaging reads TLS release inputs from:

```text
server/deploy/release-assets/tls/server.crt
server/deploy/release-assets/tls/server.key
```

`server.key` must not be committed. The build script verifies both files exist, installs them into `/etc/usb-control/tls/`, sets `server.key` mode to `0600`, and writes the server certificate SHA256 fingerprint to `component-lock.txt`.
