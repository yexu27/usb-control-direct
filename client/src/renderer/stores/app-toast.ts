import { defineStore } from 'pinia'

const SUCCESS_DURATION_MS = 2_000

type ToastType = 'success'

interface ToastState {
  visible: boolean
  message: string
  type: ToastType
  timerId: ReturnType<typeof setTimeout> | null
}

export const useAppToastStore = defineStore('app-toast', {
  state: (): ToastState => ({
    visible: false,
    message: '',
    type: 'success',
    timerId: null,
  }),
  actions: {
    showSuccess(message: string): void {
      if (this.timerId != null) {
        globalThis.clearTimeout(this.timerId)
      }
      this.message = message
      this.type = 'success'
      this.visible = true
      this.timerId = globalThis.setTimeout(() => {
        this.visible = false
        this.timerId = null
      }, SUCCESS_DURATION_MS)
    },
    hide(): void {
      if (this.timerId != null) {
        globalThis.clearTimeout(this.timerId)
      }
      this.visible = false
      this.timerId = null
    },
  },
})
