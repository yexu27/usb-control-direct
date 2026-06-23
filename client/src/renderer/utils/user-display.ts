type UnixSecondInput = number | string | { toString(): string }

export type RoleValue = 'admin' | 'operator' | 'auditor'
export type UserStatus = 'active' | 'locked'

export interface UserDisplayOption<TValue extends string = string> {
  value: TValue
  label: string
}

export const USER_ROLE_OPTIONS: UserDisplayOption<RoleValue>[] = [
  { value: 'admin', label: '系统管理员' },
  { value: 'operator', label: '操作员' },
  { value: 'auditor', label: '审计员' },
]

export function formatUserRole(role: string): string {
  return USER_ROLE_OPTIONS.find((item) => item.value === role)?.label ?? role
}

export function formatUserStatus(status: string): string {
  if (status === 'locked') {
    return '锁定'
  }

  return '正常'
}

export function formatCreatedAt(createdAt: UnixSecondInput): string {
  const seconds = Number(createdAt)

  if (!Number.isFinite(seconds) || seconds <= 0) {
    return '内置'
  }

  const date = new Date(seconds * 1000)

  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`
}
