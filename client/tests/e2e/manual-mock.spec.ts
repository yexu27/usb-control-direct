import { resolve } from 'node:path'
import { _electron as electron, test, type ElectronApplication } from '@playwright/test'
import { MockDevice } from './support/mock-device'

const APP_ENTRY = resolve(__dirname, '../../out/main/index.js')

test.describe('manual mock app', () => {
  test('keeps the admin mock app open for manual verification', async () => {
    test.setTimeout(0)

    const device = new MockDevice({
      authStatus: 'authorized',
      loginFailuresBeforeSuccess: 0,
      role: 'admin',
      uploadedLicenseValid: true,
    })
    let app: ElectronApplication | null = null

    await device.start()
    try {
      app = await electron.launch({ args: [APP_ENTRY] })
      await app.firstWindow()

      await new Promise(() => {
        // Keep the headed Electron window open until the user stops Playwright.
      })
    } finally {
      await app?.close()
      await device.stop()
    }
  })
})
