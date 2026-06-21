import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { createServer, type Server, type TLSSocket } from 'node:tls'
import { usb_control } from '../../../src/shared/proto/usb_control'
import { encodeFrame, FrameStreamParser } from '../../../src/main/tls/frame-codec'

export interface MockScenario {
  authStatus: 'authorized' | 'unauthorized' | 'expired'
  loginFailuresBeforeSuccess: number
  role: 'admin' | 'operator' | 'auditor'
  uploadedLicenseValid: boolean
}

const PORT = 9600
const HOST = '127.0.0.1'
const SESSION_TOKEN = 'e2e-session-token'
const EXPIRE_TIME = 1_893_427_200
const QR_CODE_PNG = Buffer.from(
  'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAusB9Y9ZQmcAAAAASUVORK5CYII=',
  'base64',
)

type MessageClass = {
  fromObject(object: Record<string, unknown>): unknown
  encode(message: never): { finish(): Uint8Array }
}

interface MockResponse {
  msgType: number
  messageClass: MessageClass
  body: Record<string, unknown>
}

export class MockDevice {
  private server: Server | null = null
  private readonly sockets = new Set<TLSSocket>()
  private loginAttempts = 0

  constructor(private readonly scenario: MockScenario) {}

  async start(): Promise<void> {
    const fixtureDir = resolve(__dirname, '../fixtures')
    const server = createServer(
      {
        cert: readFileSync(resolve(fixtureDir, 'server.crt')),
        key: readFileSync(resolve(fixtureDir, 'server.key')),
      },
      (socket) => this.handleConnection(socket),
    )
    this.server = server

    await new Promise<void>((resolveStart, reject) => {
      const onError = (error: Error): void => reject(error)
      server.once('error', onError)
      server.listen(PORT, HOST, () => {
        server.off('error', onError)
        resolveStart()
      })
    })
  }

  async stop(): Promise<void> {
    for (const socket of this.sockets) {
      socket.destroy()
    }
    this.sockets.clear()

    const server = this.server
    this.server = null
    if (server == null || !server.listening) {
      return
    }
    await new Promise<void>((resolveStop, reject) => {
      server.close((error) => (error == null ? resolveStop() : reject(error)))
    })
  }

  private handleConnection(socket: TLSSocket): void {
    this.sockets.add(socket)
    socket.once('close', () => this.sockets.delete(socket))

    const parser = new FrameStreamParser()
    parser.onFrame = (header, payload) => {
      const response = this.createResponse(header.msgType, payload)
      if (response == null) {
        socket.destroy(new Error(`E2E Mock 不支持消息类型 0x${header.msgType.toString(16)}`))
        return
      }
      const message = response.messageClass.fromObject(response.body)
      const responsePayload = response.messageClass.encode(message as never).finish()
      socket.write(encodeFrame(response.msgType, header.seqId, responsePayload))
    }
    socket.on('data', (chunk) => parser.feed(chunk))
  }

  private createResponse(msgType: number, payload: Uint8Array): MockResponse | null {
    switch (msgType) {
      case 0x0001:
        return this.loginResponse(payload)
      case 0x0003:
        return this.authStatusResponse()
      case 0x0005:
        return {
          msgType: 0x0006,
          messageClass: usb_control.RspMachineCode,
          body: { machineCode: 'USB-CONTROL-E2E-MACHINE-CODE', qrcodePng: QR_CODE_PNG },
        }
      case 0x0007:
        return this.uploadLicenseResponse(payload)
      case 0x0009:
      case 0x0605:
        return this.commonSuccessResponse()
      case 0x0100:
        return {
          msgType: 0x0101,
          messageClass: usb_control.RspListWhitelist,
          body: { devices: [] },
        }
      case 0x0200:
        return {
          msgType: 0x0201,
          messageClass: usb_control.RspFilePolicy,
          body: {
            execControlEnabled: false,
            autoReadControlEnabled: false,
            fileTypeBlacklistEnabled: false,
            blacklist: [],
          },
        }
      case 0x0500:
        return {
          msgType: 0x0501,
          messageClass: usb_control.RspSystemInfo,
          body: {
            systemVersion: 'e2e-system',
            virusDbVersion: 'e2e-virus-db',
            authorized: true,
            authExpireTime: EXPIRE_TIME,
            deviceDescription: 'E2E Mock Device',
            virusDbUpdatedAt: EXPIRE_TIME,
            authStatus: 'authorized',
          },
        }
      case 0x0600:
        return {
          msgType: 0x0601,
          messageClass: usb_control.RspListUsers,
          body: { users: [] },
        }
      case 0xff01:
        return {
          msgType: 0xff02,
          messageClass: usb_control.RspCommon,
          body: {},
        }
      default:
        return null
    }
  }

  private loginResponse(payload: Uint8Array): MockResponse {
    const command = usb_control.CmdLogin.decode(payload)
    this.loginAttempts += 1
    if (this.loginAttempts <= this.scenario.loginFailuresBeforeSuccess) {
      const locked = this.loginAttempts >= 5
      return {
        msgType: 0x0002,
        messageClass: usb_control.RspLogin,
        body: {
          success: false,
          resultCode: locked ? 0x0004 : 0x0101,
          errorMessage: locked ? '用户已被锁定，请5分钟后重试' : '用户名或密码错误',
        },
      }
    }

    const authorized = this.scenario.authStatus === 'authorized'
    return {
      msgType: 0x0002,
      messageClass: usb_control.RspLogin,
      body: {
        success: true,
        sessionToken: SESSION_TOKEN,
        username: command.username,
        role: this.scenario.role,
        authorized,
        authExpireTime: EXPIRE_TIME,
        deviceDescription: 'E2E Mock Device',
        resultCode: 0,
        errorMessage: '',
        authStatus: this.scenario.authStatus,
      },
    }
  }

  private authStatusResponse(): MockResponse {
    return {
      msgType: 0x0004,
      messageClass: usb_control.RspAuthStatus,
      body: {
        authorized: this.scenario.authStatus === 'authorized',
        expireTime: EXPIRE_TIME,
        deviceDescription: 'E2E Mock Device',
        authStatus: this.scenario.authStatus,
      },
    }
  }

  private uploadLicenseResponse(payload: Uint8Array): MockResponse {
    const command = usb_control.CmdUploadLicense.decode(payload)
    const success = this.scenario.uploadedLicenseValid && command.licenseData.length > 0
    if (success) {
      this.scenario.authStatus = 'authorized'
    }
    return {
      msgType: 0x0008,
      messageClass: usb_control.RspUploadLicense,
      body: {
        success,
        expireTime: EXPIRE_TIME,
        resultCode: success ? 0 : 0x0104,
        errorMessage: success ? '' : '授权文件校验失败',
      },
    }
  }

  private commonSuccessResponse(): MockResponse {
    return {
      msgType: 0xff00,
      messageClass: usb_control.RspCommon,
      body: { success: true, resultCode: 0, errorMessage: '' },
    }
  }
}
