import { onMounted, onUnmounted } from 'vue'
import { onPageRefresh } from '@/services/page-refresh-events'

type RefreshFn = () => void | Promise<void>

export function useDeviceBackedPageRefresh(refresh: RefreshFn): void {
  const runRefresh = (): void => {
    Promise.resolve()
      .then(refresh)
      .catch((error: unknown) => {
        console.error('页面数据刷新失败', error)
      })
  }

  onMounted(() => {
    runRefresh()
  })

  const unsubscribe = onPageRefresh(() => {
    runRefresh()
  })

  onUnmounted(() => {
    unsubscribe()
  })
}
