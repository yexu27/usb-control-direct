import { resolve } from 'node:path'
import {
  _electron as electron,
  expect,
  test,
  type ElectronApplication,
  type Page,
} from '@playwright/test'
import { MockDevice, type MockScenario } from './support/mock-device'

const APP_ENTRY = resolve(__dirname, '../../out/main/index.js')
const DEFAULT_SCENARIO: MockScenario = {
  authStatus: 'authorized',
  loginFailuresBeforeSuccess: 0,
  role: 'admin',
  uploadedLicenseValid: true,
}

async function launchApp(): Promise<{ app: ElectronApplication; page: Page }> {
  const app = await electron.launch({ args: [APP_ENTRY] })
  return { app, page: await app.firstWindow() }
}

async function login(page: Page): Promise<void> {
  await page.locator('[data-testid="login-username"]').fill('admin')
  await page.locator('[data-testid="login-password"]').fill('admin@123')
  await page.locator('[data-testid="login-ip"]').fill('127.0.0.1')
  await page.getByTestId('login-submit').click()
}

test.describe('管理端 UI 视觉契约', () => {
  test('登录页卡片宽度对齐原型尺寸', async () => {
    const { app, page } = await launchApp()
    try {
      const card = page.locator('.login-card')
      await expect(card).toBeVisible()
      const box = await card.boundingBox()
      expect(box).not.toBeNull()
      expect(Math.round(box?.width ?? 0)).toBe(420)
    } finally {
      await app.close()
    }
  })

  test('主窗口壳、顶栏、侧栏和页面切换保持固定窗口尺寸', async () => {
    const device = new MockDevice({ ...DEFAULT_SCENARIO })
    let app: ElectronApplication | null = null
    await device.start()
    try {
      const launched = await launchApp()
      app = launched.app
      const page = launched.page

      await login(page)
      await expect(page).toHaveURL(/#\/users$/)

      const before = await page.evaluate(() => ({
        width: window.innerWidth,
        height: window.innerHeight,
      }))

      const shell = page.locator('.app-window-shell')
      const header = page.locator('.main-header')
      const sidebar = page.locator('.main-sidebar')
      const content = page.locator('.main-content')

      await expect(shell).toBeVisible()
      await expect(header).toBeVisible()
      await expect(sidebar).toBeVisible()
      await expect(content).toBeVisible()

      const shellBox = await shell.boundingBox()
      const headerBox = await header.boundingBox()
      const sidebarBox = await sidebar.boundingBox()
      expect(shellBox).not.toBeNull()
      expect(headerBox).not.toBeNull()
      expect(sidebarBox).not.toBeNull()
      expect(Math.round(shellBox?.width ?? 0)).toBeLessThanOrEqual(1120)
      expect(Math.round(headerBox?.height ?? 0)).toBe(48)
      expect(Math.round(sidebarBox?.width ?? 0)).toBe(190)

      for (const menu of ['系统管理', '用户管理']) {
        await page.getByTestId('nav-item').filter({ hasText: menu }).click()
        const after = await page.evaluate(() => ({
          width: window.innerWidth,
          height: window.innerHeight,
        }))
        expect(after).toEqual(before)
      }

      await expect(content).toHaveCSS('overflow-y', /auto|scroll/)
    } finally {
      await app?.close()
      await device.stop()
    }
  })
})
