import { resolve } from 'node:path'
import { _electron as electron, expect, test } from '@playwright/test'

test('启动 Electron 并暴露最小 desktopApi', async () => {
  const electronApp = await electron.launch({
    args: [resolve(__dirname, '../../out/main/index.js')],
  })

  try {
    const page = await electronApp.firstWindow()
    await expect(page).toHaveURL(/#\/login$/)
    await expect(page.getByRole('heading', { name: 'USB安全管理系统' })).toBeVisible()
    await expect(page.getByTestId('login-username')).toBeVisible()
    await expect(page.getByTestId('login-password')).toBeVisible()
    await expect(page.getByTestId('login-ip')).toBeVisible()

    const apiShape = await page.evaluate(() => {
      const desktopApi = (
        globalThis as typeof globalThis & {
          desktopApi: {
            tls: Record<string, unknown>
            dialog: Record<string, unknown>
            window: Record<string, unknown>
          }
        }
      ).desktopApi
      return {
        namespaces: Object.keys(desktopApi).sort(),
        tlsMethods: Object.keys(desktopApi.tls).sort(),
        dialogMethods: Object.keys(desktopApi.dialog).sort(),
        windowMethods: Object.keys(desktopApi.window).sort(),
      }
    })

    expect(apiShape.namespaces).toEqual(['dialog', 'tls', 'window'])
    expect(apiShape.tlsMethods).toEqual([
      'applyStateEvent',
      'connect',
      'disconnect',
      'onStateChanged',
      'send',
    ])
    expect(apiShape.dialogMethods).toEqual(['openFile', 'readFile', 'saveFile', 'writeFile'])
    expect(apiShape.windowMethods).toEqual(['close', 'maximize', 'minimize'])
  } finally {
    await electronApp.close()
  }
})
