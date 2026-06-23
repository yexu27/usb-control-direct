import { describe, expect, it } from 'vitest'
import {
  dateToUnixSeconds,
  formatUnixSeconds,
  getDefaultLogRange,
  isBeforeRetentionBoundary,
} from '../../../src/renderer/utils/date-time'

describe('date-time utils', () => {
  it('formats unix seconds to Chinese management display format', () => {
    const localDate = new Date(2026, 0, 1, 0, 0, 10, 900)
    const unixSeconds = Math.floor(localDate.getTime() / 1000)

    expect(formatUnixSeconds(unixSeconds)).toBe('2026-01-01 00:00:10')
    expect(formatUnixSeconds(0)).toBe('-')
  })

  it('converts Date to unix seconds without milliseconds', () => {
    const localDate = new Date(2026, 0, 1, 0, 0, 10, 900)

    expect(dateToUnixSeconds(localDate)).toBe(Math.floor(localDate.getTime() / 1000))
  })

  it('uses today 23:59:59 and seven days before 00:00:00 as default range', () => {
    const range = getDefaultLogRange(new Date('2026-06-22T12:30:00+08:00'))

    expect(range.start.getFullYear()).toBe(2026)
    expect(range.start.getMonth()).toBe(5)
    expect(range.start.getDate()).toBe(16)
    expect(range.start.getHours()).toBe(0)
    expect(range.end.getDate()).toBe(22)
    expect(range.end.getHours()).toBe(23)
    expect(range.end.getMinutes()).toBe(59)
    expect(range.end.getSeconds()).toBe(59)
  })

  it('detects date ranges older than retention boundary', () => {
    const now = new Date('2026-06-22T12:00:00+08:00')

    expect(isBeforeRetentionBoundary(new Date('2025-12-21T23:59:59+08:00'), now)).toBe(true)
    expect(isBeforeRetentionBoundary(new Date('2025-12-22T00:00:00+08:00'), now)).toBe(false)
  })
})
