import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { shallowMount } from '@vue/test-utils'
import ConnectionAlert from '../../../src/renderer/components/ConnectionAlert.vue'
import { useConnectionStore } from '../../../src/renderer/stores/connection'

describe('ConnectionAlert', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('初始未连接时不显示断线提示', () => {
    const wrapper = shallowMount(ConnectionAlert, {
      global: { stubs: { ElAlert: true } },
    })

    expect(wrapper.find('el-alert-stub').exists()).toBe(false)
  })

  it('曾经连接成功后断线时显示提示', () => {
    const connection = useConnectionStore()
    connection.wasConnected = true
    connection.updateStatus('DISCONNECTED')
    const wrapper = shallowMount(ConnectionAlert, {
      global: { stubs: { ElAlert: true } },
    })

    expect(wrapper.get('el-alert-stub').attributes('title')).toBe(
      'USB 管控装置已断开连接，请检查网络或设备连接。',
    )
  })
})
