const SHA256_FINGERPRINT_PATTERN = /^[0-9a-f]{64}$/

function normalizeFingerprint(fingerprint: string): string {
  return fingerprint.replaceAll(':', '').trim().toLowerCase()
}

export function assertCertificateFingerprint(
  actualFingerprint: string | undefined,
  expectedFingerprint: string,
): void {
  const normalizedExpected = normalizeFingerprint(expectedFingerprint)
  const normalizedActual = normalizeFingerprint(actualFingerprint ?? '')

  if (!SHA256_FINGERPRINT_PATTERN.test(normalizedExpected)) {
    throw new Error('管理端证书指纹配置无效')
  }

  if (!SHA256_FINGERPRINT_PATTERN.test(normalizedActual)) {
    throw new Error('版本不兼容，请升级管理端')
  }

  if (normalizedActual !== normalizedExpected) {
    throw new Error('版本不兼容，请升级管理端')
  }
}
