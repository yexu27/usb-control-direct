import { beforeEach, describe, expect, it, vi } from 'vitest'
import { readFile, writeFile } from 'node:fs/promises'
import { SelectedFileAccess } from '../../../src/main/ipc/selected-file-access'
import { registerDialogIpc } from '../../../src/main/ipc/dialog-ipc'
import { IpcChannels } from '../../../src/shared/ipc-channels'

const { handle } = vi.hoisted(() => ({ handle: vi.fn() }))

vi.mock('electron', () => ({
  ipcMain: { handle },
  dialog: { showOpenDialog: vi.fn(), showSaveDialog: vi.fn() },
}))

vi.mock('node:fs/promises', () => ({
  readFile: vi.fn(),
  writeFile: vi.fn(),
}))

const readFileMock = vi.mocked(readFile)
const writeFileMock = vi.mocked(writeFile)
const SELECTED_PATH = process.platform === 'win32' ? 'C:\\用户\\授权文件.txt' : '/tmp/授权文件.txt'
const revokeSpy = vi.spyOn(SelectedFileAccess.prototype, 'revoke')

describe('SelectedFileAccess', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    handle.mockReset()
  })

  it('只允许读取用户选择过的绝对路径且权限仅消费一次', async () => {
    const access = new SelectedFileAccess()
    access.allowRead([SELECTED_PATH])
    readFileMock.mockResolvedValue(Buffer.from([0x01, 0x02]))

    await expect(access.readSelectedFile(SELECTED_PATH)).resolves.toEqual(
      new Uint8Array([0x01, 0x02]),
    )
    await expect(access.readSelectedFile(SELECTED_PATH)).rejects.toThrow('文件路径未经用户选择')
  })

  it('拒绝读取未经用户选择的路径', async () => {
    const access = new SelectedFileAccess()

    await expect(access.readSelectedFile(SELECTED_PATH)).rejects.toThrow('文件路径未经用户选择')
  })

  it('只允许向用户选择过的路径写入 Uint8Array', async () => {
    const access = new SelectedFileAccess()
    access.allowWrite(SELECTED_PATH)

    await access.writeSelectedFile(SELECTED_PATH, new Uint8Array([0x03]))

    expect(writeFileMock).toHaveBeenCalledWith(SELECTED_PATH, new Uint8Array([0x03]))
  })

  it('拒绝无效的写入内容', async () => {
    const access = new SelectedFileAccess()
    access.allowWrite(SELECTED_PATH)

    await expect(access.writeSelectedFile(SELECTED_PATH, 'invalid')).rejects.toThrow(
      '写入内容格式无效',
    )
  })

  it('撤销写权限后拒绝写入', async () => {
    const access = new SelectedFileAccess()
    access.allowWrite(SELECTED_PATH)

    access.revoke(SELECTED_PATH)

    await expect(
      access.writeSelectedFile(SELECTED_PATH, new Uint8Array([0x04])),
    ).rejects.toThrow('文件路径未经用户选择')
    expect(writeFileMock).not.toHaveBeenCalled()
  })

  it('撤销其他绝对路径时保持幂等', () => {
    const access = new SelectedFileAccess()

    expect(() => access.revoke(SELECTED_PATH)).not.toThrow()
    expect(() => access.revoke(SELECTED_PATH)).not.toThrow()
  })

  it('撤销时拒绝相对路径和非字符串', () => {
    const access = new SelectedFileAccess()

    expect(() => access.revoke('../policy.bin')).toThrow('文件路径无效')
    expect(() => access.revoke(null)).toThrow('文件路径无效')
  })
})

describe('registerDialogIpc revoke-file-access', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    handle.mockReset()
  })

  function registerRevokeHandler() {
    const mainFrame = {}
    const webContents = { mainFrame }
    const mainWindow = { isDestroyed: () => false, webContents }
    registerDialogIpc(() => mainWindow as never)
    const registration = handle.mock.calls.find(
      ([channel]) => channel === IpcChannels.dialogRevokeFileAccess,
    )
    return {
      handler: registration?.[1] as (event: unknown, path: unknown) => void,
      mainFrame,
      webContents,
    }
  }

  it('信任 sender 后将绝对路径传给权限撤销器', () => {
    const { handler, mainFrame, webContents } = registerRevokeHandler()

    expect(() => handler({ sender: webContents, senderFrame: mainFrame }, SELECTED_PATH))
      .not.toThrow()
    expect(revokeSpy).toHaveBeenCalledWith(SELECTED_PATH)
  })

  it('不可信 sender 在调用撤销器前被拒绝', () => {
    const { handler } = registerRevokeHandler()

    expect(() => handler({ sender: {}, senderFrame: {} }, '../policy.bin')).toThrow(
      '拒绝来自非主窗口的 IPC 请求',
    )
    expect(revokeSpy).not.toHaveBeenCalled()
  })

  it('信任 sender 的无效路径错误原样传播', () => {
    const { handler, mainFrame, webContents } = registerRevokeHandler()

    expect(() => handler({ sender: webContents, senderFrame: mainFrame }, '../policy.bin'))
      .toThrow('文件路径无效')
    expect(revokeSpy).toHaveBeenCalledWith('../policy.bin')
  })
})
