import { usb_control } from '../../shared/proto/usb_control'
import { MSG_RSP_COMMON, handleCommonResponse, sendCommand } from './send-command'

const MSG_CMD_LIST_WHITELIST = 0x0100
const MSG_RSP_LIST_WHITELIST = 0x0101
const MSG_CMD_ADD_WHITELIST = 0x0104
const MSG_CMD_REMOVE_WHITELIST = 0x0105
const MSG_CMD_UPDATE_WHITELIST = 0x0106

export interface AddWhitelistInput {
  serialNumber: string
  vid: string
  pid: string
  deviceName: string
  capacityBytes: number
  permission: string
  description: string
  addMethod: string
  deviceType: string
}

export async function listWhitelist(
  sessionToken: string,
): Promise<usb_control.RspListWhitelist> {
  const command = usb_control.CmdListWhitelist.fromObject({ sessionToken })
  const payload = await sendCommand(
    MSG_CMD_LIST_WHITELIST,
    usb_control.CmdListWhitelist.encode(command).finish(),
    MSG_RSP_LIST_WHITELIST,
  )
  return usb_control.RspListWhitelist.decode(payload)
}

async function sendCommonCommand(
  msgType: number,
  commandPayload: Uint8Array,
): Promise<void> {
  const payload = await sendCommand(msgType, commandPayload, MSG_RSP_COMMON)
  handleCommonResponse(usb_control.RspCommon.decode(payload))
}

export async function addWhitelist(
  sessionToken: string,
  input: AddWhitelistInput,
): Promise<void> {
  const command = usb_control.CmdAddWhitelist.fromObject({ sessionToken, ...input })
  await sendCommonCommand(
    MSG_CMD_ADD_WHITELIST,
    usb_control.CmdAddWhitelist.encode(command).finish(),
  )
}

export async function removeWhitelist(
  sessionToken: string,
  serialNumber: string,
): Promise<void> {
  const command = usb_control.CmdRemoveWhitelist.fromObject({ sessionToken, serialNumber })
  await sendCommonCommand(
    MSG_CMD_REMOVE_WHITELIST,
    usb_control.CmdRemoveWhitelist.encode(command).finish(),
  )
}

export async function updateWhitelist(
  sessionToken: string,
  serialNumber: string,
  permission: string,
  description: string,
): Promise<void> {
  const command = usb_control.CmdUpdateWhitelist.fromObject({
    sessionToken,
    serialNumber,
    permission,
    description,
  })
  await sendCommonCommand(
    MSG_CMD_UPDATE_WHITELIST,
    usb_control.CmdUpdateWhitelist.encode(command).finish(),
  )
}
