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
  importPolicyFails?: boolean
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

interface StoredWhitelistDevice {
  serialNumber: string
  vid: string
  pid: string
  deviceName: string
  capacityBytes: number
  permission: string
  description: string
  addMethod: string
  createdAt: number
  deviceType: string
}

interface StoredBlacklistItem {
  extension: string
  description: string
  isDefault: boolean
  createdAt: number
}

interface StoredPolicy {
  whitelist: StoredWhitelistDevice[]
  filePolicy: {
    execControlEnabled: boolean
    autoReadControlEnabled: boolean
    fileTypeBlacklistEnabled: boolean
    blacklist: StoredBlacklistItem[]
  }
}

const POLICY_MAGIC = Buffer.from('USBPOLICY\n', 'ascii')
const POLICY_VERSION = Buffer.from('VERSION:1\n', 'ascii')
const POLICY_HEADER = Buffer.concat([POLICY_MAGIC, POLICY_VERSION])
const CREATED_AT = 1_767_225_600

function initialPolicy(): StoredPolicy {
  return {
    whitelist: [{
      serialNumber: 'WL-EXISTING-001', vid: '0951', pid: '1666',
      deviceName: 'Existing USB', capacityBytes: 16_000_000_000,
      permission: 'readonly', description: '初始白名单', addMethod: 'device',
      createdAt: CREATED_AT, deviceType: 'storage',
    }],
    filePolicy: {
      execControlEnabled: false,
      autoReadControlEnabled: false,
      fileTypeBlacklistEnabled: false,
      blacklist: [{ extension: '.jse', description: 'JScript Encoded Script', isDefault: true, createdAt: CREATED_AT }],
    },
  }
}

export class MockDevice {
  private server: Server | null = null
  private readonly sockets = new Set<TLSSocket>()
  private loginAttempts = 0
  private policy = initialPolicy()
  private connectedDevices = [{
    serialNumber: 'DEVICE-ADDABLE-001', deviceName: 'Device-side USB', vid: '0781', pid: '5591',
    capacityBytes: 32_000_000_000, deviceType: 'storage', interfaceType: 'mass_storage',
    admissionStatus: 'addable', failReason: '',
  }, {
    serialNumber: 'DEVICE-REMOVED-002', deviceName: 'Soon removed USB', vid: '0781', pid: '5592',
    capacityBytes: 16_000_000_000, deviceType: 'storage', interfaceType: 'mass_storage',
    admissionStatus: 'addable', failReason: '',
  }]

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

  removeConnectedDevice(serialNumber: string): void {
    this.connectedDevices = this.connectedDevices.filter((device) => device.serialNumber !== serialNumber)
  }

  failNextImport(): void {
    this.scenario.importPolicyFails = true
  }

  disconnectSockets(): void {
    for (const socket of this.sockets) socket.destroy()
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
          body: { devices: this.policy.whitelist },
        }
      case 0x0102:
        return {
          msgType: 0x0103,
          messageClass: usb_control.RspConnectedDevices,
          body: {
            devices: this.connectedDevices.filter((device) =>
              device.admissionStatus === 'addable' &&
              !this.policy.whitelist.some((item) => item.serialNumber === device.serialNumber),
            ),
          },
        }
      case 0x0104:
        return this.addWhitelistResponse(payload)
      case 0x0105:
        return this.removeWhitelistResponse(payload)
      case 0x0106:
        return this.updateWhitelistResponse(payload)
      case 0x0200:
        return {
          msgType: 0x0201,
          messageClass: usb_control.RspFilePolicy,
          body: this.policy.filePolicy,
        }
      case 0x0202:
        return this.updateFilePolicySwitchResponse(payload)
      case 0x0203:
        return this.addBlacklistResponse(payload)
      case 0x0204:
        return this.removeBlacklistResponse(payload)
      case 0x0300:
        return {
          msgType: 0x0301,
          messageClass: usb_control.RspExportPolicy,
          body: { success: true, policyData: this.encodePolicy(), resultCode: 0, errorMessage: '' },
        }
      case 0x0302:
        return this.importPolicyResponse(payload)
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
          messageClass: usb_control.RspHeartbeat,
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

  private commonErrorResponse(resultCode: number, errorMessage: string): MockResponse {
    return {
      msgType: 0xff00,
      messageClass: usb_control.RspCommon,
      body: { success: false, resultCode, errorMessage },
    }
  }

  private addWhitelistResponse(payload: Uint8Array): MockResponse {
    const command = usb_control.CmdAddWhitelist.decode(payload)
    if (command.serialNumber.trim() === '') return this.commonErrorResponse(0x0201, '序列号不能为空')
    if (this.policy.whitelist.some((device) => device.serialNumber === command.serialNumber)) {
      return this.commonErrorResponse(0x0202, '该设备已在白名单中')
    }
    if (command.addMethod === 'device' && !this.connectedDevices.some(
      (device) => device.serialNumber === command.serialNumber && device.admissionStatus === 'addable',
    )) return this.commonErrorResponse(0x0005, '设备已移除，请重新插入后再添加')
    this.policy.whitelist.push({
      serialNumber: command.serialNumber, vid: command.vid, pid: command.pid,
      deviceName: command.deviceName, capacityBytes: Number(command.capacityBytes),
      permission: command.permission, description: command.description,
      addMethod: command.addMethod, createdAt: CREATED_AT, deviceType: command.deviceType,
    })
    return this.commonSuccessResponse()
  }

