import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'
import { join } from 'node:path'

const root = join(__dirname, '../../..')

function read(relativePath: string): string {
  return readFileSync(join(root, relativePath), 'utf8')
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
})
