import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import DataTable from '../../../src/renderer/components/DataTable.vue'

const stubs = {
  ElTable: { template: '<div class="el-table"><slot /><slot name="empty" /></div>' },
  ElTableColumn: { template: '<div><slot :row="{}" /></div>' },
  ElPagination: { template: '<nav class="el-pagination" />' },
}
const directives = {
  loading: {},
}

describe('DataTable style contract', () => {
  it('uses global filter and compact pagination classes', () => {
    const wrapper = mount(DataTable, {
      props: {
        columns: [{ prop: 'name', label: '名称' }],
        data: [{ name: '记录' }],
        total: 40,
        page: 1,
        pageSize: 20,
      },
      slots: { filters: '<button type="button">搜索</button>' },
      global: { stubs, directives },
    })

    const filters = wrapper.get('[data-testid="table-filters"]')
    expect(filters.classes()).toContain('app-filter-bar')

    const pagination = wrapper.get('[data-testid="table-pagination"]')
    expect(pagination.classes()).toContain('app-table-pagination')
  })

  it('keeps table errors on a global alert surface', () => {
    const wrapper = mount(DataTable, {
      props: {
        columns: [],
        data: [],
        error: '查询失败',
        total: 0,
        page: 1,
        pageSize: 20,
      },
      global: { stubs, directives },
    })

    expect(wrapper.get('[role="alert"]').classes()).toContain('app-table-error')
  })
})
