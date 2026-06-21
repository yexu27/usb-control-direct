import { resolve } from 'path'
import { defineConfig, externalizeDepsPlugin } from 'electron-vite'
import vue from '@vitejs/plugin-vue'
import AutoImport from 'unplugin-auto-import/vite'
import Components from 'unplugin-vue-components/vite'
import { ElementPlusResolver } from 'unplugin-vue-components/resolvers'

const certFingerprint = process.env['USB_CONTROL_CERT_FINGERPRINT']?.trim().toLowerCase() ?? ''

if (!/^[0-9a-f]{64}$/.test(certFingerprint)) {
  throw new Error('USB_CONTROL_CERT_FINGERPRINT 必须是 server.crt 的 64 位 SHA256 十六进制指纹')
}

export default defineConfig({
  main: {
    define: {
      __USB_CONTROL_CERT_FINGERPRINT__: JSON.stringify(certFingerprint),
    },
    plugins: [externalizeDepsPlugin()],
    build: {
      rollupOptions: {
        external: ['protobufjs'],
      },
    },
  },
  preload: {
    plugins: [externalizeDepsPlugin()],
  },
  renderer: {
    resolve: {
      alias: {
        '@': resolve('src/renderer'),
      },
    },
    plugins: [
      vue(),
      AutoImport({
        resolvers: [ElementPlusResolver()],
      }),
      Components({
        resolvers: [ElementPlusResolver()],
      }),
    ],
    css: {
      preprocessorOptions: {
        scss: {
          additionalData: `@use "@/styles/variables" as *;`,
        },
      },
    },
  },
})
