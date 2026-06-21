export interface PasswordValidationResult {
  valid: boolean
  message: string
}

const MIN_PASSWORD_LENGTH = 8

export function validatePasswordComplexity(password: string): PasswordValidationResult {
  if (password.length < MIN_PASSWORD_LENGTH) {
    return { valid: false, message: `密码长度不能少于${MIN_PASSWORD_LENGTH}位` }
  }

  const categoryCount = [/[a-zA-Z]/, /\d/, /[^a-zA-Z0-9]/].filter((pattern) =>
    pattern.test(password),
  ).length

  if (categoryCount < 2) {
    return { valid: false, message: '密码至少包含字母、数字、特殊字符中的两种' }
  }

  return { valid: true, message: '' }
}
