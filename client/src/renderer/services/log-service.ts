import { usb_control } from '../../shared/proto/usb_control'
import {
  FILE_TRANSFER_TIMEOUT,
  MSG_RSP_COMMON,
  handleCommonResponse,
  handleResultError,
  sendCommand,
} from './send-command'

const MSG_CMD_QUERY_LOGS = 0x0400
const MSG_RSP_QUERY_LOGS = 0x0401
const MSG_CMD_EXPORT_LOGS = 0x0402
const MSG_RSP_EXPORT_LOGS = 0x0403
const MSG_CMD_DELETE_LOGS = 0x0404

export interface LogQueryInput {
  logType: string
  startTime: number
  endTime: number
  keyword: string
  eventType: string
  page: number
  pageSize: number
}

export type LogExportInput = Omit<LogQueryInput, 'page' | 'pageSize'>

export async function queryLogs(
  sessionToken: string,
  input: LogQueryInput,
): Promise<usb_control.RspQueryLogs> {
  const command = usb_control.CmdQueryLogs.fromObject({ sessionToken, ...input })
  const payload = await sendCommand(
    MSG_CMD_QUERY_LOGS,
    usb_control.CmdQueryLogs.encode(command).finish(),
    MSG_RSP_QUERY_LOGS,
  )
  const response = usb_control.RspQueryLogs.decode(payload)
  if (!response.success) {
    handleResultError(response.resultCode, response.errorMessage)
  }
  return response
}

export async function exportLogs(
  sessionToken: string,
  input: LogExportInput,
): Promise<usb_control.RspExportLogs> {
  const command = usb_control.CmdExportLogs.fromObject({ sessionToken, ...input })
  const payload = await sendCommand(
    MSG_CMD_EXPORT_LOGS,
    usb_control.CmdExportLogs.encode(command).finish(),
    MSG_RSP_EXPORT_LOGS,
    FILE_TRANSFER_TIMEOUT,
  )
  const response = usb_control.RspExportLogs.decode(payload)
  if (!response.success) {
    handleResultError(response.resultCode, response.errorMessage)
  }
  return response
}

export async function deleteLogs(
  sessionToken: string,
  logType: string,
  startTime: number,
  endTime: number,
): Promise<void> {
  const command = usb_control.CmdDeleteLogs.fromObject({
    sessionToken,
    logType,
    startTime,
    endTime,
  })
  const payload = await sendCommand(
    MSG_CMD_DELETE_LOGS,
    usb_control.CmdDeleteLogs.encode(command).finish(),
    MSG_RSP_COMMON,
  )
  handleCommonResponse(usb_control.RspCommon.decode(payload))
}
