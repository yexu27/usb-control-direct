import { EventEmitter } from 'events'
import type { ConnectionStatus } from '../shared/connection-state'
import type { TlsResponse } from '../shared/tls-response'
import { TlsTransport } from './tls/tls-transport'
import { FrameStreamParser, encodeFrame } from './tls/frame-codec'
import { RequestDispatcher } from './tls/request-dispatcher'
import { HeartbeatManager } from './tls/heartbeat'
import { ConnectionStateMachine } from './tls/connection-state'

const MSG_HEARTBEAT_CMD = 0xff01
const MSG_HEARTBEAT_RSP = 0xff02

export class TlsClient extends EventEmitter {
  private transport: TlsTransport
  private parser = new FrameStreamParser()
  private dispatcher: RequestDispatcher
  private heartbeat: HeartbeatManager
  private stateMachine = new ConnectionStateMachine()

  constructor(
    transport: TlsTransport = new TlsTransport(),
    heartbeat: HeartbeatManager = new HeartbeatManager(),
  ) {
    super()
    this.transport = transport
    this.heartbeat = heartbeat

    this.dispatcher = new RequestDispatcher((frame: Buffer) => {
      this.transport.write(frame)
    })

    this.parser.onFrame = (header, payload) => {
      if (header.msgType === MSG_HEARTBEAT_RSP) {
        this.heartbeat.onHeartbeatResponse()
        return
      }
      this.dispatcher.handleResponse(header.seqId, header.msgType, payload)
    }

    this.transport.on('data', (chunk: Buffer) => {
      this.parser.feed(chunk)
    })

    this.transport.on('close', () => {
      this.handleDisconnect('NETWORK_ERROR')
    })

    this.transport.on('error', () => {
      this.handleDisconnect('NETWORK_ERROR')
    })

    this.heartbeat.onTimeout = () => {
      this.handleDisconnect('HEARTBEAT_TIMEOUT')
    }

    this.stateMachine.onStateChange = (from, to, event) => {
      if (from === 'CONNECTED' && to !== 'CONNECTED') {
        this.stopHeartbeat()
      }
      if (to === 'CONNECTED' && from !== 'CONNECTED') {
        this.startHeartbeat()
      }
      this.emit('state-change', from, to, event)
    }
  }

  async connect(host: string, port: number): Promise<void> {
    this.stateMachine.transition('CONNECT_START')

    try {
      await this.transport.connect(host, port)
      this.stateMachine.transition('CONNECT_SUCCESS')
    } catch (err) {
      try {
        this.stateMachine.transition('CONNECT_FAIL')
      } catch {
        // 状态机可能已经被 NETWORK_ERROR 推到 DISCONNECTED
      }
      throw err
    }
  }

  disconnect(): void {
    this.heartbeat.stop()
    this.transport.disconnect()
    this.dispatcher.rejectAll(new Error('主动断开连接'))

    if (this.stateMachine.current !== 'DISCONNECTED') {
      this.stateMachine.transition('LOGOUT')
    }
  }

  async send(msgType: number, payload: Uint8Array, timeout?: number): Promise<TlsResponse> {
    if (!this.transport.isConnected()) {
      throw new Error('装置已断开，请重新连接后再操作')
    }

    return this.dispatcher.dispatch(msgType, payload, timeout)
  }

  getConnectionStatus(): ConnectionStatus {
    return this.stateMachine.current
  }

  transitionState(event: import('../shared/connection-state').ConnectionEvent): void {
    this.stateMachine.transition(event)
  }

  startHeartbeat(): void {
    this.heartbeat.start(async () => {
      const frame = encodeFrame(MSG_HEARTBEAT_CMD, 0, new Uint8Array(0))
      this.transport.write(frame)
    })
  }

  stopHeartbeat(): void {
    this.heartbeat.stop()
  }

  private handleDisconnect(
    event: 'NETWORK_ERROR' | 'HEARTBEAT_TIMEOUT',
  ): void {
    this.heartbeat.stop()
    this.dispatcher.rejectAll(new Error(event === 'HEARTBEAT_TIMEOUT' ? '心跳超时' : '网络断开'))

    if (this.stateMachine.current !== 'DISCONNECTED') {
      try {
        this.stateMachine.transition(event)
      } catch {
        // 已经 DISCONNECTED
      }
    }

    if (event === 'HEARTBEAT_TIMEOUT') {
      this.transport.disconnect()
    }
  }
}
