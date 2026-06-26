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
  sessionExpiresAfterLogin?: boolean
}

const PORT = 9600
const HOST = '127.0.0.1'
const EXPIRE_TIME = 1_893_427_200
const QR_CODE_PNG = Buffer.from(
  'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAusB9Y9ZQmcAAAAASUVORK5CYII=',
  'base64',
)

type MessageClass = {
  fromObject(object: Record<string, unknown>): unknown
  encode(message: never): { finish(): Uint8Array }
  decode(reader: Uint8Array): unknown
}

interface MockResponse {
  msgType: number
  messageClass: MessageClass
  body: Record<string, unknown>
  disconnectAfterResponse?: boolean
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

interface StoredUser {
  username: string
  role: MockScenario['role']
  status: 'active' | 'locked'
  isBuiltin: boolean
  createdAt: number
  password: string
}

const POLICY_MAGIC = Buffer.from('USBPOLICY\n', 'ascii')
const POLICY_VERSION = Buffer.from('VERSION:1\n', 'ascii')
const POLICY_HEADER = Buffer.concat([POLICY_MAGIC, POLICY_VERSION])
const POLICY_HEADER_CRLF = Buffer.from('USBPOLICY\r\nVERSION:1\r\n', 'ascii')
const CREATED_AT = 1_767_225_600

interface MockLogFilter {
  keyword: string
  eventType: string
}

function stringifyLogValue(value: unknown): string {
  if (value == null) {
    return ''
  }
  if (value instanceof Uint8Array) {
    return ''
  }
  return String(value)
}

function matchesKeyword(entry: Record<string, unknown>, keyword: string): boolean {
  const normalizedKeyword = keyword.trim().toLowerCase()
  if (normalizedKeyword === '') {
    return true
  }
  return Object.values(entry).some((value) =>
    stringifyLogValue(value).toLowerCase().includes(normalizedKeyword),
  )
}

export function filterMockLogsForQuery<TEntry extends object>(
  entries: TEntry[],
  filter: MockLogFilter,
): TEntry[] {
  return entries.filter((entry) => {
    const logEntry = entry as Record<string, unknown>
    if (!matchesKeyword(logEntry, filter.keyword)) {
      return false
    }
    if (filter.eventType !== '' && logEntry.eventType !== filter.eventType) {
      return false
    }
    return true
  })
}

function paginateMockLogs<TEntry>(entries: TEntry[], page: number, pageSize: number): TEntry[] {
  const safePage = Number.isFinite(page) && page > 0 ? Math.trunc(page) : 1
  const safePageSize = Number.isFinite(pageSize) && pageSize > 0 ? Math.trunc(pageSize) : entries.length
  const start = (safePage - 1) * safePageSize
  return entries.slice(start, start + safePageSize)
}

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
  private activeSocket: TLSSocket | null = null
  private readonly fallbackRole: MockScenario['role']
  private currentUsername = ''
  private loginAttempts = 0
  private activeSessionToken = ''
  private invalidatedSessionTokens = new Set<string>()
  private sessionExpired = false
  private policy = initialPolicy()
  private deletedUsernames = new Set<string>()
  private users: StoredUser[] = [
    { username: 'admin', role: 'admin', status: 'active', isBuiltin: true, createdAt: 0, password: 'admin@123' },
    { username: 'operator', role: 'operator', status: 'active', isBuiltin: true, createdAt: 0, password: 'operator@123' },
    { username: 'audit', role: 'auditor', status: 'active', isBuiltin: true, createdAt: 0, password: 'audit@123' },
  ]
  private systemInfo = {
    systemVersion: 'v1.0.0',
    virusDbVersion: 'v3.0.3',
    virusDbUpdatedAt: EXPIRE_TIME,
    deviceDescription: 'E2E_MOCK_DEVICE',
  }
  private usbAuditLogs = [{
    id: 1,
    eventTime: 1_767_225_610,
    deviceSn: 'USB-AUDIT-001',
    deviceName: 'Kingston DataTraveler',
    deviceType: 'storage',
    interfaceType: 'mass_storage',
    eventType: 'insert_success',
    permission: 'readwrite',
    capacityBytes: 32_000_000_000,
    detail: '授权设备',
  }, {
    id: 2,
    eventTime: 1_767_225_611,
    deviceSn: 'USB-AUDIT-002',
    deviceName: 'Blocked USB',
    deviceType: 'storage',
    interfaceType: 'mass_storage',
    eventType: 'insert_success',
    permission: '',
    capacityBytes: 0,
    detail: '未授权设备',
  }, {
    id: 3,
    eventTime: 1_767_225_612,
    deviceSn: '',
    deviceName: 'Lenovo Keyboard',
    deviceType: 'keyboard',
    interfaceType: 'hid_keyboard',
    eventType: 'insert_success',
    permission: '',
    capacityBytes: 0,
    detail: '键盘',
  }, {
    id: 4,
    eventTime: 1_767_225_613,
    deviceSn: '',
    deviceName: 'Lenovo Mouse',
    deviceType: 'mouse',
    interfaceType: 'hid_mouse',
    eventType: 'insert_success',
    permission: '',
    capacityBytes: 0,
    detail: '鼠标',
  }, {
    id: 5,
    eventTime: 1_767_225_614,
    deviceSn: 'USB-AUDIT-001',
    deviceName: 'Kingston DataTraveler',
    deviceType: 'storage',
    interfaceType: 'mass_storage',
    eventType: 'device_remove',
    permission: 'readonly',
    capacityBytes: 32_000_000_000,
    detail: '授权设备',
  }]
  private malwareLogs = [{
    id: 1,
    scanTime: 1_767_225_620,
    deviceSn: 'USB-MALWARE-001',
    deviceName: 'SanDisk',
    filePath: '/mnt/usb/eicar.com',
    scanResult: 'infected',
    virusName: 'EICAR-Test-File',
    processResult: 'blocked',
    detail: '发现病毒并阻断',
  }, {
    id: 2,
    scanTime: 1_767_225_621,
    deviceSn: 'USB-MALWARE-002',
    deviceName: 'Clean USB',
    filePath: '/mnt/usb/readme.txt',
    scanResult: 'clean',
    virusName: '',
    processResult: 'no_action',
    detail: '扫描通过',
  }, {
    id: 3,
    scanTime: 1_767_225_622,
    deviceSn: 'USB-MALWARE-003',
    deviceName: 'Broken USB',
    filePath: '',
    scanResult: 'failed',
    virusName: '',
    processResult: 'failed',
    failReason: '病毒库不可用',
    detail: '旧失败详情',
  }, {
    id: 4,
    scanTime: 1_767_225_623,
    deviceSn: 'USB-MALWARE-004',
    deviceName: 'Removed USB',
    filePath: '',
    scanResult: 'cancelled',
    virusName: '',
    processResult: 'no_action',
    failReason: '设备拔出',
    detail: '',
  }]
  private operationLogs = [{
    id: 1,
    opTime: 1_767_225_630,
    username: 'admin',
    role: 'admin',
    logCategory: 'user_management',
    actionType: 'user_create',
    target: 'new_operator',
    result: '0',
    failReason: '',
    detail: '',
  }, {
    id: 2,
    opTime: 1_767_225_631,
    username: 'audit',
    role: 'auditor',
    logCategory: 'login_auth',
    actionType: 'login',
    target: 'audit',
    result: '0',
    failReason: '',
    detail: '',
  }, {
    id: 3,
    opTime: 1_767_225_632,
    username: 'operator',
    role: 'operator',
    logCategory: 'security_config',
    actionType: 'whitelist_update',
    target: 'USB-AUDIT-001',
    beforeValue: '{"permission":"readonly"}',
    afterValue: '{"permission":"readwrite"}',
    result: '0',
    failReason: '',
    detail: '',
  }]
  private connectedDevices = [{
    serialNumber: 'DEVICE-ADDABLE-001', deviceName: 'Device-side USB', vid: '0781', pid: '5591',
    capacityBytes: 32_000_000_000, deviceType: 'storage', interfaceType: 'mass_storage',
    admissionStatus: 'addable', failReason: '',
  }, {
    serialNumber: 'DEVICE-REMOVED-002', deviceName: 'Soon removed USB', vid: '0781', pid: '5592',
    capacityBytes: 16_000_000_000, deviceType: 'storage', interfaceType: 'mass_storage',
    admissionStatus: 'addable', failReason: '',
  }]
  public systemUpgradeUploadCount = 0
  public virusdbUpgradeUploadCount = 0
  public licenseUploadCount = 0
  public listUsersCount = 0

