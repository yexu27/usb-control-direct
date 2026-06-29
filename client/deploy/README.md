# Client NSIS Packaging

Run from the repository root:

```powershell
powershell -ExecutionPolicy Bypass -File client/deploy/build-nsis.ps1
```

The script reads `deploy/assets/tls/server.crt.sha256`, validates it, sets `USB_CONTROL_CERT_FINGERPRINT` for the build process only, runs `npm ci`, builds the Electron/Vue app, and generates an NSIS installer under `client/dist`.

The installer contains only the management client runtime files required to execute on Windows. It does not contain server TLS private keys, service-side license keys, SM2/SM4 policy keys, ClamAV files, git metadata, default device IPs, usernames, passwords, tokens, or business data.

`client/node_modules`, `client/out`, and `client/dist` are generated locally and must not be committed.
