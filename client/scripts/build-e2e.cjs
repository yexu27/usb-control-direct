const { X509Certificate } = require('node:crypto')
const { readFileSync } = require('node:fs')
const { resolve } = require('node:path')
const { spawnSync } = require('node:child_process')

const certificatePath = resolve(__dirname, '../tests/e2e/fixtures/server.crt')
const certificate = new X509Certificate(readFileSync(certificatePath))
const fingerprint = certificate.fingerprint256.replaceAll(':', '').toLowerCase()
const npmCommand = process.platform === 'win32' ? 'npm.cmd' : 'npm'
const result = spawnSync(npmCommand, ['run', 'build'], {
  cwd: resolve(__dirname, '..'),
  env: { ...process.env, USB_CONTROL_CERT_FINGERPRINT: fingerprint },
  stdio: 'inherit',
})

if (result.error != null) {
  throw result.error
}
process.exit(result.status ?? 1)
