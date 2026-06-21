import { describe, expect, it } from 'vitest'
import { validatePasswordComplexity } from '../../../src/renderer/utils/password-validator'

describe('validatePasswordComplexity', () => {
  it('拒绝少于 8 位的密码', () => {
    expect(validatePasswordComplexity('Abc123!')).toEqual({
      valid: false,
      message: '密码长度不能少于8位',
    })
  })

  it.each(['abcdefgh', '12345678', '!@#$%^&*'])('拒绝只包含一种字符类型的密码', (password) => {
    expect(validatePasswordComplexity(password).valid).toBe(false)
  })

  it.each(['abcdef12', 'abcdef!@', '123456!@'])('接受包含至少两种字符类型的密码', (password) => {
    expect(validatePasswordComplexity(password)).toEqual({ valid: true, message: '' })
  })
})
