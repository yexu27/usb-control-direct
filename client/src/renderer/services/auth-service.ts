import { usb_control } from '../../shared/proto/usb_control'
import {
  FILE_TRANSFER_TIMEOUT,
  MSG_RSP_COMMON,
  handleCommonResponse,
  handleResultError,
  sendCommand,
} from './send-command'

const MSG_CMD_LOGIN = 0x0001
const MSG_RSP_LOGIN = 0x0002
const MSG_CMD_AUTH_STATUS_QUERY = 0x0003
const MSG_RSP_AUTH_STATUS = 0x0004
const MSG_CMD_GET_MACHINE_CODE = 0x0005
const MSG_RSP_MACHINE_CODE = 0x0006
const MSG_CMD_UPLOAD_LICENSE = 0x0007
const MSG_RSP_UPLOAD_LICENSE = 0x0008
const MSG_CMD_LOGOUT = 0x0009

export async function login(username: string, password: string): Promise<usb_control.RspLogin> {
  const command = usb_control.CmdLogin.fromObject({ username, password })
  const responsePayload = await sendCommand(
    MSG_CMD_LOGIN,
    usb_control.CmdLogin.encode(command).finish(),
    MSG_RSP_LOGIN,
  )
  const response = usb_control.RspLogin.decode(responsePayload)
  if (!response.success) {
    handleResultError(response.resultCode, response.errorMessage)
  }
  return response
}

export async function logout(sessionToken: string): Promise<void> {
  const command = usb_control.CmdLogout.fromObject({ sessionToken })
  const responsePayload = await sendCommand(
    MSG_CMD_LOGOUT,
    usb_control.CmdLogout.encode(command).finish(),
    MSG_RSP_COMMON,
  )
  handleCommonResponse(usb_control.RspCommon.decode(responsePayload))
}

export async function queryAuthStatus(
  sessionToken: string,
): Promise<usb_control.RspAuthStatus> {
  const command = usb_control.CmdAuthStatusQuery.fromObject({ sessionToken })
  const responsePayload = await sendCommand(
    MSG_CMD_AUTH_STATUS_QUERY,
    usb_control.CmdAuthStatusQuery.encode(command).finish(),
    MSG_RSP_AUTH_STATUS,
  )
  return usb_control.RspAuthStatus.decode(responsePayload)
}

export async function getMachineCode(
  sessionToken: string,
): Promise<usb_control.RspMachineCode> {
  const command = usb_control.CmdGetMachineCode.fromObject({ sessionToken })
  const responsePayload = await sendCommand(
    MSG_CMD_GET_MACHINE_CODE,
    usb_control.CmdGetMachineCode.encode(command).finish(),
    MSG_RSP_MACHINE_CODE,
  )
  return usb_control.RspMachineCode.decode(responsePayload)
}

export async function uploadLicense(
  sessionToken: string,
  licenseData: Uint8Array,
): Promise<usb_control.RspUploadLicense> {
  const command = usb_control.CmdUploadLicense.fromObject({ sessionToken, licenseData })
  const responsePayload = await sendCommand(
    MSG_CMD_UPLOAD_LICENSE,
    usb_control.CmdUploadLicense.encode(command).finish(),
    MSG_RSP_UPLOAD_LICENSE,
    FILE_TRANSFER_TIMEOUT,
  )
  const response = usb_control.RspUploadLicense.decode(responsePayload)
  if (!response.success) {
    handleResultError(response.resultCode, response.errorMessage)
  }
  return response
}
