# Release TLS Inputs

Stage 2 deb packaging reads TLS release inputs from the repository-level shared packaging assets:

```text
deploy/assets/tls/server.crt
deploy/assets/tls/server.key
deploy/assets/tls/server.crt.sha256
```

`server.crt` and `server.key` are committed because the server deb and client NSIS packaging must use the same release identity. The build script verifies both files exist, verifies the cert/key pair, verifies `server.crt.sha256`, installs them into `/etc/usb-control/tls/`, sets `server.key` mode to `0600`, and writes fingerprints to `component-lock.txt`.
