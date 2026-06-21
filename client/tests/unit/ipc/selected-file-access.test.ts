import { beforeEach, describe, expect, it, vi } from 'vitest'
import { readFile, writeFile } from 'node:fs/promises'
import { SelectedFileAccess } from '../../../src/main/ipc/selected-file-access'

vi.mock('node:fs/promises', () => ({
  readFile: vi.fn(),
  writeFile: vi.fn(),
}))

const readFileMock = vi.mocked(readFile)
const writeFileMock = vi.mocked(writeFile)
const SELECTED_PATH = process.platform === 'win32' ? 'C:\\用户\\授权文件.txt' : '/tmp/授权文件.txt'

describe('SelectedFileAccess', () => {
  beforeEach(() => {
    vi.clearAllMocks()
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
})
