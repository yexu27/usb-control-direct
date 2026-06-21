import { usb_control } from '../../shared/proto/usb_control'
import { MSG_RSP_COMMON, handleCommonResponse, sendCommand } from './send-command'

const MSG_CMD_LIST_USERS = 0x0600
const MSG_RSP_LIST_USERS = 0x0601
const MSG_CMD_CREATE_USER = 0x0602
const MSG_CMD_DELETE_USER = 0x0603
const MSG_CMD_RESET_PASSWORD = 0x0604
const MSG_CMD_CHANGE_PASSWORD = 0x0605

export async function listUsers(sessionToken: string): Promise<usb_control.RspListUsers> {
  const command = usb_control.CmdListUsers.fromObject({ sessionToken })
  const payload = await sendCommand(
    MSG_CMD_LIST_USERS,
    usb_control.CmdListUsers.encode(command).finish(),
    MSG_RSP_LIST_USERS,
  )
  return usb_control.RspListUsers.decode(payload)
}

async function sendCommonCommand(msgType: number, commandPayload: Uint8Array): Promise<void> {
  const payload = await sendCommand(msgType, commandPayload, MSG_RSP_COMMON)
  handleCommonResponse(usb_control.RspCommon.decode(payload))
}

export async function createUser(
  sessionToken: string,
  username: string,
  role: string,
  password: string,
  confirmPassword: string,
): Promise<void> {
  const command = usb_control.CmdCreateUser.fromObject({
    sessionToken,
    username,
    role,
    password,
    confirmPassword,
  })
  await sendCommonCommand(
    MSG_CMD_CREATE_USER,
    usb_control.CmdCreateUser.encode(command).finish(),
  )
}

export async function deleteUser(sessionToken: string, username: string): Promise<void> {
  const command = usb_control.CmdDeleteUser.fromObject({ sessionToken, username })
  await sendCommonCommand(
    MSG_CMD_DELETE_USER,
    usb_control.CmdDeleteUser.encode(command).finish(),
  )
}

export async function resetPassword(
  sessionToken: string,
  username: string,
  newPassword: string,
  confirmPassword: string,
): Promise<void> {
  const command = usb_control.CmdResetPassword.fromObject({
    sessionToken,
    username,
    newPassword,
    confirmPassword,
  })
  await sendCommonCommand(
    MSG_CMD_RESET_PASSWORD,
    usb_control.CmdResetPassword.encode(command).finish(),
  )
}

export async function changePassword(
  sessionToken: string,
  oldPassword: string,
  newPassword: string,
  confirmPassword: string,
): Promise<void> {
  const command = usb_control.CmdChangePassword.fromObject({
    sessionToken,
    oldPassword,
    newPassword,
    confirmPassword,
  })
  await sendCommonCommand(
    MSG_CMD_CHANGE_PASSWORD,
    usb_control.CmdChangePassword.encode(command).finish(),
  )
}
