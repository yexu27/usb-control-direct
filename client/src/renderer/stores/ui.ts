import { ref } from 'vue'
import { defineStore } from 'pinia'

export type ToastType = 'success' | 'warning' | 'error' | 'info'

export interface ToastItem {
  id: string
  message: string
  type: ToastType
}

let toastIdCounter = 0

export const useUiStore = defineStore('ui', () => {
  const isGlobalLoading = ref(false)
  const toastQueue = ref<ToastItem[]>([])

  function showLoading(): void {
    isGlobalLoading.value = true
  }

  function hideLoading(): void {
    isGlobalLoading.value = false
  }

  function showToast(message: string, type: ToastType = 'info'): void {
    const id = `toast-${++toastIdCounter}`
    toastQueue.value.push({ id, message, type })
  }

  function removeToast(id: string): void {
    const index = toastQueue.value.findIndex((t) => t.id === id)
    if (index !== -1) {
      toastQueue.value.splice(index, 1)
    }
  }

  return {
    isGlobalLoading,
    toastQueue,
    showLoading,
    hideLoading,
    showToast,
    removeToast,
  }
})
