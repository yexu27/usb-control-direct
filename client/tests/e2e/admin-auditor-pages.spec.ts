import { mkdtemp, readFile, rm, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { basename, join, resolve } from 'node:path'
import { _electron as electron, expect, test, type ElectronApplication, type Page } from '@playwright/test'
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

async function login(page: Page, username: string, password: string): Promise<void> {
  await page.locator('[data-testid="login-username"]').fill(username)
  await page.locator('[data-testid="login-password"]').fill(password)
  await page.locator('[data-testid="login-ip"]').fill('127.0.0.1')
  await page.getByTestId('login-submit').click()
}

async function openMenu(page: Page, name: string): Promise<void> {
  await page.getByTestId('nav-item').filter({ hasText: name }).click()
}

async function expectLatestMessage(page: Page, text: string | RegExp): Promise<void> {
  await expect(page.locator('.el-message__content').filter({ hasText: text }).last()).toBeVisible()
}

async function expectAppDialog(page: Page, title: string | RegExp, message: string | RegExp): Promise<void> {
  const dialog = page.locator('.app-confirm-message-box')
  await expect(dialog).toBeVisible()
  await expect(dialog).toContainText(title)
  await expect(dialog).toContainText(message)
}

async function closeAppDialog(page: Page): Promise<void> {
  const dialog = page.locator('.app-confirm-message-box')
  await dialog.getByRole('button', { name: '确定' }).click()
  await expect(dialog).toBeHidden()
}

function progressDialog(page: Page) {
  return page.getByTestId('progress-dialog')
}

async function withDevice(
  run: (device: MockDevice, app: ElectronApplication, page: Page) => Promise<void>,
): Promise<void> {
  const device = new MockDevice({ ...DEFAULT_SCENARIO })
  let app: ElectronApplication | null = null
  await device.start()
  try {
    const launched = await launchApp()
    app = launched.app
    await run(device, launched.app, launched.page)
  } finally {
    await app?.close()
    await device.stop()
  }
}

test.describe('管理员与审计员页面业务闭环', () => {
  test('审计员进入日志管理并导出 PRD 英文文件名 ZIP 到中文路径', async () => {
    await withDevice(async (_device, app, page) => {
      const directory = await mkdtemp(join(tmpdir(), '日志导出-'))
      const exportPath = join(directory, 'USBUsageLog20260622120000.zip')
      try {
        await login(page, 'audit', 'audit@123')
        await expect(page).toHaveURL(/#\/logs$/)
        await expect(page.getByRole('heading', { name: '日志管理' })).toBeVisible()
        await expect(page.getByText('USB审计日志', { exact: true })).toBeVisible()

        await app.evaluate(({ dialog }, path) => {
          dialog.showSaveDialog = async () => ({ canceled: false, filePath: path })
        }, exportPath)
        await page.getByTestId('log-export').click()
        await expectLatestMessage(page, '日志导出成功')

        const exported = await readFile(exportPath)
        expect(basename(exportPath)).toBe('USBUsageLog20260622120000.zip')
        expect(exported.subarray(0, 4).toString('binary')).toBe('PK\u0003\u0004')
      } finally {
        await rm(directory, { recursive: true, force: true })
      }
    })
  })

  test('系统升级失败不掉线且成功升级后需要重新连接', async () => {
    await withDevice(async (_device, app, page) => {
      const directory = await mkdtemp(join(tmpdir(), 'system-upgrade-'))
      const failedPackage = join(directory, 'usb-control-system-v9.9.9.bin')
      const successPackage = join(directory, 'usb-control-system-v1.1.0.bin')
      try {
        await writeFile(failedPackage, Buffer.from('failed package'))
        await writeFile(successPackage, Buffer.from('success package'))
        await login(page, 'admin', 'admin@123')
        await expect(page).toHaveURL(/#\/users$/)
        await openMenu(page, '系统管理')
        await expect(page.getByRole('heading', { name: '系统管理' })).toBeVisible()

        await app.evaluate(({ dialog }, path) => {
          dialog.showOpenDialog = async () => ({ canceled: false, filePaths: [path] })
        }, failedPackage)
        await page.getByTestId('system-upgrade-upload').click()
        await expect(progressDialog(page)).toBeVisible()
        await expect(progressDialog(page)).toBeHidden()
        await expectAppDialog(page, '系统升级失败', '系统升级失败，已回滚至原版本')
        await closeAppDialog(page)
        await expect(page.getByTestId('connection-status')).toContainText('已连接')
        await expect(page.getByText(/当前版本: v1\.0\.0/)).toBeVisible()

        const systemButton = page.getByTestId('system-upgrade-upload')
        await expect(systemButton).not.toHaveClass(/is-loading/)
        await expect(systemButton).not.toBeFocused()

        await app.evaluate(({ dialog }, path) => {
          dialog.showOpenDialog = async () => ({ canceled: false, filePaths: [path] })
        }, successPackage)
        await systemButton.click()
        await expect(progressDialog(page)).toBeVisible()
        await expect(progressDialog(page)).toBeHidden()
        await expectAppDialog(page, '系统升级完成', '升级完成，请重新连接')
        await closeAppDialog(page)
        await expect(systemButton).not.toHaveClass(/is-loading/)
        await expect(systemButton).not.toBeFocused()
        await expect(page.getByTestId('connection-status')).toContainText('未连接')
      } finally {
        await rm(directory, { recursive: true, force: true })
      }
    })
  })

  test('病毒库升级选择文件后显示居中进度弹窗且至少保持 3 秒', async () => {
    await withDevice(async (_device, app, page) => {
      const directory = await mkdtemp(join(tmpdir(), 'virusdb-upgrade-'))
      const packagePath = join(directory, 'usb-control-virusdb-v3.0.5.zip')
      try {
        await writeFile(packagePath, Buffer.from('virusdb package'))
        await login(page, 'admin', 'admin@123')
        await expect(page).toHaveURL(/#\/users$/)
        await openMenu(page, '系统管理')
        await expect(page.getByRole('heading', { name: '系统管理' })).toBeVisible()

        await app.evaluate(({ dialog }, path) => {
          dialog.showOpenDialog = async () => ({ canceled: false, filePaths: [path] })
        }, packagePath)
        const startedAt = Date.now()
        await page.getByTestId('virusdb-upgrade-upload').click()
        await expect(progressDialog(page)).toBeVisible()
        await expect(progressDialog(page)).toContainText('病毒库升级')
        await expect(progressDialog(page)).toContainText('正在传输文件...')

        await page.waitForTimeout(2_800)
        await expect(progressDialog(page)).toBeVisible()
        await expectAppDialog(page, '病毒库升级完成', '病毒库升级成功')
        expect(Date.now() - startedAt).toBeGreaterThanOrEqual(3_000)
        await expect(progressDialog(page)).toBeHidden()
        await closeAppDialog(page)
        const virusdbButton = page.getByTestId('virusdb-upgrade-upload')
        await expect(virusdbButton).not.toHaveClass(/is-loading/)
        await expect(virusdbButton).not.toBeFocused()
      } finally {
        await rm(directory, { recursive: true, force: true })
      }
    })
  })

  test('升级包文件名不符合规则时提示错误且不进入上传状态', async () => {
    await withDevice(async (device, app, page) => {
      const directory = await mkdtemp(join(tmpdir(), 'invalid-upgrade-'))
      const invalidSystemPackage = join(directory, 'bad-system.bin')
      const invalidVirusdbPackage = join(directory, 'bad-virusdb.zip')
      try {
        await writeFile(invalidSystemPackage, Buffer.from('invalid system package'))
        await writeFile(invalidVirusdbPackage, Buffer.from('invalid virusdb package'))
        await login(page, 'admin', 'admin@123')
        await expect(page).toHaveURL(/#\/users$/)
        await openMenu(page, '系统管理')
        await expect(page.getByRole('heading', { name: '系统管理' })).toBeVisible()

        const systemButton = page.getByTestId('system-upgrade-upload')
        await app.evaluate(({ dialog }, path) => {
          dialog.showOpenDialog = async () => ({ canceled: false, filePaths: [path] })
        }, invalidSystemPackage)
        await systemButton.click()
        await expectAppDialog(page, '系统升级包文件名错误', /系统升级包文件名格式错误/)
        await expect(progressDialog(page)).toBeHidden()
        await closeAppDialog(page)
        await expect(systemButton).not.toHaveClass(/is-loading/)
        await expect(systemButton).not.toBeFocused()
        expect(device.systemUpgradeUploadCount).toBe(0)

        const virusdbButton = page.getByTestId('virusdb-upgrade-upload')
        await app.evaluate(({ dialog }, path) => {
          dialog.showOpenDialog = async () => ({ canceled: false, filePaths: [path] })
        }, invalidVirusdbPackage)
        await virusdbButton.click()
        await expectAppDialog(page, '病毒库升级包文件名错误', /病毒库升级包文件名格式错误/)
        await expect(progressDialog(page)).toBeHidden()
        await closeAppDialog(page)
        await expect(virusdbButton).not.toHaveClass(/is-loading/)
        await expect(virusdbButton).not.toBeFocused()
        expect(device.virusdbUpgradeUploadCount).toBe(0)
      } finally {
        await rm(directory, { recursive: true, force: true })
      }
    })
  })

  test('授权文件上传使用居中进度和结果弹窗', async () => {
    await withDevice(async (device, app, page) => {
      const directory = await mkdtemp(join(tmpdir(), 'license-upload-'))
      const invalidLicensePath = join(directory, 'license.bin')
      const validLicensePath = join(directory, 'license.txt')
      try {
        await writeFile(invalidLicensePath, Buffer.from('invalid license'))
        await writeFile(validLicensePath, Buffer.from('valid license'))
        await login(page, 'admin', 'admin@123')
        await expect(page).toHaveURL(/#\/users$/)
        await openMenu(page, '系统管理')
        await expect(page.getByRole('heading', { name: '系统管理' })).toBeVisible()

        const licenseButton = page.getByTestId('license-upload')
        await app.evaluate(({ dialog }, path) => {
          dialog.showOpenDialog = async () => ({ canceled: false, filePaths: [path] })
        }, invalidLicensePath)
        await licenseButton.click()
        await expectAppDialog(page, '授权文件格式错误', '仅支持 .txt 格式授权文件')
        await expect(progressDialog(page)).toBeHidden()
        await closeAppDialog(page)
        await expect(licenseButton).not.toHaveClass(/is-loading/)
        await expect(licenseButton).not.toBeFocused()
        expect(device.licenseUploadCount).toBe(0)

        await app.evaluate(({ dialog }, path) => {
          dialog.showOpenDialog = async () => ({ canceled: false, filePaths: [path] })
        }, validLicensePath)
        const startedAt = Date.now()
        await licenseButton.click()
        await expect(progressDialog(page)).toBeVisible()
        await expect(progressDialog(page)).toContainText('上传授权文件')
        await expect(progressDialog(page)).toContainText('正在传输文件...')
        await page.waitForTimeout(2_800)
        await expect(progressDialog(page)).toBeVisible()
        await expectAppDialog(page, '授权文件上传完成', '授权文件上传成功')
        expect(Date.now() - startedAt).toBeGreaterThanOrEqual(3_000)
        await expect(progressDialog(page)).toBeHidden()
        await closeAppDialog(page)
        await expect(licenseButton).not.toHaveClass(/is-loading/)
        await expect(licenseButton).not.toBeFocused()
        expect(device.licenseUploadCount).toBe(1)
      } finally {
        await rm(directory, { recursive: true, force: true })
      }
    })
  })

  test('用户管理支持创建用户、重置密码并保护内置账号删除入口', async () => {
    await withDevice(async (_device, _app, page) => {
      await login(page, 'admin', 'admin@123')
      await expect(page).toHaveURL(/#\/users$/)
      await expect(page.getByText('admin', { exact: true })).toBeVisible()
      await expect(page.getByTestId('delete-user-admin')).toHaveCount(0)

      await page.getByTestId('create-user-open').click()
      await page.getByTestId('create-username').fill('1')
      await page.getByTestId('create-password').fill('1')
      await page.getByTestId('create-confirm-password').fill('1')
      await page.getByTestId('create-user-submit').click()
      await expect(page.locator('.el-form-item__error').filter({ hasText: '密码长度不能少于8位' })).toBeVisible()
      await page.getByTestId('create-username').fill('new_operator')
      await page.getByTestId('create-role').click()
      await page.getByRole('option', { name: '操作员' }).click()
      await page.getByTestId('create-password').fill('NewPass@123')
      await page.getByTestId('create-confirm-password').fill('NewPass@123')
      await page.getByTestId('create-user-submit').click()
      await expectLatestMessage(page, '用户创建成功')
      await expect(page.getByText('new_operator', { exact: true })).toBeVisible()
      await page.getByTestId('delete-user-new_operator').click()
      const confirmBox = page.locator('.app-confirm-message-box')
      await expect(confirmBox).toContainText('是否要删除 new_operator 用户？')
      const confirmBounds = await confirmBox.boundingBox()
      const viewport = await page.evaluate(() => ({
        width: window.innerWidth,
        height: window.innerHeight,
      }))
      expect(confirmBounds).not.toBeNull()
      if (confirmBounds != null) {
        expect(confirmBounds.x).toBeGreaterThan(viewport.width * 0.25)
        expect(confirmBounds.x + confirmBounds.width).toBeLessThan(viewport.width * 0.75)
        expect(confirmBounds.y).toBeGreaterThan(viewport.height * 0.25)
        expect(confirmBounds.y + confirmBounds.height).toBeLessThan(viewport.height * 0.75)
      }
      await page.getByRole('button', { name: '取消' }).click()

      await page.getByTestId('user-menu-trigger').click()
      await page.getByText('登出', { exact: true }).click()
      await login(page, 'new_operator', 'NewPass@123')
      await expect(page).toHaveURL(/#\/file-access$/)

      await page.getByTestId('user-menu-trigger').click()
      await page.getByText('登出', { exact: true }).click()
      await login(page, 'admin', 'admin@123')
      await page.getByTestId('reset-password-new_operator').click()
      await page.getByTestId('reset-password').fill('Reset@123')
      await page.getByTestId('reset-confirm-password').fill('Reset@123')
      await page.getByTestId('reset-password-submit').click()
      await expectLatestMessage(page, '密码已重置')

      await page.getByTestId('user-menu-trigger').click()
      await page.getByText('登出', { exact: true }).click()
      await login(page, 'new_operator', 'NewPass@123')
      await expect(page.getByText('用户名或密码错误', { exact: true })).toBeVisible()
      await login(page, 'new_operator', 'Reset@123')
      await expect(page).toHaveURL(/#\/file-access$/)
    })
  })
})
