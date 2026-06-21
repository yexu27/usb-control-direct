import { usb_control } from '../../shared/proto/usb_control'
import {
  FILE_TRANSFER_TIMEOUT,
  MSG_RSP_COMMON,
  handleCommonResponse,
  sendCommand,
} from './send-command'

const MSG_CMD_GET_SYSTEM_INFO = 0x0500
const MSG_RSP_SYSTEM_INFO = 0x0501
const MSG_CMD_UPLOAD_SYSTEM_UPGRADE = 0x0502
const MSG_CMD_UPLOAD_VIRUSDB_UPGRADE = 0x0503
const MSG_CMD_UPDATE_DEVICE_DESC = 0x0504

export async function getSystemInfo(sessionToken: string): Promise<usb_control.RspSystemInfo> {
  const command = usb_control.CmdGetSystemInfo.fromObject({ sessionToken })
  const payload = await sendCommand(
    MSG_CMD_GET_SYSTEM_INFO,
    usb_control.CmdGetSystemInfo.encode(command).finish(),
    MSG_RSP_SYSTEM_INFO,
  )
  return usb_control.RspSystemInfo.decode(payload)
}

async function sendCommonCommand(
  msgType: number,
  commandPayload: Uint8Array,
  timeout?: number,
): Promise<void> {
  const payload = await sendCommand(msgType, commandPayload, MSG_RSP_COMMON, timeout)
  handleCommonResponse(usb_control.RspCommon.decode(payload))
}

export async function uploadSystemUpgrade(
  sessionToken: string,
  upgradeData: Uint8Array,
  targetVersion: string,
  sha256Checksum: string,
): Promise<void> {
  const command = usb_control.CmdUploadSystemUpgrade.fromObject({
    sessionToken,
    upgradeData,
    targetVersion,
    sha256Checksum,
  })
  await sendCommonCommand(
    MSG_CMD_UPLOAD_SYSTEM_UPGRADE,
    usb_control.CmdUploadSystemUpgrade.encode(command).finish(),
    FILE_TRANSFER_TIMEOUT,
  )
}

export async function uploadVirusdbUpgrade(
  sessionToken: string,
  upgradeData: Uint8Array,
  targetVersion: string,
  sha256Checksum: string,
): Promise<void> {
  const command = usb_control.CmdUploadVirusdbUpgrade.fromObject({
    sessionToken,
    upgradeData,
    targetVersion,
    sha256Checksum,
  })
  await sendCommonCommand(
    MSG_CMD_UPLOAD_VIRUSDB_UPGRADE,
    usb_control.CmdUploadVirusdbUpgrade.encode(command).finish(),
    FILE_TRANSFER_TIMEOUT,
  )
}

export async function updateDeviceDescription(
  sessionToken: string,
  description: string,
): Promise<void> {
  const command = usb_control.CmdUpdateDeviceDesc.fromObject({ sessionToken, description })
  await sendCommonCommand(
    MSG_CMD_UPDATE_DEVICE_DESC,
    usb_control.CmdUpdateDeviceDesc.encode(command).finish(),
  )
}
