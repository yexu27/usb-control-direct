import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import DataTable from '../../../src/renderer/components/DataTable.vue'

const stubs = {
  ElTable: { template: '<div><slot /><slot name="empty" /></div>' },
  ElTableColumn: { template: '<div><slot :row="{}" /></div>' },
  ElPagination: {
    template: '<button class="page" @click="$emit(\'current-change\', 2)">下一页</button>',
  },
}

describe('DataTable', () => {
  it('展示筛选插槽和空状态文案', () => {
    const wrapper = mount(DataTable, {
      props: {
        columns: [{ prop: 'name', label: '名称' }],
        data: [],
        total: 0,
        page: 1,
        pageSize: 20,
        emptyText: '没有记录',
      },
      slots: { filters: '<input aria-label="筛选" />' },
      global: { stubs },
    })

    expect(wrapper.get('[aria-label="筛选"]').exists()).toBe(true)
    expect(wrapper.text()).toContain('没有记录')
  })

  it('转发分页事件', async () => {
    const wrapper = mount(DataTable, {
      props: {
        columns: [],
        data: [{ id: 1 }],
        total: 40,
        page: 1,
        pageSize: 20,
      },
      global: { stubs },
    })

    await wrapper.get('.page').trigger('click')

    expect(wrapper.emitted('page-change')).toEqual([[2]])
  })

  it('错误状态隐藏表格并展示错误文案', () => {
    const wrapper = mount(DataTable, {
      props: {
        columns: [],
        data: [],
        error: '查询失败',
        total: 0,
        page: 1,
        pageSize: 20,
      },
      global: { stubs },
    })

    expect(wrapper.get('[role="alert"]').text()).toBe('查询失败')
  })
})
