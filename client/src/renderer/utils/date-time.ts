export interface DateRange {
  start: Date
  end: Date
}

type UnixSecondInput = number | string | { toString(): string }

function padTwoDigits(value: number): string {
  return String(value).padStart(2, '0')
}

export function formatUnixSeconds(value: UnixSecondInput | null | undefined): string {
  const seconds = Number(value ?? 0)

  if (!Number.isFinite(seconds) || seconds <= 0) {
    return '-'
  }

  const date = new Date(seconds * 1000)

  return [
    `${date.getFullYear()}-${padTwoDigits(date.getMonth() + 1)}-${padTwoDigits(date.getDate())}`,
    `${padTwoDigits(date.getHours())}:${padTwoDigits(date.getMinutes())}:${padTwoDigits(date.getSeconds())}`,
  ].join(' ')
}

export function dateToUnixSeconds(date: Date): number {
  return Math.floor(date.getTime() / 1000)
}

export function getDefaultLogRange(now: Date = new Date()): DateRange {
  const start = new Date(now)
  start.setDate(start.getDate() - 6)
  start.setHours(0, 0, 0, 0)

  const end = new Date(now)
  end.setHours(23, 59, 59, 999)

  return { start, end }
}

export function getRetentionBoundary(now: Date = new Date()): Date {
  const boundary = new Date(now)
  boundary.setMonth(boundary.getMonth() - 6)
  boundary.setHours(0, 0, 0, 0)

  return boundary
}

export function isBeforeRetentionBoundary(value: Date, now: Date = new Date()): boolean {
  return value.getTime() < getRetentionBoundary(now).getTime()
}
