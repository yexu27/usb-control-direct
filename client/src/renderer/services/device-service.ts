import { usb_control } from '../../shared/proto/usb_control'
import { sendCommand } from './send-command'

const MSG_CMD_GET_CONNECTED_DEVICES = 0x0102
const MSG_RSP_CONNECTED_DEVICES = 0x0103

export async function getConnectedDevices(
  sessionToken: string,
): Promise<usb_control.RspConnectedDevices> {
  const command = usb_control.CmdGetConnectedDevices.fromObject({ sessionToken })
  const payload = await sendCommand(
    MSG_CMD_GET_CONNECTED_DEVICES,
    usb_control.CmdGetConnectedDevices.encode(command).finish(),
    MSG_RSP_CONNECTED_DEVICES,
  )
  return usb_control.RspConnectedDevices.decode(payload)
}
