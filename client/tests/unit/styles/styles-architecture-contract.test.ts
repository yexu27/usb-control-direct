import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'
import { join } from 'node:path'

const root = join(__dirname, '../../..')

function read(relativePath: string): string {
  return readFileSync(join(root, relativePath), 'utf8')
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}

describe('styles architecture contract', () => {
  it('does not keep the legacy variables compatibility layer', () => {
    expect(existsSync(join(root, 'src/renderer/styles/_variables.scss'))).toBe(false)
  })

  it('does not auto-inject legacy variables into SCSS', () => {
    const electronConfig = read('electron.vite.config.ts')
    const vitestConfig = read('vitest.config.ts')

    expect(electronConfig).not.toContain('@/styles/variables')
    expect(vitestConfig).not.toContain('@/styles/variables')
    expect(electronConfig).not.toContain('additionalData')
    expect(vitestConfig).not.toContain('additionalData')
  })

  it('keeps tokens as the only SCSS design-token source', () => {
    const global = read('src/renderer/styles/global.scss')
    const theme = read('src/renderer/styles/_theme.scss')
    const tokens = read('src/renderer/styles/_tokens.scss')

    expect(global).not.toContain('./variables')
    expect(theme).toContain("@use './tokens' as *;")
    expect(tokens).toContain('$spacing-1: 4px;')
    expect(tokens).toContain('$font-weight-semibold: 600;')
    expect(tokens).toContain('$border-width-base: 1px;')
    expect(tokens).toContain('$info:')
  })

  it('does not leave legacy SCSS variable names in renderer source', () => {
    const legacyNames = [
      '$bg-white',
      '$bg-sidebar',
      '$border-color',
      '$border-width',
      '$border-radius',
      '$border-radius-lg',
      '$border-radius-card',
      '$font-base',
      '$font-sm',
      '$font-xxl',
      '$color-white',
      '$color-success',
      '$color-danger',
      '$color-warning',
      '$color-info',
      '$text-regular',
    ]

    const files = [
      'src/renderer/styles/global.scss',
      'src/renderer/pages/SystemPage.vue',
      'src/renderer/pages/UsersPage.vue',
      'src/renderer/pages/FileAccessPage.vue',
      'src/renderer/pages/UsbDevicesPage.vue',
      'src/renderer/pages/PoliciesPage.vue',
      'src/renderer/pages/LogsPage.vue',
      'src/renderer/pages/LoginPage.vue',
      'src/renderer/pages/LicensePage.vue',
      'src/renderer/components/ProgressDialog.vue',
      'src/renderer/components/ConnectionAlert.vue',
      'src/renderer/components/file-policy/AddBlacklistDialog.vue',
      'src/renderer/components/whitelist/AddWhitelistDialog.vue',
      'src/renderer/components/whitelist/EditWhitelistDialog.vue',
    ]

    const source = files.map((file) => read(file)).join('\n')

    for (const legacyName of legacyNames) {
      expect(source).not.toMatch(new RegExp(`${escapeRegExp(legacyName)}(?![a-zA-Z0-9_-])`))
    }
  })
})
