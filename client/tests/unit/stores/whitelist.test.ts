import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useWhitelistStore } from '../../../src/renderer/stores/whitelist'
import {
  addWhitelist,
  listWhitelist,
  removeWhitelist,
  updateWhitelist,
  type AddWhitelistInput,
} from '../../../src/renderer/services/whitelist-service'
import { usb_control } from '../../../src/shared/proto/usb_control'

vi.mock('../../../src/renderer/services/whitelist-service', () => ({
  addWhitelist: vi.fn(),
  listWhitelist: vi.fn(),
  removeWhitelist: vi.fn(),
  updateWhitelist: vi.fn(),
}))

const addWhitelistMock = vi.mocked(addWhitelist)
const listWhitelistMock = vi.mocked(listWhitelist)
const removeWhitelistMock = vi.mocked(removeWhitelist)
const updateWhitelistMock = vi.mocked(updateWhitelist)

function createDevice(serialNumber: string): usb_control.WhitelistDevice {
  return usb_control.WhitelistDevice.fromObject({ serialNumber, permission: 'read_only' })
}

interface Deferred<T> {
  promise: Promise<T>
  resolve: (value: T) => void
  reject: (reason: unknown) => void
}

function createDeferred<T>(): Deferred<T> {
  let resolve: (value: T) => void = () => {
    throw new Error('deferred 尚未初始化')
  }
  let reject: (reason: unknown) => void = () => {
    throw new Error('deferred 尚未初始化')
  }
  const promise = new Promise<T>((promiseResolve, promiseReject) => {
    resolve = promiseResolve
    reject = promiseReject
  })
  return { promise, resolve, reject }
}

const addInput: AddWhitelistInput = {
  serialNumber: 'SN-002',
  vid: '1234',
  pid: '5678',
  deviceName: 'USB Disk',
  capacityBytes: 1024,
  permission: 'read_only',
  description: '测试盘',
  addMethod: 'manual',
  deviceType: 'storage',
}

