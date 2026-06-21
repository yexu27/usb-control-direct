import { resolve } from 'node:path'
import { _electron as electron, expect, test } from '@playwright/test'

test('启动 Electron 并暴露最小 desktopApi', async () => {
  const electronApp = await electron.launch({
    args: [resolve(__dirname, '../../out/main/index.js')],
  })

  try {
    const page = await electronApp.firstWindow()
    await expect(page).toHaveURL(/#\/login$/)
    await expect(page.getByText('登录页面')).toBeVisible()

    const apiShape = await page.evaluate(() => {
      const desktopApi = (
        globalThis as typeof globalThis & {
          desktopApi: {
            tls: Record<string, unknown>
            dialog: Record<string, unknown>
          }
        }
      ).desktopApi
      return {
        namespaces: Object.keys(desktopApi).sort(),
        tlsMethods: Object.keys(desktopApi.tls).sort(),
        dialogMethods: Object.keys(desktopApi.dialog).sort(),
      }
    })

    expect(apiShape.namespaces).toEqual(['dialog', 'tls'])
    expect(apiShape.tlsMethods).toEqual([
      'applyStateEvent',
      'connect',
      'disconnect',
      'onStateChanged',
      'send',
    ])
    expect(apiShape.dialogMethods).toEqual(['openFile', 'readFile', 'saveFile', 'writeFile'])
  } finally {
    await electronApp.close()
  }
})
