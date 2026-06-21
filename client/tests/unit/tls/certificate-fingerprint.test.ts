import { describe, expect, it } from 'vitest'
import { assertCertificateFingerprint } from '../../../src/main/tls/certificate-fingerprint'

const EXPECTED_FINGERPRINT = 'a'.repeat(64)

describe('assertCertificateFingerprint', () => {
  it('接受大小写和冒号格式不同但内容一致的指纹', () => {
    const actualFingerprint = Array.from({ length: 32 }, () => 'AA').join(':')

    expect(() => {
      assertCertificateFingerprint(actualFingerprint, EXPECTED_FINGERPRINT)
    }).not.toThrow()
  })

  it('拒绝缺失的构建指纹', () => {
    expect(() => {
      assertCertificateFingerprint(EXPECTED_FINGERPRINT, '')
    }).toThrow('管理端证书指纹配置无效')
  })

  it('拒绝装置端缺失或格式错误的指纹', () => {
    expect(() => {
      assertCertificateFingerprint(undefined, EXPECTED_FINGERPRINT)
    }).toThrow('版本不兼容，请升级管理端')
  })

  it('拒绝与构建指纹不一致的装置证书', () => {
    expect(() => {
      assertCertificateFingerprint('b'.repeat(64), EXPECTED_FINGERPRINT)
    }).toThrow('版本不兼容，请升级管理端')
  })
})
