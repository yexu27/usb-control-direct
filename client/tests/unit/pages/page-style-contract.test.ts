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

  it.each(pageFiles.filter((fileName) => fileName !== 'UsbDevicesPage.vue'))(
    '%s uses the shared card surface for card regions',
    (fileName) => {
      const source = readPage(fileName)

      expect(source).toContain('app-card')
    },
  )
})
