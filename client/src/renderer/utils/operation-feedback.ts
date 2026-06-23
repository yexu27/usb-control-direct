import { ElMessage } from 'element-plus'
import { alertAction } from '@/utils/confirm-action'

const SUCCESS_DURATION_MS = 2000
const TOP_CENTER_OFFSET = 24

export function showSuccessToast(message: string): void {
  ElMessage.success({
    message,
    duration: SUCCESS_DURATION_MS,
    offset: TOP_CENTER_OFFSET,
    showClose: false,
    grouping: true,
  })
}

export async function showErrorDialog(title: string, message: string): Promise<void> {
  await alertAction({
    title,
    message,
    type: 'error',
  })
}

export function errorMessage(error: unknown, fallback: string): string {
  return error instanceof Error ? error.message : fallback
}
