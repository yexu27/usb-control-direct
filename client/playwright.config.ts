import { defineConfig } from '@playwright/test'

const manualMockSpec = 'manual-mock.spec.ts'
const includeManualMock =
  process.env.E2E_MANUAL_MOCK === '1' ||
  process.argv.some((arg) => arg.includes(manualMockSpec))

export default defineConfig({
  testDir: './tests/e2e',
  testIgnore: includeManualMock ? [] : [`**/${manualMockSpec}`],
  timeout: 30_000,
  fullyParallel: false,
  workers: 1,
  reporter: 'list',
  use: {
    trace: 'retain-on-failure',
  },
})
