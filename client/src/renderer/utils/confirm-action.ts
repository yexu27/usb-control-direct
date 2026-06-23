import { ElMessageBox, type ElMessageBoxOptions } from 'element-plus'

interface ConfirmActionOptions {
  title: string
  message: string
  confirmButtonText: string
  cancelButtonText?: string
  type?: ElMessageBoxOptions['type']
}

interface AlertActionOptions {
  title: string
  message: string
  confirmButtonText?: string
  type?: ElMessageBoxOptions['type']
}

const appMessageBoxOptions = {
  customClass: 'app-confirm-message-box',
  modalClass: 'app-confirm-message-box-overlay',
  appendTo: document.body,
  closeOnClickModal: false,
  closeOnPressEscape: true,
}

export async function confirmAction(options: ConfirmActionOptions): Promise<void> {
  await ElMessageBox.confirm(options.message, options.title, {
    type: options.type ?? 'warning',
    confirmButtonText: options.confirmButtonText,
    cancelButtonText: options.cancelButtonText ?? '取消',
    ...appMessageBoxOptions,
    distinguishCancelAndClose: true,
  })
}

export async function alertAction(options: AlertActionOptions): Promise<void> {
  await ElMessageBox.alert(options.message, options.title, {
    type: options.type ?? 'info',
    confirmButtonText: options.confirmButtonText ?? '确定',
    ...appMessageBoxOptions,
  })
}