describe('useWhitelistStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
  })

  it('初始状态为空', () => {
    const store = useWhitelistStore()

    expect(store.devices).toEqual([])
    expect(store.isLoading).toBe(false)
    expect(store.errorMessage).toBe('')
    expect([...store.pendingSerialNumbers]).toEqual([])
  })

  it('加载成功后替换设备列表', async () => {
    const device = createDevice('SN-001')
    listWhitelistMock.mockResolvedValue(
      usb_control.RspListWhitelist.fromObject({ devices: [device] }),
    )
    const store = useWhitelistStore()

    await store.listWhitelist('token')

    expect(store.devices).toHaveLength(1)
    expect(store.devices[0]).toBeInstanceOf(usb_control.WhitelistDevice)
    expect(store.devices[0]?.serialNumber).toBe('SN-001')
    expect(store.errorMessage).toBe('')
  })

  it('首次加载失败时保持空列表并记录错误', async () => {
    listWhitelistMock.mockRejectedValue(new Error('连接中断'))
    const store = useWhitelistStore()

    await expect(store.listWhitelist('token')).rejects.toThrow('连接中断')

    expect(store.devices).toEqual([])
    expect(store.errorMessage).toBe('连接中断')
    expect(store.isLoading).toBe(false)
  })

  it('重新加载失败时保留旧列表', async () => {
    listWhitelistMock
      .mockResolvedValueOnce(
        usb_control.RspListWhitelist.fromObject({ devices: [createDevice('SN-001')] }),
      )
      .mockRejectedValueOnce('unknown')
    const store = useWhitelistStore()
    await store.listWhitelist('token')

    await expect(store.listWhitelist('token')).rejects.toBe('unknown')

    expect(store.devices.map((device) => device.serialNumber)).toEqual(['SN-001'])
    expect(store.errorMessage).toBe('白名单加载失败')
  })

  it('并发加载反序完成时仅最新请求可写状态且全部结束前保持加载中', async () => {
    const oldLoad = createDeferred<usb_control.RspListWhitelist>()
    const latestLoad = createDeferred<usb_control.RspListWhitelist>()
    const oldError = new Error('旧请求失败')
    listWhitelistMock
      .mockImplementationOnce(() => oldLoad.promise)
      .mockImplementationOnce(() => latestLoad.promise)
    const store = useWhitelistStore()

    const oldLoadPromise = store.listWhitelist('token')
    const latestLoadPromise = store.listWhitelist('token')
    latestLoad.resolve(usb_control.RspListWhitelist.fromObject({
      devices: [createDevice('SN-new')],
    }))
    await latestLoadPromise

    expect(store.devices.map((device) => device.serialNumber)).toEqual(['SN-new'])
    expect(store.errorMessage).toBe('')
    expect(store.isLoading).toBe(true)

    const oldRejection = expect(oldLoadPromise).rejects.toBe(oldError)
    oldLoad.reject(oldError)
    await oldRejection

    expect(store.devices.map((device) => device.serialNumber)).toEqual(['SN-new'])
    expect(store.errorMessage).toBe('')
    expect(store.isLoading).toBe(false)
  })

  it('clear 后忽略迟到加载的成功、失败及 finally', async () => {
    const successfulLoad = createDeferred<usb_control.RspListWhitelist>()
    const failedLoad = createDeferred<usb_control.RspListWhitelist>()
    const lateError = new Error('迟到失败')
    listWhitelistMock
      .mockImplementationOnce(() => successfulLoad.promise)
      .mockImplementationOnce(() => failedLoad.promise)
    const store = useWhitelistStore()

    const successfulLoadPromise = store.listWhitelist('token')
    const failedLoadPromise = store.listWhitelist('token')
    const failedLoadRejection = expect(failedLoadPromise).rejects.toBe(lateError)
    store.clear()
    successfulLoad.resolve(usb_control.RspListWhitelist.fromObject({
      devices: [createDevice('SN-late')],
    }))
    failedLoad.reject(lateError)
    await successfulLoadPromise
    await failedLoadRejection

    expect(store.devices).toEqual([])
    expect(store.errorMessage).toBe('')
    expect(store.isLoading).toBe(false)
  })

  it('旧 epoch 的 finally 不影响 clear 后新加载的计数', async () => {
    const oldLoad = createDeferred<usb_control.RspListWhitelist>()
    const newLoad = createDeferred<usb_control.RspListWhitelist>()
    listWhitelistMock
      .mockImplementationOnce(() => oldLoad.promise)
      .mockImplementationOnce(() => newLoad.promise)
    const store = useWhitelistStore()

    const oldLoadPromise = store.listWhitelist('old-token')
    store.clear()
    const newLoadPromise = store.listWhitelist('new-token')
    oldLoad.resolve(usb_control.RspListWhitelist.fromObject({
      devices: [createDevice('SN-old')],
    }))
    await oldLoadPromise

    expect(store.devices).toEqual([])
    expect(store.isLoading).toBe(true)

    newLoad.resolve(usb_control.RspListWhitelist.fromObject({
      devices: [createDevice('SN-new')],
    }))
    await newLoadPromise
    expect(store.devices.map((device) => device.serialNumber)).toEqual(['SN-new'])
    expect(store.isLoading).toBe(false)
  })

  it('添加成功后刷新列表', async () => {
    addWhitelistMock.mockResolvedValue(undefined)
    listWhitelistMock.mockResolvedValue(usb_control.RspListWhitelist.fromObject({
      devices: [createDevice('SN-002')],
    }))
    const store = useWhitelistStore()

    await store.addWhitelist('token', addInput)

    expect(addWhitelistMock).toHaveBeenCalledWith('token', addInput)
    expect(store.devices[0]?.serialNumber).toBe('SN-002')
    expect([...store.pendingSerialNumbers]).toEqual([])
  })

  it('更新失败时不改变旧列表', async () => {
    const oldResponse = usb_control.RspListWhitelist.fromObject({
      devices: [createDevice('SN-001')],
    })
    listWhitelistMock.mockResolvedValue(oldResponse)
    updateWhitelistMock.mockRejectedValue(new Error('保存失败'))
    const store = useWhitelistStore()
    await store.listWhitelist('token')

    await expect(
      store.updateWhitelist('token', 'SN-001', 'read_write', '工作盘'),
    ).rejects.toThrow('保存失败')

    expect(store.devices.map((device) => device.serialNumber)).toEqual(['SN-001'])
    expect(listWhitelistMock).toHaveBeenCalledTimes(1)
  })

  it('删除成功后刷新列表', async () => {
    removeWhitelistMock.mockResolvedValue(undefined)
    listWhitelistMock.mockResolvedValue(usb_control.RspListWhitelist.fromObject({ devices: [] }))
    const store = useWhitelistStore()

    await store.removeWhitelist('token', 'SN-001')

    expect(removeWhitelistMock).toHaveBeenCalledWith('token', 'SN-001')
    expect(store.devices).toEqual([])
  })

  it('同一序列号提交期间拒绝重复提交且不同序列号可独立提交', async () => {
    let finishFirst: () => void = () => {}
    updateWhitelistMock
      .mockImplementationOnce(
        () => new Promise<void>((resolve) => {
          finishFirst = resolve
        }),
      )
      .mockResolvedValueOnce(undefined)
    listWhitelistMock.mockResolvedValue(usb_control.RspListWhitelist.fromObject({ devices: [] }))
    const store = useWhitelistStore()

    const firstUpdate = store.updateWhitelist('token', 'SN-001', 'read_only', '')
    await store.removeWhitelist('token', 'SN-001')
    await store.updateWhitelist('token', 'SN-002', 'read_only', '')

    expect(updateWhitelistMock).toHaveBeenCalledTimes(2)
    expect(removeWhitelistMock).not.toHaveBeenCalled()
    expect(store.pendingSerialNumbers.has('SN-001')).toBe(true)
    finishFirst()
    await firstUpdate
  })

  it('clear 后旧同序列号写结束不能清除新操作的 pending 所有权', async () => {
    const oldWrite = createDeferred<void>()
    const newWrite = createDeferred<void>()
    updateWhitelistMock
      .mockImplementationOnce(() => oldWrite.promise)
      .mockImplementationOnce(() => newWrite.promise)
    listWhitelistMock.mockResolvedValue(usb_control.RspListWhitelist.fromObject({ devices: [] }))
    const store = useWhitelistStore()

    const oldWritePromise = store.updateWhitelist('old-token', 'SN-001', 'read_only', '')
    store.clear()
    const newWritePromise = store.updateWhitelist('new-token', 'SN-001', 'read_only', '')
    oldWrite.resolve()
    await oldWritePromise

    expect(listWhitelistMock).not.toHaveBeenCalled()
    expect(store.pendingSerialNumbers.has('SN-001')).toBe(true)
    await store.removeWhitelist('new-token', 'SN-001')
    expect(updateWhitelistMock).toHaveBeenCalledTimes(2)
    expect(removeWhitelistMock).not.toHaveBeenCalled()

    newWrite.resolve()
    await newWritePromise
    expect(listWhitelistMock).toHaveBeenCalledWith('new-token')
    expect(store.pendingSerialNumbers.has('SN-001')).toBe(false)
  })

  it('写入成功但刷新失败时保留旧列表并传播加载错误', async () => {
    listWhitelistMock
      .mockResolvedValueOnce(
        usb_control.RspListWhitelist.fromObject({ devices: [createDevice('SN-001')] }),
      )
      .mockRejectedValueOnce(new Error('刷新失败'))
    removeWhitelistMock.mockResolvedValue(undefined)
    const store = useWhitelistStore()
    await store.listWhitelist('token')

    await expect(store.removeWhitelist('token', 'SN-001')).rejects.toThrow('刷新失败')

    expect(store.devices.map((device) => device.serialNumber)).toEqual(['SN-001'])
    expect(store.errorMessage).toBe('刷新失败')
    expect([...store.pendingSerialNumbers]).toEqual([])
  })
})
