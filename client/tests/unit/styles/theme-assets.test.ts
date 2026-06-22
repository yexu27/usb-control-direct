import { existsSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'

describe('theme assets', () => {
  it('bundles selected Source Han Sans font weights in renderer assets', () => {
    const root = resolve(__dirname, '../../../src/renderer/assets/fonts')

    expect(existsSync(resolve(root, 'SourceHanSansCN-Regular.otf'))).toBe(true)
    expect(existsSync(resolve(root, 'SourceHanSansCN-Medium.otf'))).toBe(true)
    expect(existsSync(resolve(root, 'SourceHanSansCN-Bold.otf'))).toBe(true)
  })
})
