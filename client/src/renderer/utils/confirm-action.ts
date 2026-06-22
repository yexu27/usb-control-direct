import { ElMessageBox, type ElMessageBoxOptions } from 'element-plus'

interface ConfirmActionOptions {
  title: string
  message: string
  confirmButtonText: string
  cancelButtonText?: string
  type?: ElMessageBoxOptions['type']
}

export async function confirmAction(options: ConfirmActionOptions): Promise<void> {
  await ElMessageBox.confirm(options.message, options.title, {
    type: options.type ?? 'warning',
    confirmButtonText: options.confirmButtonText,
    cancelButtonText: options.cancelButtonText ?? '取消',
    customClass: 'app-confirm-message-box',
    modalClass: 'app-confirm-message-box-overlay',
    appendTo: document.body,
    distinguishCancelAndClose: true,
    closeOnClickModal: false,
    closeOnPressEscape: true,
  })
}