  constructor(private readonly scenario: MockScenario) {
    const { role } = scenario
    this.fallbackRole = role
  }

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

  expireCurrentSessionForTest(): void {
    this.sessionExpired = true
  }

  setAuthStatusForTest(authStatus: MockScenario['authStatus']): void {
    this.scenario.authStatus = authStatus
  }

  async requestForTest(msgType: number, body: Record<string, unknown>): Promise<Record<string, unknown>> {
    const payload = this.encodeCommandForTest(msgType, body)
    const response = this.createResponse(msgType, payload)
    if (response == null) {
      throw new Error(`E2E Mock 不支持消息类型 0x${msgType.toString(16)}`)
    }
    const message = response.messageClass.fromObject(response.body)
    const responsePayload = response.messageClass.encode(message as never).finish()
    const decoded = response.messageClass.decode(responsePayload) as Record<string, unknown>
    if (response.msgType === 0xff00 && decoded.success === false) {
      throw new Error(String(decoded.errorMessage || '操作失败'))
    }
    return decoded
  }

  disconnectSockets(): void {
    for (const socket of this.sockets) socket.destroy()
  }

  private handleConnection(socket: TLSSocket): void {
    if (this.activeSocket != null && !this.activeSocket.destroyed) {
      socket.destroy(new Error('装置已有管理端连接，请稍后重试'))
      return
    }

    this.activeSocket = socket
    this.sockets.add(socket)
    socket.once('close', () => {
      this.sockets.delete(socket)
      if (this.activeSocket === socket) {
        this.activeSocket = null
      }
    })

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
      if (response.disconnectAfterResponse) {
        setTimeout(() => this.disconnectSockets(), 50)
      }
    }
    socket.on('data', (chunk) => parser.feed(chunk))
  }

  private createResponse(msgType: number, payload: Uint8Array): MockResponse | null {
    if (msgType !== 0x0001 && msgType !== 0xff01) {
      const sessionToken = this.sessionTokenForCommand(msgType, payload)
      if (sessionToken == null) {
        return null
      }
      const sessionError = this.validateSessionToken(sessionToken)
      if (sessionError != null) {
        return sessionError
      }
    }

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
        return this.logoutResponse(payload)
      case 0x0400:
        return this.queryLogsResponse(payload)
      case 0x0402:
        return this.exportLogsResponse()
      case 0x0404:
        return this.deleteLogsResponse(payload)
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
            systemVersion: this.systemInfo.systemVersion,
            virusDbVersion: this.systemInfo.virusDbVersion,
            authorized: true,
            authExpireTime: EXPIRE_TIME,
            deviceDescription: this.systemInfo.deviceDescription,
            virusDbUpdatedAt: this.systemInfo.virusDbUpdatedAt,
            authStatus: 'authorized',
          },
        }
      case 0x0502:
        return this.uploadSystemUpgradeResponse(payload)
      case 0x0503:
        return this.uploadVirusdbUpgradeResponse(payload)
      case 0x0504:
        return this.updateDeviceDescriptionResponse(payload)
      case 0x0600:
        this.listUsersCount += 1
        return {
          msgType: 0x0601,
          messageClass: usb_control.RspListUsers,
          body: { users: this.users.map(({ password: _password, ...user }) => user) },
        }
      case 0x0602:
        return this.createUserResponse(payload)
      case 0x0603:
        return this.deleteUserResponse(payload)
      case 0x0604:
        return this.resetPasswordResponse(payload)
      case 0x0605:
        return this.changePasswordResponse(payload)
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

    const user = this.users.find((item) => item.username === command.username)
    const role = user?.role ?? this.fallbackRole
    const passwordMatches = user == null
      ? command.password === 'Password1!'
      : command.password === user.password
    if (!passwordMatches) {
      return {
        msgType: 0x0002,
        messageClass: usb_control.RspLogin,
        body: { success: false, resultCode: 0x0101, errorMessage: '用户名或密码错误' },
      }
    }
    if (user?.status === 'locked') {
      return {
        msgType: 0x0002,
        messageClass: usb_control.RspLogin,
        body: { success: false, resultCode: 0x0004, errorMessage: '用户已被锁定，请5分钟后重试' },
      }
    }

    const authorized = this.scenario.authStatus === 'authorized'
    this.currentUsername = command.username
    const sessionToken = `e2e-session-token-${Date.now()}-${this.loginAttempts}`
    this.activeSessionToken = sessionToken
    this.invalidatedSessionTokens.delete(sessionToken)
    this.sessionExpired = this.scenario.sessionExpiresAfterLogin === true
    return {
      msgType: 0x0002,
      messageClass: usb_control.RspLogin,
      body: {
        success: true,
        sessionToken,
        username: command.username,
        role,
        authorized,
        authExpireTime: EXPIRE_TIME,
        deviceDescription: this.systemInfo.deviceDescription,
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
        deviceDescription: this.systemInfo.deviceDescription,
        authStatus: this.scenario.authStatus,
      },
    }
  }

  private logoutResponse(payload: Uint8Array): MockResponse {
    const command = usb_control.CmdLogout.decode(payload)
    this.invalidatedSessionTokens.add(command.sessionToken)
    if (this.activeSessionToken === command.sessionToken) {
      this.activeSessionToken = ''
    }
    this.currentUsername = ''
    return this.commonSuccessResponse()
  }

  private validateSessionToken(sessionToken: string): MockResponse | null {
    if (
      sessionToken === '' ||
      sessionToken !== this.activeSessionToken ||
      this.invalidatedSessionTokens.has(sessionToken) ||
      this.sessionExpired
    ) {
      return this.commonErrorResponse(0x0001, '会话已失效')
    }
    return null
  }

  private sessionTokenForCommand(msgType: number, payload: Uint8Array): string | null {
    switch (msgType) {
      case 0x0003:
        return usb_control.CmdAuthStatusQuery.decode(payload).sessionToken
      case 0x0005:
        return usb_control.CmdGetMachineCode.decode(payload).sessionToken
      case 0x0007:
        return usb_control.CmdUploadLicense.decode(payload).sessionToken
      case 0x0009:
        return usb_control.CmdLogout.decode(payload).sessionToken
      case 0x0100:
        return usb_control.CmdListWhitelist.decode(payload).sessionToken
      case 0x0102:
        return usb_control.CmdGetConnectedDevices.decode(payload).sessionToken
      case 0x0104:
        return usb_control.CmdAddWhitelist.decode(payload).sessionToken
      case 0x0105:
        return usb_control.CmdRemoveWhitelist.decode(payload).sessionToken
      case 0x0106:
        return usb_control.CmdUpdateWhitelist.decode(payload).sessionToken
      case 0x0200:
        return usb_control.CmdGetFilePolicy.decode(payload).sessionToken
      case 0x0202:
        return usb_control.CmdUpdateFilePolicySwitch.decode(payload).sessionToken
      case 0x0203:
        return usb_control.CmdAddBlacklistExtension.decode(payload).sessionToken
      case 0x0204:
        return usb_control.CmdRemoveBlacklistExtension.decode(payload).sessionToken
      case 0x0300:
        return usb_control.CmdExportPolicy.decode(payload).sessionToken
      case 0x0302:
        return usb_control.CmdImportPolicy.decode(payload).sessionToken
      case 0x0400:
        return usb_control.CmdQueryLogs.decode(payload).sessionToken
      case 0x0402:
        return usb_control.CmdExportLogs.decode(payload).sessionToken
      case 0x0404:
        return usb_control.CmdDeleteLogs.decode(payload).sessionToken
      case 0x0500:
        return usb_control.CmdGetSystemInfo.decode(payload).sessionToken
      case 0x0502:
        return usb_control.CmdUploadSystemUpgrade.decode(payload).sessionToken
      case 0x0503:
        return usb_control.CmdUploadVirusdbUpgrade.decode(payload).sessionToken
      case 0x0504:
        return usb_control.CmdUpdateDeviceDesc.decode(payload).sessionToken
      case 0x0600:
        return usb_control.CmdListUsers.decode(payload).sessionToken
      case 0x0602:
        return usb_control.CmdCreateUser.decode(payload).sessionToken
      case 0x0603:
        return usb_control.CmdDeleteUser.decode(payload).sessionToken
      case 0x0604:
        return usb_control.CmdResetPassword.decode(payload).sessionToken
      case 0x0605:
        return usb_control.CmdChangePassword.decode(payload).sessionToken
      default:
        return null
    }
  }

  private encodeCommandForTest(msgType: number, body: Record<string, unknown>): Uint8Array {
    switch (msgType) {
      case 0x0001:
        return usb_control.CmdLogin.encode(usb_control.CmdLogin.fromObject(body)).finish()
      case 0x0003:
        return usb_control.CmdAuthStatusQuery.encode(
          usb_control.CmdAuthStatusQuery.fromObject(body),
        ).finish()
      case 0x0009:
        return usb_control.CmdLogout.encode(usb_control.CmdLogout.fromObject(body)).finish()
      default:
        throw new Error(`requestForTest 不支持消息类型 0x${msgType.toString(16)}`)
    }
  }

  private uploadLicenseResponse(payload: Uint8Array): MockResponse {
    this.licenseUploadCount += 1
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

  private queryLogsResponse(payload: Uint8Array): MockResponse {
    const command = usb_control.CmdQueryLogs.decode(payload)
    const filter = {
      keyword: command.keyword,
      eventType: command.eventType,
    }
    const usbAuditEntries = command.logType === 'usb_audit'
      ? filterMockLogsForQuery(this.usbAuditLogs, filter)
      : []
    const malwareEntries = command.logType === 'malware'
      ? filterMockLogsForQuery(this.malwareLogs, filter)
      : []
    const operationEntries = command.logType === 'operation'
      ? filterMockLogsForQuery(this.operationLogs, filter)
      : []
    return {
      msgType: 0x0401,
      messageClass: usb_control.RspQueryLogs,
      body: {
        success: true,
        total: this.filteredLogCount(command.logType, {
          usbAuditEntries,
          malwareEntries,
          operationEntries,
        }),
        usbAuditEntries: paginateMockLogs(usbAuditEntries, command.page, command.pageSize),
        malwareEntries: paginateMockLogs(malwareEntries, command.page, command.pageSize),
        operationEntries: paginateMockLogs(operationEntries, command.page, command.pageSize),
        resultCode: 0,
        errorMessage: '',
      },
    }
  }

  private exportLogsResponse(): MockResponse {
    return {
      msgType: 0x0403,
      messageClass: usb_control.RspExportLogs,
      body: {
        success: true,
        zipData: Buffer.from('PK\u0003\u0004E2E-LOG-ZIP', 'binary'),
        suggestedFilename: 'USBUsageLog20260622120000.zip',
        resultCode: 0,
        errorMessage: '',
      },
    }
  }

  private deleteLogsResponse(payload: Uint8Array): MockResponse {
    const command = usb_control.CmdDeleteLogs.decode(payload)
    const sixMonthsAgo = Math.floor(Date.now() / 1000) - 183 * 24 * 60 * 60
    if (Number(command.endTime) >= sixMonthsAgo) {
      return this.commonErrorResponse(0x0501, '半年内的日志不可清理')
    }
    return this.commonSuccessResponse()
  }

  private filteredLogCount(
    logType: string,
    entries: {
      usbAuditEntries: unknown[]
      malwareEntries: unknown[]
      operationEntries: unknown[]
    },
  ): number {
    if (logType === 'usb_audit') {
      return entries.usbAuditEntries.length
    }
    if (logType === 'malware') {
      return entries.malwareEntries.length
    }
    if (logType === 'operation') {
      return entries.operationEntries.length
    }
    return 0
  }

  private uploadSystemUpgradeResponse(payload: Uint8Array): MockResponse {
    this.systemUpgradeUploadCount += 1
    const command = usb_control.CmdUploadSystemUpgrade.decode(payload)
    if (compareVersions(command.targetVersion, this.systemInfo.systemVersion) <= 0) {
      return this.commonErrorResponse(0x0601, '升级包版本低于当前版本')
    }
    if (command.targetVersion === 'v9.9.9') {
      return this.commonErrorResponse(0x0604, '系统升级安装失败')
    }
    this.systemInfo.systemVersion = command.targetVersion
    return { ...this.commonSuccessResponse(), disconnectAfterResponse: true }
  }

  private uploadVirusdbUpgradeResponse(payload: Uint8Array): MockResponse {
    this.virusdbUpgradeUploadCount += 1
    const command = usb_control.CmdUploadVirusdbUpgrade.decode(payload)
    if (command.targetVersion.includes('4')) {
      return this.commonErrorResponse(0x0605, '病毒库版本号命中 4 跳过规则')
    }
    this.systemInfo.virusDbVersion = command.targetVersion
    this.systemInfo.virusDbUpdatedAt = Math.floor(Date.now() / 1000)
    return this.commonSuccessResponse()
  }

  private updateDeviceDescriptionResponse(payload: Uint8Array): MockResponse {
    const command = usb_control.CmdUpdateDeviceDesc.decode(payload)
    if (!/^[A-Za-z0-9_]{1,32}$/.test(command.description)) {
      return this.commonErrorResponse(0x0609, '设备描述格式错误')
    }
    this.systemInfo.deviceDescription = command.description
    return this.commonSuccessResponse()
  }

  private createUserResponse(payload: Uint8Array): MockResponse {
    const command = usb_control.CmdCreateUser.decode(payload)
    if (this.users.some((user) => user.username === command.username)) {
      return this.commonErrorResponse(0x0701, '用户名已存在')
    }
    if (this.deletedUsernames.has(command.username)) {
      return this.commonErrorResponse(0x0702, '该用户名曾被删除，不可再次使用')
    }
    const passwordError = this.validatePassword(command.password, command.confirmPassword)
    if (passwordError != null) {
      return passwordError
    }
    this.users.push({
      username: command.username,
      role: roleFromString(command.role),
      status: 'active',
      isBuiltin: false,
      createdAt: CREATED_AT,
      password: command.password,
    })
    return this.commonSuccessResponse()
  }

  private deleteUserResponse(payload: Uint8Array): MockResponse {
    const command = usb_control.CmdDeleteUser.decode(payload)
    const user = this.users.find((item) => item.username === command.username)
    if (user?.isBuiltin) {
      return this.commonErrorResponse(0x0707, '内置用户不可删除')
    }
    if (user == null) {
      return this.commonErrorResponse(0x0706, '用户不存在')
    }
    this.users = this.users.filter((item) => item.username !== command.username)
    this.deletedUsernames.add(command.username)
    return this.commonSuccessResponse()
  }

  private resetPasswordResponse(payload: Uint8Array): MockResponse {
    const command = usb_control.CmdResetPassword.decode(payload)
    const passwordError = this.validatePassword(command.newPassword, command.confirmPassword)
    if (passwordError != null) {
      return passwordError
    }
    const user = this.users.find((item) => item.username === command.username)
    if (user == null) {
      return this.commonErrorResponse(0x0706, '用户不存在')
    }
    user.password = command.newPassword
    user.status = 'active'
    return this.commonSuccessResponse()
  }

  private changePasswordResponse(payload: Uint8Array): MockResponse {
    const command = usb_control.CmdChangePassword.decode(payload)
    const user = this.users.find((item) => item.username === this.currentUsername)
    const currentPassword = user?.password ?? 'Password1!'

    if (command.oldPassword !== currentPassword) {
      return this.commonErrorResponse(0x0706, '旧密码错误')
    }

    const passwordError = this.validatePassword(command.newPassword, command.confirmPassword)
    if (passwordError != null) {
      return passwordError
    }

    if (user != null) {
      user.password = command.newPassword
    }
    return this.commonSuccessResponse()
  }

  private validatePassword(password: string, confirmPassword: string): MockResponse | null {
    if (password.length < 8 || [/[a-zA-Z]/, /\d/, /[^a-zA-Z0-9]/].filter((pattern) => pattern.test(password)).length < 2) {
      return this.commonErrorResponse(0x0704, '密码复杂度不符合要求')
    }
    if (password !== confirmPassword) {
      return this.commonErrorResponse(0x0705, '两次输入的密码不一致')
    }
    return null
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
    const headerLength = policyHeaderLength(bytes)
    if (headerLength == null) {
      return this.commonErrorResponse(0x0402, '策略文件格式错误')
    }
    try {
      const parsed = JSON.parse(bytes.subarray(headerLength).toString('utf8')) as unknown
      if (!isStoredPolicy(parsed)) throw new Error('invalid')
      this.policy = structuredClone(parsed)
      return this.commonSuccessResponse()
    } catch {
      return this.commonErrorResponse(0x0402, '策略文件格式错误')
    }
  }
}

function compareVersions(left: string, right: string): number {
  const leftParts = parseVersionParts(left)
  const rightParts = parseVersionParts(right)
  for (let index = 0; index < 3; index += 1) {
    const diff = leftParts[index] - rightParts[index]
    if (diff !== 0) {
      return diff
    }
  }
  return 0
}

function parseVersionParts(version: string): [number, number, number] {
  const match = version.match(/^v?(\d+)\.(\d+)\.(\d+)$/)
  if (match == null) {
    return [0, 0, 0]
  }
  return [Number(match[1]), Number(match[2]), Number(match[3])]
}

function roleFromString(role: string): MockScenario['role'] {
  if (role === 'admin' || role === 'operator' || role === 'auditor') {
    return role
  }
  return 'operator'
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

function policyHeaderLength(bytes: Buffer): number | null {
  if (bytes.subarray(0, POLICY_HEADER.length).equals(POLICY_HEADER)) {
    return POLICY_HEADER.length
  }
  if (bytes.subarray(0, POLICY_HEADER_CRLF.length).equals(POLICY_HEADER_CRLF)) {
    return POLICY_HEADER_CRLF.length
  }
  return null
}
