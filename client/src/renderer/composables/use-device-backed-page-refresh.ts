import { onMounted, onUnmounted } from 'vue'
import { onPageRefresh } from '@/services/page-refresh-events'

type RefreshFn = () => void | Promise<void>

export function useDeviceBackedPageRefresh(refresh: RefreshFn): void {
  onMounted(() => {
    void refresh()
  })

  const unsubscribe = onPageRefresh(() => {
    void refresh()
  })

  onUnmounted(() => {
    unsubscribe()
  })
}
