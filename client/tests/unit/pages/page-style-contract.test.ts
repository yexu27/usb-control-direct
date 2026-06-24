import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'

const pageFiles = [
  'FileAccessPage.vue',
  'UsbDevicesPage.vue',
  'PoliciesPage.vue',
  'LogsPage.vue',
  'SystemPage.vue',
  'UsersPage.vue',
] as const

function readPage(fileName: string): string {
  return readFileSync(resolve(__dirname, '../../../src/renderer/pages', fileName), 'utf8')
}

describe('page style contract', () => {
  it.each(pageFiles)('%s uses the shared page shell classes', (fileName) => {
    const source = readPage(fileName)

    expect(source).toContain('app-page')
    expect(source).toContain('app-page-header')
    expect(source).toContain('app-page-title')
  })

  it.each(pageFiles.filter((fileName) => ![
    'UsbDevicesPage.vue',
    'UsersPage.vue',
    'PoliciesPage.vue',
    'SystemPage.vue',
    'LogsPage.vue',
  ].includes(fileName)))(
    '%s uses the shared card surface for card regions',
    (fileName) => {
      const source = readPage(fileName)

      expect(source).toContain('app-card')
    },
  )

  it('UsersPage uses the confirmed rounded table shell instead of the old card', () => {
    const source = readPage('UsersPage.vue')

    expect(source).toContain('users-table-shell')
    expect(source).not.toContain('users-card')
  })

  it('UsbDevicesPage uses the confirmed whitelist panel', () => {
    const source = readPage('UsbDevicesPage.vue')

    expect(source).toContain('usb-table-panel')
    expect(source).toContain('usb-bottom-note')
    expect(source).not.toContain('usb-whitelist-card')
  })

  it('PoliciesPage uses the confirmed transfer cards', () => {
    const source = readPage('PoliciesPage.vue')

    expect(source).toContain('policy-transfer-card')
    expect(source).toContain('transfer-icon')
  })

  it('SystemPage uses the confirmed four system cards', () => {
    const source = readPage('SystemPage.vue')

    expect(source).toContain('system-card')
    expect(source).toContain('system-card-grid')
    expect(source).not.toContain('安全U盘自动升级')
  })

  it('LogsPage uses the confirmed logs prototype shell and custom pagination', () => {
    const source = readPage('LogsPage.vue')

    expect(source).toContain('logs-prototype-shell')
    expect(source).toContain('logs-segmented-tabs')
    expect(source).toContain('logs-prototype-pagination')
    expect(source).toContain(':show-default-pagination="false"')
    expect(source).not.toContain('logs-card app-card')
    expect(source).not.toContain('data-testid="logs-role-badge"')
  })

  it('LogsPage keeps log filter controls within the confirmed content width', () => {
    const source = readPage('LogsPage.vue')

    expect(source).toContain('grid-template-columns: 150px 168px 168px 86px minmax(0, 1fr)')
    expect(source).toContain('grid-template-columns: 220px 176px 176px minmax(0, 1fr)')
    expect(source).toContain('font-size: 13px')
    expect(source).toContain('height: 34px')
    expect(source).not.toContain('grid-template-columns: minmax(260px, 1.4fr) 280px 280px 150px auto')
    expect(source).not.toContain('grid-template-columns: minmax(320px, 1fr) 280px 280px auto')
  })
})
