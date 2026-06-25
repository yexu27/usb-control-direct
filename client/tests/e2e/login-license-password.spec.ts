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
const LICENSE_FIXTURE = resolve(__dirname, 'fixtures/license.txt')
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

async function fillAndSubmitLogin(
  page: Page,
  username = 'e2e-user',
  password = 'Password1!',
): Promise<void> {
  await page.locator('[data-testid="login-username"]').fill(username)
  await page.locator('[data-testid="login-password"]').fill(password)
  await page.locator('[data-testid="login-ip"]').fill('127.0.0.1')
  await page.getByTestId('login-submit').click()
}

async function assertOwnershipLabelsAbsent(page: Page): Promise<void> {
  for (const label of ['操作员', '审计员', '系统管理员']) {
    await expect(page.getByText(label, { exact: true })).toHaveCount(0)
  }
}

async function withScenario(
  scenario: Partial<MockScenario>,
  run: (app: ElectronApplication, page: Page) => Promise<void>,
): Promise<void> {
  const device = new MockDevice({ ...DEFAULT_SCENARIO, ...scenario })
  let app: ElectronApplication | null = null
  await device.start()
  try {
    const launched = await launchApp()
    app = launched.app
    await run(launched.app, launched.page)
  } finally {
    await app?.close()
    await device.stop()
  }
}

test.describe('登录、授权与修改密码闭环', () => {
  for (const [role, route] of [
    ['admin', '/users'],
    ['operator', '/file-access'],
    ['auditor', '/logs'],
  ] as const) {
    test(`${role} 登录后进入默认页面`, async () => {
      await withScenario({ role }, async (_app, page) => {
        await fillAndSubmitLogin(page, `${role}-user`)
        await expect(page).toHaveURL(new RegExp(`#${route}$`))
        await expect(page.getByTestId('user-menu-trigger')).toBeVisible()
        await expect(page.getByTestId('user-menu-trigger')).toHaveText(`${role}-user`)
      })
    })
  }

  test('密码错误五次后显示账号锁定提示', async () => {
    await withScenario({ loginFailuresBeforeSuccess: 5 }, async (_app, page) => {
      for (let attempt = 1; attempt <= 5; attempt += 1) {
        await fillAndSubmitLogin(page)
        if (attempt < 5) {
          await expect(page.getByText('用户名或密码错误', { exact: true })).toBeVisible()
        }
      }
      await expect(page.getByText('用户已被锁定，请5分钟后重试', { exact: true })).toBeVisible()
    })
  })

  for (const role of ['admin', 'operator', 'auditor'] as const) {
    test(`${role} 在装置未授权时进入授权页`, async () => {
      await withScenario({ authStatus: 'unauthorized', role }, async (_app, page) => {
        await fillAndSubmitLogin(page)
        await expect(page).toHaveURL(/#\/license$/)
        await expect(page.getByRole('heading', { name: '设备授权' })).toBeVisible()
        await assertOwnershipLabelsAbsent(page)
      })
    })
  }

  test('授权到期可查看机器码并续期，完成后返回空白登录表单', async () => {
    await withScenario({ authStatus: 'expired' }, async (app, page) => {
      await fillAndSubmitLogin(page)
      await expect(page.getByRole('heading', { name: '授权已到期' })).toBeVisible()
      await assertOwnershipLabelsAbsent(page)

      await page.getByTestId('machine-code-open').click()
      await expect(page.locator('.machine-code-content textarea')).toHaveValue('USB-CONTROL-E2E-MACHINE-CODE')
      await expect(page.getByRole('img', { name: '机器码二维码' })).toBeVisible()
      await page.keyboard.press('Escape')

      await page.getByTestId('license-upload-open').click()
      await page.getByTestId('license-upload-confirm').click()
      await expect(page.getByText('请先选择授权文件', { exact: true })).toBeVisible()

      await app.evaluate(({ dialog }, licensePath) => {
        dialog.showOpenDialog = async () => ({ canceled: false, filePaths: [licensePath] })
      }, LICENSE_FIXTURE)
      await page.locator('.file-selector').click()
      await expect(page.locator('.el-dialog input[readonly]')).toHaveValue('license.txt')
      await page.getByTestId('license-upload-confirm').click()

      await expect(page).toHaveURL(/#\/login$/)
      await expect(page.locator('[data-testid="login-username"]')).toHaveValue('')
      await expect(page.locator('[data-testid="login-password"]')).toHaveValue('')
      await expect(page.locator('[data-testid="login-ip"]')).toHaveValue('')
      await assertOwnershipLabelsAbsent(page)
    })
  })

  test('未建立授权会话时直接访问授权页会返回登录页', async () => {
    const { app, page } = await launchApp()
    try {
      await page.evaluate(() => {
        window.location.hash = '#/license'
      })
      await expect(page).toHaveURL(/#\/login$/)
    } finally {
      await app.close()
    }
  })

  test('修改密码成功后关闭弹窗且保持当前登录页签', async () => {
    await withScenario({ role: 'admin' }, async (_app, page) => {
      await fillAndSubmitLogin(page, 'admin', 'admin@123')
      await expect(page).toHaveURL(/#\/users$/)

      await page.getByTestId('user-menu-trigger').click()
      await page.getByText('修改密码', { exact: true }).click()
      await page.getByPlaceholder('请输入旧密码').fill('admin@123')
      await page.getByPlaceholder('请输入新密码').fill('NewPassword2!')
      await page.getByPlaceholder('请再次输入新密码').fill('NewPassword2!')
      await page.getByRole('button', { name: '确定', exact: true }).click()

      await expect(page.getByRole('dialog', { name: '修改密码' })).toHaveCount(0)
      await expect(page.getByTestId('app-toast')).toHaveText('密码修改成功')
      await expect(page).toHaveURL(/#\/users$/)
    })
  })

  test('修改密码旧密码错误时在弹窗内显示红色错误提示', async () => {
    await withScenario({ role: 'admin' }, async (_app, page) => {
      await fillAndSubmitLogin(page, 'admin', 'admin@123')
      await expect(page).toHaveURL(/#\/users$/)

      await page.getByTestId('user-menu-trigger').click()
      await page.getByText('修改密码', { exact: true }).click()
      const dialog = page.getByRole('dialog', { name: '修改密码' })
      await expect(dialog).toBeVisible()

      await page.getByPlaceholder('请输入旧密码').fill('wrong@123')
      await page.getByPlaceholder('请输入新密码').fill('NewPassword2!')
      await page.getByPlaceholder('请再次输入新密码').fill('NewPassword2!')
      await page.getByRole('button', { name: '确定', exact: true }).click()

      const errorAlert = dialog.locator('.el-alert--error').filter({ hasText: '旧密码错误' })
      await expect(errorAlert).toBeVisible()
      await expect(dialog).toBeVisible()
    })
  })
})
