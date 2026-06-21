import { usb_control } from '../../shared/proto/usb_control'
import { MSG_RSP_COMMON, handleCommonResponse, sendCommand } from './send-command'

const MSG_CMD_GET_FILE_POLICY = 0x0200
const MSG_RSP_FILE_POLICY = 0x0201
const MSG_CMD_UPDATE_FILE_POLICY_SWITCH = 0x0202
const MSG_CMD_ADD_BLACKLIST_EXTENSION = 0x0203
const MSG_CMD_REMOVE_BLACKLIST_EXTENSION = 0x0204

export async function getFilePolicy(sessionToken: string): Promise<usb_control.RspFilePolicy> {
  const command = usb_control.CmdGetFilePolicy.fromObject({ sessionToken })
  const payload = await sendCommand(
    MSG_CMD_GET_FILE_POLICY,
    usb_control.CmdGetFilePolicy.encode(command).finish(),
    MSG_RSP_FILE_POLICY,
  )
  return usb_control.RspFilePolicy.decode(payload)
}

async function sendCommonCommand(msgType: number, commandPayload: Uint8Array): Promise<void> {
  const payload = await sendCommand(msgType, commandPayload, MSG_RSP_COMMON)
  handleCommonResponse(usb_control.RspCommon.decode(payload))
}

export async function updateSwitch(
  sessionToken: string,
  policyKey: string,
  enabled: boolean,
): Promise<void> {
  const command = usb_control.CmdUpdateFilePolicySwitch.fromObject({
    sessionToken,
    policyKey,
    enabled,
  })
  await sendCommonCommand(
    MSG_CMD_UPDATE_FILE_POLICY_SWITCH,
    usb_control.CmdUpdateFilePolicySwitch.encode(command).finish(),
  )
}

export async function addBlacklistExtension(
  sessionToken: string,
  extension: string,
  description: string,
): Promise<void> {
  const command = usb_control.CmdAddBlacklistExtension.fromObject({
    sessionToken,
    extension,
    description,
  })
  await sendCommonCommand(
    MSG_CMD_ADD_BLACKLIST_EXTENSION,
    usb_control.CmdAddBlacklistExtension.encode(command).finish(),
  )
}

export async function removeBlacklistExtension(
  sessionToken: string,
  extension: string,
): Promise<void> {
  const command = usb_control.CmdRemoveBlacklistExtension.fromObject({ sessionToken, extension })
  await sendCommonCommand(
    MSG_CMD_REMOVE_BLACKLIST_EXTENSION,
    usb_control.CmdRemoveBlacklistExtension.encode(command).finish(),
  )
}
