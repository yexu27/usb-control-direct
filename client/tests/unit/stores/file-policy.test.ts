import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useFilePolicyStore } from '../../../src/renderer/stores/file-policy'
import {
  addBlacklistExtension,
  getFilePolicy,
  removeBlacklistExtension,
  updateSwitch,
} from '../../../src/renderer/services/file-policy-service'
import { usb_control } from '../../../src/shared/proto/usb_control'

vi.mock('../../../src/renderer/services/file-policy-service', () => ({
  addBlacklistExtension: vi.fn(),
  getFilePolicy: vi.fn(),
  removeBlacklistExtension: vi.fn(),
  updateSwitch: vi.fn(),
}))

const addBlacklistExtensionMock = vi.mocked(addBlacklistExtension)
const getFilePolicyMock = vi.mocked(getFilePolicy)
const removeBlacklistExtensionMock = vi.mocked(removeBlacklistExtension)
const updateSwitchMock = vi.mocked(updateSwitch)

function createPolicy(execControlEnabled: boolean): usb_control.RspFilePolicy {
  return usb_control.RspFilePolicy.fromObject({
    execControlEnabled,
    autoReadControlEnabled: false,
    fileTypeBlacklistEnabled: false,
    blacklist: [],
  })
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

describe('useFilePolicyStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
  })

  it('初始状态为空', () => {
    const store = useFilePolicyStore()

    expect(store.policy).toBeNull()
    expect(store.isLoading).toBe(false)
    expect(store.errorMessage).toBe('')
    expect([...store.pendingKeys]).toEqual([])
  })

  it('加载成功后替换策略', async () => {
    const policy = createPolicy(true)
    getFilePolicyMock.mockResolvedValue(policy)
    const store = useFilePolicyStore()

    await store.load('token')

    expect(store.policy).toEqual(policy)
    expect(store.errorMessage).toBe('')
    expect(store.isLoading).toBe(false)
  })

  it('首次加载失败时保持空状态并记录错误', async () => {
    getFilePolicyMock.mockRejectedValue(new Error('连接中断'))
    const store = useFilePolicyStore()

    await expect(store.load('token')).rejects.toThrow('连接中断')

    expect(store.policy).toBeNull()
    expect(store.errorMessage).toBe('连接中断')
    expect(store.isLoading).toBe(false)
  })

  it('重新加载失败时保留旧策略', async () => {
    const oldPolicy = createPolicy(false)
    getFilePolicyMock.mockResolvedValueOnce(oldPolicy).mockRejectedValueOnce('unknown')
    const store = useFilePolicyStore()
    await store.load('token')

    await expect(store.load('token')).rejects.toBe('unknown')

    expect(store.policy).toEqual(oldPolicy)
    expect(store.errorMessage).toBe('文件访问策略加载失败')
  })

  it('并发加载反序完成时仅最新请求可写状态且全部结束前保持加载中', async () => {
    const oldLoad = createDeferred<usb_control.RspFilePolicy>()
    const latestLoad = createDeferred<usb_control.RspFilePolicy>()
    const oldError = new Error('旧请求失败')
    getFilePolicyMock
      .mockImplementationOnce(() => oldLoad.promise)
      .mockImplementationOnce(() => latestLoad.promise)
    const store = useFilePolicyStore()

    const oldLoadPromise = store.load('token')
    const latestLoadPromise = store.load('token')
    latestLoad.resolve(createPolicy(true))
    await latestLoadPromise

    expect(store.policy?.execControlEnabled).toBe(true)
    expect(store.errorMessage).toBe('')
    expect(store.isLoading).toBe(true)

    const oldRejection = expect(oldLoadPromise).rejects.toBe(oldError)
    oldLoad.reject(oldError)
    await oldRejection

    expect(store.policy?.execControlEnabled).toBe(true)
    expect(store.errorMessage).toBe('')
    expect(store.isLoading).toBe(false)
  })

  it('旧加载成功迟到时不能覆盖最新结果', async () => {
    const oldLoad = createDeferred<usb_control.RspFilePolicy>()
    const latestLoad = createDeferred<usb_control.RspFilePolicy>()
    getFilePolicyMock
      .mockImplementationOnce(() => oldLoad.promise)
      .mockImplementationOnce(() => latestLoad.promise)
    const store = useFilePolicyStore()

    const oldLoadPromise = store.load('token')
    const latestLoadPromise = store.load('token')
    latestLoad.resolve(createPolicy(true))
    await latestLoadPromise
    oldLoad.resolve(createPolicy(false))
    await oldLoadPromise

    expect(store.policy?.execControlEnabled).toBe(true)
  })

  it('clear 后忽略迟到加载的成功、失败及 finally', async () => {
    const successfulLoad = createDeferred<usb_control.RspFilePolicy>()
    const failedLoad = createDeferred<usb_control.RspFilePolicy>()
    const lateError = new Error('迟到失败')
    getFilePolicyMock
      .mockImplementationOnce(() => successfulLoad.promise)
      .mockImplementationOnce(() => failedLoad.promise)
    const store = useFilePolicyStore()

    const successfulLoadPromise = store.load('token')
    const failedLoadPromise = store.load('token')
    const failedLoadRejection = expect(failedLoadPromise).rejects.toBe(lateError)
    store.clear()
    successfulLoad.resolve(createPolicy(true))
    failedLoad.reject(lateError)
    await successfulLoadPromise
    await failedLoadRejection

    expect(store.policy).toBeNull()
    expect(store.errorMessage).toBe('')
    expect(store.isLoading).toBe(false)
  })

  it('旧 epoch 的 finally 不影响 clear 后新加载的计数', async () => {
    const oldLoad = createDeferred<usb_control.RspFilePolicy>()
    const newLoad = createDeferred<usb_control.RspFilePolicy>()
    getFilePolicyMock
      .mockImplementationOnce(() => oldLoad.promise)
      .mockImplementationOnce(() => newLoad.promise)
    const store = useFilePolicyStore()

    const oldLoadPromise = store.load('old-token')
    store.clear()
    const newLoadPromise = store.load('new-token')
    oldLoad.resolve(createPolicy(false))
    await oldLoadPromise

    expect(store.policy).toBeNull()
    expect(store.isLoading).toBe(true)

    newLoad.resolve(createPolicy(true))
    await newLoadPromise
    expect(store.policy?.execControlEnabled).toBe(true)
    expect(store.isLoading).toBe(false)
  })

  it('更新开关成功后重新加载策略', async () => {
    const oldPolicy = createPolicy(false)
    const newPolicy = createPolicy(true)
    getFilePolicyMock.mockResolvedValueOnce(oldPolicy).mockResolvedValueOnce(newPolicy)
    updateSwitchMock.mockResolvedValue(undefined)
    const store = useFilePolicyStore()
    await store.load('token')

    await store.setSwitch('token', 'exec_control', true)

    expect(updateSwitchMock).toHaveBeenCalledWith('token', 'exec_control', true)
    expect(store.policy).toEqual(newPolicy)
    expect([...store.pendingKeys]).toEqual([])
  })

  it('更新开关失败时不改变旧策略', async () => {
    const oldPolicy = createPolicy(false)
    getFilePolicyMock.mockResolvedValue(oldPolicy)
    updateSwitchMock.mockRejectedValue(new Error('保存失败'))
    const store = useFilePolicyStore()
    await store.load('token')

    await expect(store.setSwitch('token', 'exec_control', true)).rejects.toThrow('保存失败')

    expect(store.policy).toEqual(oldPolicy)
    expect(getFilePolicyMock).toHaveBeenCalledTimes(1)
    expect([...store.pendingKeys]).toEqual([])
  })

  it('同一策略键提交期间拒绝重复提交', async () => {
    let finishUpdate: () => void = () => {}
    updateSwitchMock.mockImplementation(
      () => new Promise<void>((resolve) => {
        finishUpdate = resolve
      }),
    )
    getFilePolicyMock.mockResolvedValue(createPolicy(true))
    const store = useFilePolicyStore()

    const firstUpdate = store.setSwitch('token', 'exec_control', true)
    await store.setSwitch('token', 'exec_control', false)

    expect(updateSwitchMock).toHaveBeenCalledTimes(1)
    expect(store.pendingKeys.has('exec_control')).toBe(true)
    finishUpdate()
    await firstUpdate
  })

  it('clear 后旧同键写结束不能清除新操作的 pending 所有权', async () => {
    const oldWrite = createDeferred<void>()
    const newWrite = createDeferred<void>()
    updateSwitchMock
      .mockImplementationOnce(() => oldWrite.promise)
      .mockImplementationOnce(() => newWrite.promise)
    getFilePolicyMock.mockResolvedValue(createPolicy(true))
    const store = useFilePolicyStore()

    const oldWritePromise = store.setSwitch('old-token', 'exec_control', true)
    store.clear()
    const newWritePromise = store.setSwitch('new-token', 'exec_control', true)
    oldWrite.resolve()
    await oldWritePromise

    expect(getFilePolicyMock).not.toHaveBeenCalled()
    expect(store.pendingKeys.has('exec_control')).toBe(true)
    await store.setSwitch('new-token', 'exec_control', false)
    expect(updateSwitchMock).toHaveBeenCalledTimes(2)

    newWrite.resolve()
    await newWritePromise
    expect(getFilePolicyMock).toHaveBeenCalledWith('new-token')
    expect(store.pendingKeys.has('exec_control')).toBe(false)
  })

  it('添加扩展名成功后刷新策略', async () => {
    const refreshedPolicy = createPolicy(false)
    getFilePolicyMock.mockResolvedValue(refreshedPolicy)
    addBlacklistExtensionMock.mockResolvedValue(undefined)
    const store = useFilePolicyStore()

    await store.addExtension('token', '.cmd', '命令脚本')

    expect(addBlacklistExtensionMock).toHaveBeenCalledWith('token', '.cmd', '命令脚本')
    expect(store.policy).toEqual(refreshedPolicy)
  })

  it('删除扩展名成功后刷新策略', async () => {
    const refreshedPolicy = createPolicy(false)
    getFilePolicyMock.mockResolvedValue(refreshedPolicy)
    removeBlacklistExtensionMock.mockResolvedValue(undefined)
    const store = useFilePolicyStore()

    await store.removeExtension('token', '.cmd')

    expect(removeBlacklistExtensionMock).toHaveBeenCalledWith('token', '.cmd')
    expect(store.policy).toEqual(refreshedPolicy)
  })

  it('扩展名提交按扩展名去重且写失败保留旧策略', async () => {
    const oldPolicy = createPolicy(false)
    getFilePolicyMock.mockResolvedValue(oldPolicy)
    let failAdd: (error: Error) => void = () => {}
    addBlacklistExtensionMock.mockImplementation(
      () => new Promise<void>((_resolve, reject) => {
        failAdd = reject
      }),
    )
    const store = useFilePolicyStore()
    await store.load('token')

    const firstAdd = store.addExtension('token', '.cmd', '命令脚本')
    await store.removeExtension('token', '.cmd')

    expect(addBlacklistExtensionMock).toHaveBeenCalledTimes(1)
    expect(removeBlacklistExtensionMock).not.toHaveBeenCalled()
    failAdd(new Error('保存失败'))
    await expect(firstAdd).rejects.toThrow('保存失败')
    expect(store.policy).toEqual(oldPolicy)
  })

  it('写入成功但刷新失败时保留旧策略并传播加载错误', async () => {
    const oldPolicy = createPolicy(false)
    getFilePolicyMock.mockResolvedValueOnce(oldPolicy).mockRejectedValueOnce(new Error('刷新失败'))
    updateSwitchMock.mockResolvedValue(undefined)
    const store = useFilePolicyStore()
    await store.load('token')

    await expect(store.setSwitch('token', 'exec_control', true)).rejects.toThrow('刷新失败')

    expect(store.policy).toEqual(oldPolicy)
    expect(store.errorMessage).toBe('刷新失败')
    expect([...store.pendingKeys]).toEqual([])
  })
})
