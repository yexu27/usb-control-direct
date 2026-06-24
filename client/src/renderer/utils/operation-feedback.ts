import { useAppToastStore } from '@/stores/app-toast'
import { alertAction } from '@/utils/confirm-action'

export function showSuccessToast(message: string): void {
  useAppToastStore().showSuccess(message)
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
