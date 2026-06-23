import { defineConfig } from 'vitest/config'
import { resolve } from 'path'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [vue()],
  test: {
    environment: 'node',
    include: ['tests/**/*.test.ts'],
    environmentMatchGlobs: [
      ['tests/unit/stores/**', 'happy-dom'],
      ['tests/unit/layouts/**', 'happy-dom'],
      ['tests/unit/components/**', 'happy-dom'],
      ['tests/unit/services/**', 'happy-dom'],
      ['tests/unit/pages/**', 'happy-dom'],
    ],
  },
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src/renderer'),
    },
  },
  css: {
    preprocessorOptions: {
      scss: {
        additionalData: `@use "@/styles/variables" as *;`,
      },
    },
  },
})
