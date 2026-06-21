import { usb_control } from '../../shared/proto/usb_control'
import {
  FILE_TRANSFER_TIMEOUT,
  MSG_RSP_COMMON,
  handleCommonResponse,
  handleResultError,
  sendCommand,
} from './send-command'

const MSG_CMD_EXPORT_POLICY = 0x0300
const MSG_RSP_EXPORT_POLICY = 0x0301
const MSG_CMD_IMPORT_POLICY = 0x0302

export async function exportPolicy(sessionToken: string): Promise<usb_control.RspExportPolicy> {
  const command = usb_control.CmdExportPolicy.fromObject({ sessionToken })
  const payload = await sendCommand(
    MSG_CMD_EXPORT_POLICY,
    usb_control.CmdExportPolicy.encode(command).finish(),
    MSG_RSP_EXPORT_POLICY,
    FILE_TRANSFER_TIMEOUT,
  )
  const response = usb_control.RspExportPolicy.decode(payload)
  if (!response.success) {
    handleResultError(response.resultCode, response.errorMessage)
  }
  return response
}

export async function importPolicy(
  sessionToken: string,
  policyData: Uint8Array,
): Promise<void> {
  const command = usb_control.CmdImportPolicy.fromObject({ sessionToken, policyData })
  const payload = await sendCommand(
    MSG_CMD_IMPORT_POLICY,
    usb_control.CmdImportPolicy.encode(command).finish(),
    MSG_RSP_COMMON,
    FILE_TRANSFER_TIMEOUT,
  )
  handleCommonResponse(usb_control.RspCommon.decode(payload))
}
