const { X509Certificate } = require('node:crypto')
const { readFileSync } = require('node:fs')
const { resolve } = require('node:path')
const { spawnSync } = require('node:child_process')

const certificatePath = resolve(__dirname, '../tests/e2e/fixtures/server.crt')
const usbDevicesPath = resolve(__dirname, '../tests/e2e/fixtures/management-usb-devices.json')
const certificate = new X509Certificate(readFileSync(certificatePath))
const fingerprint = certificate.fingerprint256.replaceAll(':', '').toLowerCase()
const usbDevicesJson = readFileSync(usbDevicesPath, 'utf8')
let usbDevices
try {
  usbDevices = JSON.parse(usbDevicesJson)
} catch (error) {
  throw new Error(`E2E USB fixture 不是合法 JSON: ${error.message}`)
}
if (!Array.isArray(usbDevices) || usbDevices.some((device) =>
  device == null || typeof device !== 'object' || typeof device.serialNumber !== 'string' ||
  typeof device.vid !== 'string' || typeof device.pid !== 'string' ||
  typeof device.deviceName !== 'string' || !Number.isFinite(device.capacityBytes) ||
  device.capacityBytes < 0 ||
  device.deviceType !== 'storage' || typeof device.addable !== 'boolean' ||
  typeof device.unavailableReason !== 'string'
)) {
  throw new Error('E2E USB fixture 不符合 ManagementUsbDevice[]')
}
const buildCommand =
  process.platform === 'win32'
    ? { command: 'cmd.exe', args: ['/d', '/s', '/c', 'npm run build'] }
    : { command: 'npm', args: ['run', 'build'] }
const result = spawnSync(buildCommand.command, buildCommand.args, {
  cwd: resolve(__dirname, '..'),
  env: {
    ...process.env,
    USB_CONTROL_CERT_FINGERPRINT: fingerprint,
    USB_CONTROL_E2E_USB_DEVICES: JSON.stringify(usbDevices),
  },
  stdio: 'inherit',
})

if (result.error != null) {
  throw result.error
}
process.exit(result.status ?? 1)