  private removeWhitelistResponse(payload: Uint8Array): MockResponse {
    const command = usb_control.CmdRemoveWhitelist.decode(payload)
    const index = this.policy.whitelist.findIndex((device) => device.serialNumber === command.serialNumber)
    if (index < 0) return this.commonErrorResponse(0x0203, '白名单设备不存在')
    this.policy.whitelist.splice(index, 1)
    return this.commonSuccessResponse()
  }

  private updateWhitelistResponse(payload: Uint8Array): MockResponse {
    const command = usb_control.CmdUpdateWhitelist.decode(payload)
    const device = this.policy.whitelist.find((item) => item.serialNumber === command.serialNumber)
    if (device == null) return this.commonErrorResponse(0x0203, '白名单设备不存在')
    if (command.permission !== '') device.permission = command.permission
    if (command.description !== '') device.description = command.description
    return this.commonSuccessResponse()
  }

  private updateFilePolicySwitchResponse(payload: Uint8Array): MockResponse {
    const command = usb_control.CmdUpdateFilePolicySwitch.decode(payload)
    const fields = {
      exec_control: 'execControlEnabled', auto_read_control: 'autoReadControlEnabled',
      file_type_blacklist_control: 'fileTypeBlacklistEnabled',
    } as const
    const field = fields[command.policyKey as keyof typeof fields]
    if (field == null) return this.commonErrorResponse(0x0301, '无效的策略键')
    this.policy.filePolicy[field] = command.enabled
    return this.commonSuccessResponse()
  }

  private addBlacklistResponse(payload: Uint8Array): MockResponse {
    const command = usb_control.CmdAddBlacklistExtension.decode(payload)
    const extension = command.extension.trim().toLowerCase()
    if (!/^\.[a-z0-9]{1,10}$/.test(extension)) return this.commonErrorResponse(0x0302, '文件后缀格式错误')
    if (this.policy.filePolicy.blacklist.some((item) => item.extension === extension)) {
      return this.commonErrorResponse(0x0303, '该文件后缀已在黑名单中')
    }
    this.policy.filePolicy.blacklist.push({ extension, description: command.description, isDefault: false, createdAt: CREATED_AT })
    return this.commonSuccessResponse()
  }

  private removeBlacklistResponse(payload: Uint8Array): MockResponse {
    const command = usb_control.CmdRemoveBlacklistExtension.decode(payload)
    const extension = command.extension.trim().toLowerCase()
    const index = this.policy.filePolicy.blacklist.findIndex((item) => item.extension === extension)
    if (index < 0) return this.commonErrorResponse(0x0304, '该文件后缀不在黑名单中')
    this.policy.filePolicy.blacklist.splice(index, 1)
    return this.commonSuccessResponse()
  }

  private encodePolicy(): Uint8Array {
    return Buffer.concat([POLICY_HEADER, Buffer.from(JSON.stringify(this.policy), 'utf8')])
  }

  private importPolicyResponse(payload: Uint8Array): MockResponse {
    const command = usb_control.CmdImportPolicy.decode(payload)
    if (this.scenario.importPolicyFails) {
      this.scenario.importPolicyFails = false
      return this.commonErrorResponse(0x0407, '策略导入失败，原策略未变更')
    }
    const bytes = Buffer.from(command.policyData)
    if (!bytes.subarray(0, POLICY_MAGIC.length).equals(POLICY_MAGIC) ||
      !bytes.subarray(POLICY_MAGIC.length, POLICY_HEADER.length).equals(POLICY_VERSION)) {
      return this.commonErrorResponse(0x0402, '策略文件格式错误')
    }
    try {
      const parsed = JSON.parse(bytes.subarray(POLICY_HEADER.length).toString('utf8')) as unknown
      if (!isStoredPolicy(parsed)) throw new Error('invalid')
      this.policy = structuredClone(parsed)
      return this.commonSuccessResponse()
    } catch {
      return this.commonErrorResponse(0x0402, '策略文件格式错误')
    }
  }
}

function isStoredPolicy(value: unknown): value is StoredPolicy {
  if (value == null || typeof value !== 'object') return false
  const candidate = value as Partial<StoredPolicy>
  if (!Array.isArray(candidate.whitelist) || candidate.filePolicy == null ||
    typeof candidate.filePolicy !== 'object' || !Array.isArray(candidate.filePolicy.blacklist) ||
    typeof candidate.filePolicy.execControlEnabled !== 'boolean' ||
    typeof candidate.filePolicy.autoReadControlEnabled !== 'boolean' ||
    typeof candidate.filePolicy.fileTypeBlacklistEnabled !== 'boolean') return false
  return candidate.whitelist.every((device) =>
    device != null && typeof device === 'object' &&
    typeof device.serialNumber === 'string' && device.serialNumber.trim() !== '' &&
    typeof device.vid === 'string' && typeof device.pid === 'string' &&
    typeof device.deviceName === 'string' && Number.isFinite(device.capacityBytes) &&
    typeof device.permission === 'string' && typeof device.description === 'string' &&
    typeof device.addMethod === 'string' && Number.isFinite(device.createdAt) &&
    typeof device.deviceType === 'string',
  ) && candidate.filePolicy.blacklist.every((item) =>
    item != null && typeof item === 'object' && typeof item.extension === 'string' &&
    /^\.[a-z0-9]{1,10}$/.test(item.extension) && typeof item.description === 'string' &&
    typeof item.isDefault === 'boolean' && Number.isFinite(item.createdAt),
  )
}
