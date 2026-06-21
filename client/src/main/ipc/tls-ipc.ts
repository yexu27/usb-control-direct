import { ipcMain, type BrowserWindow } from 'electron'
import { isIP } from 'node:net'
import { IpcChannels } from '../../shared/ipc-channels'
import type { ConnectionEvent } from '../../shared/connection-state'
import type { TlsClient } from '../tls-client'
import { assertTrustedSender } from './trusted-sender'

const DEFAULT_PORT = 9600
const MAX_REQUEST_TIMEOUT = 300_000
const CONNECTION_EVENTS: ReadonlySet<ConnectionEvent> = new Set([
  'CONNECT_START',
  'CONNECT_SUCCESS',
  'CONNECT_FAIL',
  'AUTH_SUCCESS',
  'AUTH_FAIL',
  'LICENSE_AUTHORIZED',
  'LICENSE_UNAUTHORIZED',
  'LICENSE_EXPIRED',
  'CONFIG_LOADED',
  'CONFIG_FAILED',
  'HEARTBEAT_TIMEOUT',
  'NETWORK_ERROR',
  'LOGOUT',
  'LICENSE_UPLOAD_SUCCESS',
])

function parseIpv4Address(input: unknown): string {
  if (typeof input !== 'string' || isIP(input) !== 4) {
    throw new Error('装置 IP 格式错误')
  }
  return input
}

function parseMessageType(input: unknown): number {
  if (!Number.isInteger(input) || (input as number) < 0 || (input as number) > 0xffffffff) {
    throw new Error('消息类型无效')
  }
  return input as number
}

function parsePayload(input: unknown): Uint8Array {
  if (!(input instanceof Uint8Array)) {
    throw new Error('消息载荷无效')
  }
  return input
}

function parseTimeout(input: unknown): number | undefined {
  if (input == null) {
    return undefined
  }
  if (!Number.isInteger(input) || (input as number) <= 0 || (input as number) > MAX_REQUEST_TIMEOUT) {
    throw new Error('请求超时时间无效')
  }
  return input as number
}

function parseConnectionEvent(input: unknown): ConnectionEvent {
  if (typeof input !== 'string' || !CONNECTION_EVENTS.has(input as ConnectionEvent)) {
    throw new Error('连接状态事件无效')
  }
  return input as ConnectionEvent
}

export function registerTlsIpc(
  tlsClient: TlsClient,
  getMainWindow: () => BrowserWindow | null,
): void {
  ipcMain.handle(IpcChannels.tlsConnect, async (event, ip: unknown) => {
    assertTrustedSender(event, getMainWindow())
    await tlsClient.connect(parseIpv4Address(ip), DEFAULT_PORT)
  })

  ipcMain.handle(IpcChannels.tlsDisconnect, (event) => {
    assertTrustedSender(event, getMainWindow())
    tlsClient.disconnect()
  })

  ipcMain.handle(
    IpcChannels.tlsSend,
    async (event, msgType: unknown, payload: unknown, timeout?: unknown) => {
      assertTrustedSender(event, getMainWindow())
      return tlsClient.send(
        parseMessageType(msgType),
        parsePayload(payload),
        parseTimeout(timeout),
      )
    },
  )

  ipcMain.handle(IpcChannels.tlsApplyStateEvent, (event, stateEvent: unknown) => {
    assertTrustedSender(event, getMainWindow())
    tlsClient.transitionState(parseConnectionEvent(stateEvent))
  })

  tlsClient.on('state-change', (from, to, event) => {
    const mainWindow = getMainWindow()
    if (mainWindow != null && !mainWindow.isDestroyed()) {
      mainWindow.webContents.send(IpcChannels.connectionStateChanged, to)
    }
  })
}
