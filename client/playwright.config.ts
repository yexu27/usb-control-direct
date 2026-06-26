import { defineConfig } from '@playwright/test'

const includeManualMock = process.env.E2E_MANUAL_MOCK === '1'

export default defineConfig({
  testDir: './tests/e2e',
  testIgnore: includeManualMock ? [] : ['**/manual-mock.spec.ts'],
  timeout: 30_000,
  fullyParallel: false,
  workers: 1,
  reporter: 'list',
  use: {
    trace: 'retain-on-failure',
  },
})
