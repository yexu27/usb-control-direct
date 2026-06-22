import { mkdtemp, readFile, readdir, rm, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { basename, join, resolve } from 'node:path'
import { _electron as electron, expect, test, type ElectronApplication, type Page } from '@playwright/test'
import { MockDevice, type MockScenario } from './support/mock-device'

const APP_ENTRY = resolve(__dirname, '../../out/main/index.js')
const IMPORT_FIXTURE = resolve(__dirname, 'fixtures/import-policy.bin')
const DEFAULT_SCENARIO: MockScenario = {
  authStatus: 'authorized', loginFailuresBeforeSuccess: 0, role: 'operator', uploadedLicenseValid: true,
}

async function login(page: Page): Promise<void> {
  await page.locator('[data-testid="login-username"]').fill('operator-user')
  await page.locator('[data-testid="login-password"]').fill('Password1!')
  await page.locator('[data-testid="login-ip"]').fill('127.0.0.1')
  await page.getByTestId('login-submit').click()
  await expect(page).toHaveURL(/#\/file-access$/)
}

async function openMenu(page: Page, name: string): Promise<void> {
  await page.getByTestId('nav-item').filter({ hasText: name }).click()
}

async function chooseCandidate(page: Page, serial: string): Promise<void> {
  await page.getByTestId('candidate-select').click()
  await page.getByRole('option', { name: new RegExp(serial) }).click()
}

async function expectLatestMessage(page: Page, text: string | RegExp): Promise<void> {
  await expect(page.locator('.el-message__content').filter({ hasText: text }).last()).toBeVisible()
}

async function withOperator(
  run: (device: MockDevice, app: ElectronApplication, page: Page) => Promise<void>,
): Promise<void> {
  const device = new MockDevice({ ...DEFAULT_SCENARIO })
  let app: ElectronApplication | null = null
  await device.start()
  try {
    app = await electron.launch({ args: [APP_ENTRY] })
    const page = await app.firstWindow()
    await login(page)
    await run(device, app, page)
  } finally {
    await app?.close()
    await device.stop()
  }
}

test.describe('操作员三页面业务闭环', () => {
  test('仅显示三个菜单且越权 /users 重定向到文件访问控制', async () => {
    await withOperator(async (_device, _app, page) => {
      await expect(page.getByTestId('nav-item')).toHaveText(['文件访问控制', 'U盘设备控制', '策略管理'])
      await page.evaluate(() => { window.location.hash = '#/users' })
      await expect(page).toHaveURL(/#\/file-access$/)
    })
  })

  test('三个开关逐次成功并在重进页面后保持状态', async () => {
    await withOperator(async (_device, _app, page) => {
      for (const testId of ['exec-control-switch', 'auto-read-control-switch', 'blacklist-control-switch']) {
        await page.getByTestId(testId).click()
        await expectLatestMessage(page, '修改成功，重新拔插或重新映射后生效')
        await expect(page.getByTestId(testId)).toHaveClass(/is-checked/)
      }
      await openMenu(page, 'U盘设备控制')
      await openMenu(page, '文件访问控制')
      for (const testId of ['exec-control-switch', 'auto-read-control-switch', 'blacklist-control-switch']) {
        await expect(page.getByTestId(testId)).toHaveClass(/is-checked/)
      }
    })
  })

  test('.PS1 规范化为 .ps1、重复提示且默认 .jse 可删除', async () => {
    await withOperator(async (_device, _app, page) => {
      await page.getByTestId('add-blacklist-trigger').click()
      await page.locator('[data-testid="blacklist-extension-input"]').fill('.PS1')
      await page.getByTestId('blacklist-submit').click()
      await expect(page.getByText('.ps1', { exact: true })).toBeVisible()
      await page.getByTestId('add-blacklist-trigger').click()
      await page.locator('[data-testid="blacklist-extension-input"]').fill('.ps1')
      await page.getByTestId('blacklist-submit').click()
      await expectLatestMessage(page, '该文件后缀已在黑名单中')
      await page.keyboard.press('Escape')
      await page.locator('button[data-extension=".jse"]').click()
      await page.getByLabel('删除确认').getByRole('button', { name: '删除', exact: true }).click()
      await expect(page.getByText('.jse', { exact: true })).toHaveCount(0)
    })
  })

  test('装置端候选可添加且提交前移除返回精确错误', async () => {
    await withOperator(async (device, _app, page) => {
      await openMenu(page, 'U盘设备控制')
      await page.getByTestId('add-device-trigger').click()
      await chooseCandidate(page, 'DEVICE-ADDABLE-001')
      await page.getByTestId('whitelist-add-submit').click()
      await expect(page.getByText('DEVICE-ADDABLE-001', { exact: true })).toBeVisible()
      await page.getByTestId('add-device-trigger').click()
      await chooseCandidate(page, 'DEVICE-REMOVED-002')
      device.removeConnectedDevice('DEVICE-REMOVED-002')
      await page.getByTestId('whitelist-add-submit').click()
      await expectLatestMessage(page, '设备已移除，请重新插入后再添加')
    })
  })

  test('管理端候选仅点击后显示，空序列号禁用且正常设备按 management 添加', async () => {
    await withOperator(async (_device, _app, page) => {
      await openMenu(page, 'U盘设备控制')
      await expect(page.getByText('Management USB', { exact: false })).toHaveCount(0)
      await page.getByTestId('add-management-trigger').click()
      await page.getByTestId('candidate-select').click()
      await expect(page.getByRole('option', { name: /Broken serial USB/ })).toBeDisabled()
      await page.getByRole('option', { name: /MANAGEMENT-USB-001/ }).click()
      await page.getByTestId('whitelist-add-submit').click()
      await expect(page.getByText('MANAGEMENT-USB-001', { exact: true })).toBeVisible()
      await expect(page.getByRole('row', { name: /MANAGEMENT-USB-001.*管理端添加/ })).toBeVisible()
    })
  })

  test('白名单修改与删除后列表刷新', async () => {
    await withOperator(async (_device, _app, page) => {
      await openMenu(page, 'U盘设备控制')
      await page.getByTestId('edit-WL-EXISTING-001').click()
      await page.locator('[data-testid="whitelist-edit-description-input"]').fill('已修改')
      await page.getByText('读写', { exact: true }).last().click()
      await page.getByTestId('whitelist-edit-submit').click()
      await expect(page.getByText('已修改', { exact: true })).toBeVisible()
      await page.getByTestId('remove-WL-EXISTING-001').click()
      await page.getByLabel('删除确认').getByRole('button', { name: '删除', exact: true }).click()
      await expect(page.getByText('WL-EXISTING-001', { exact: true })).toHaveCount(0)
    })
  })

  test('导出使用默认文件名并写入临时目录', async () => {
    await withOperator(async (_device, app, page) => {
      const directory = await mkdtemp(join(tmpdir(), 'usb-policy-e2e-'))
      let exportedPath = ''
      try {
        await openMenu(page, '策略管理')
        await app.evaluate(({ dialog }, dir) => {
          dialog.showSaveDialog = async (_window, options) => ({
            canceled: false, filePath: `${dir}/${String(options.defaultPath)}`,
          })
        }, directory)
        await page.getByTestId('export-policy').click()
        await expectLatestMessage(page, '策略导出成功')
        exportedPath = (await readdir(directory)).map((name) => join(directory, name))[0]
        expect(basename(exportedPath)).toMatch(/^安全策略-\d{8}-\d{6}\.bin$/)
        const exported = await readFile(exportedPath)
        expect(exported.subarray(0, 10).toString('ascii')).toBe('USBPOLICY\n')
        expect(exported.subarray(10, 20).toString('ascii')).toBe('VERSION:1\n')
        expect(JSON.parse(exported.subarray(20).toString('utf8'))).toMatchObject({
          whitelist: [{ serialNumber: 'WL-EXISTING-001' }],
          filePolicy: { execControlEnabled: false },
        })
      } finally {
        await rm(directory, { recursive: true, force: true })
      }
    })
  })

  test('非 bin 文件在读取前拒绝', async () => {
    await withOperator(async (_device, app, page) => {
      await openMenu(page, '策略管理')
      await app.evaluate(({ dialog }) => {
        dialog.showOpenDialog = async () => ({ canceled: false, filePaths: ['/does-not-exist.txt'] })
      })
      await page.getByTestId('import-policy').click()
      await expect(page.getByText('仅支持 .bin 策略文件', { exact: true })).toBeVisible()
      await expect(page.getByText('策略文件读取失败', { exact: true })).toHaveCount(0)
    })
  })

  test('导入失败不改变状态且确定性 fixture 可成功导入', async () => {
    await withOperator(async (device, app, page) => {
      device.failNextImport()
      await openMenu(page, '策略管理')
      await app.evaluate(({ dialog }, fixture) => {
        dialog.showOpenDialog = async () => ({ canceled: false, filePaths: [fixture] })
      }, IMPORT_FIXTURE)
      await page.getByTestId('import-policy').click()
      await page.getByRole('button', { name: '导入', exact: true }).click()
      await expectLatestMessage(page, '策略导入失败，原策略未变更')
      await openMenu(page, 'U盘设备控制')
      await expect(page.getByText('WL-EXISTING-001', { exact: true })).toBeVisible()
      await expect(page.getByText('IMPORTED-USB-001', { exact: true })).toHaveCount(0)
      await openMenu(page, '策略管理')
      await page.getByTestId('import-policy').click()
      await page.getByRole('button', { name: '导入', exact: true }).click()
      await expectLatestMessage(page, /策略已导入/)
      await openMenu(page, 'U盘设备控制')
      await expect(page.getByText('IMPORTED-USB-001', { exact: true })).toBeVisible()
    })
  })

  test('内容格式错误的 bin 文件被拒绝且原策略不变', async () => {
    await withOperator(async (_device, app, page) => {
      const directory = await mkdtemp(join(tmpdir(), 'usb-policy-invalid-'))
      const invalidPath = join(directory, 'invalid.bin')
      try {
        await writeFile(invalidPath, Buffer.from('USBPOLICY\nVERSION:99\n{}', 'utf8'))
        await openMenu(page, '策略管理')
        await app.evaluate(({ dialog }, path) => {
          dialog.showOpenDialog = async () => ({ canceled: false, filePaths: [path] })
        }, invalidPath)
        await page.getByTestId('import-policy').click()
        await page.getByRole('button', { name: '导入', exact: true }).click()
        await expect(page.getByText('策略文件格式错误', { exact: true })).toBeVisible()
        await openMenu(page, 'U盘设备控制')
        await expect(page.getByText('WL-EXISTING-001', { exact: true })).toBeVisible()
      } finally {
        await rm(directory, { recursive: true, force: true })
      }
    })
  })

  test('导出后修改再导入可恢复白名单与文件策略', async () => {
    await withOperator(async (_device, app, page) => {
      const directory = await mkdtemp(join(tmpdir(), 'usb-policy-roundtrip-'))
      const policyPath = join(directory, 'roundtrip.bin')
      try {
        await openMenu(page, '策略管理')
        await app.evaluate(({ dialog }, path) => {
          dialog.showSaveDialog = async () => ({ canceled: false, filePath: path })
        }, policyPath)
        await page.getByTestId('export-policy').click()
        await expect.poll(async () => (await readFile(policyPath)).length).toBeGreaterThan(8)
        await openMenu(page, '文件访问控制')
        await page.getByTestId('exec-control-switch').click()
        await openMenu(page, 'U盘设备控制')
        await page.getByTestId('remove-WL-EXISTING-001').click()
        await page.getByLabel('删除确认').getByRole('button', { name: '删除', exact: true }).click()
        await openMenu(page, '策略管理')
        await app.evaluate(({ dialog }, path) => {
          dialog.showOpenDialog = async () => ({ canceled: false, filePaths: [path] })
        }, policyPath)
        await page.getByTestId('import-policy').click()
        await page.getByRole('button', { name: '导入', exact: true }).click()
        await expectLatestMessage(page, /策略已导入/)
        await openMenu(page, 'U盘设备控制')
        await expect(page.getByText('WL-EXISTING-001', { exact: true })).toBeVisible()
        await openMenu(page, '文件访问控制')
        await expect(page.getByTestId('exec-control-switch')).not.toHaveClass(/is-checked/)
      } finally {
        await rm(directory, { recursive: true, force: true })
      }
    })
  })

  test('连接断开后保留数据且写入、导入和导出均失败', async () => {
    await withOperator(async (device, app, page) => {
      const directory = await mkdtemp(join(tmpdir(), 'usb-policy-disconnect-'))
      const importPath = join(directory, 'import.bin')
      await writeFile(importPath, await readFile(IMPORT_FIXTURE))
      try {
        device.disconnectSockets()
        await expect(page.getByTestId('connection-status')).toContainText('未连接')
        await expect(page.getByText('.jse', { exact: true })).toBeVisible()
        await page.getByTestId('exec-control-switch').click()
        await expectLatestMessage(page, '装置已断开连接，无法修改策略')
        await page.getByTestId('add-blacklist-trigger').click()
        await page.locator('[data-testid="blacklist-extension-input"]').fill('.bat')
        await page.getByTestId('blacklist-submit').click()
        await expectLatestMessage(page, '装置已断开连接，无法修改策略')
        await page.keyboard.press('Escape')
        await openMenu(page, 'U盘设备控制')
        await expect(page.getByText('WL-EXISTING-001', { exact: true })).toBeVisible()
        await page.getByTestId('remove-WL-EXISTING-001').click()
        await expectLatestMessage(page, '装置已断开连接，无法修改白名单')
        await openMenu(page, '策略管理')
        await app.evaluate(({ dialog }, path) => {
          dialog.showSaveDialog = async () => ({ canceled: false, filePath: `${path}.out` })
          dialog.showOpenDialog = async () => ({ canceled: false, filePaths: [path] })
        }, importPath)
        await page.getByTestId('export-policy').click()
        await expectLatestMessage(page, '装置已断开连接，无法传输策略')
        await page.getByTestId('import-policy').click()
        await expectLatestMessage(page, '装置已断开连接，无法传输策略')
      } finally {
        await rm(directory, { recursive: true, force: true })
      }
    })
  })

  test('顶栏不显示 IP、用户名和角色，用户图标提供修改密码与登出', async () => {
    await withOperator(async (_device, _app, page) => {
      await expect(page.getByText('127.0.0.1', { exact: true })).toHaveCount(0)
      await expect(page.getByText('operator-user', { exact: true })).toHaveCount(0)
      await expect(page.getByText('操作员', { exact: true })).toHaveCount(0)
      await page.getByTestId('user-menu-trigger').click()
      await expect(page.getByText('修改密码', { exact: true })).toBeVisible()
      await expect(page.getByText('登出', { exact: true })).toBeVisible()
      await page.getByText('修改密码', { exact: true }).click()
      await expect(page.getByRole('dialog', { name: '修改密码' })).toBeVisible()
      await page.keyboard.press('Escape')
      await page.getByTestId('user-menu-trigger').click()
      await page.getByText('登出', { exact: true }).click()
      await expect(page).toHaveURL(/#\/login$/)
    })
  })
})
