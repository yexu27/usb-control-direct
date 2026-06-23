<script setup lang="ts">
import type { DataTableColumn } from './data-table'

interface Props {
  columns: DataTableColumn[]
  data: object[]
  loading?: boolean
  error?: string
  total: number
  page: number
  pageSize: number
  emptyText?: string
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
  error: '',
  emptyText: '暂无数据',
})

const emit = defineEmits<{
  'page-change': [page: number]
  'page-size-change': [pageSize: number]
}>()
</script>

<template>
  <div class="data-table-wrapper">
    <div v-if="$slots.filters" class="app-filter-bar" data-testid="table-filters">
      <slot name="filters" />
    </div>

    <div v-if="props.error" class="app-table-error" role="alert">
      {{ props.error }}
    </div>

    <el-table
      v-else
      v-loading="props.loading"
      :data="props.data"
      border
      stripe
      class="data-table"
    >
      <el-table-column
        v-for="column in props.columns"
        :key="column.prop"
        :prop="column.prop"
        :label="column.label"
        :width="column.width"
        :min-width="column.minWidth"
        :fixed="column.fixed"
        :sortable="column.sortable"
      >
        <template v-if="column.slot" #default="scope">
          <slot :name="column.slot" v-bind="scope" />
        </template>
      </el-table-column>

      <template #empty>
        <span>{{ props.emptyText }}</span>
      </template>
    </el-table>

    <el-pagination
      v-if="!props.error && props.total > 0"
      class="app-table-pagination"
      data-testid="table-pagination"
      :current-page="props.page"
      :page-size="props.pageSize"
      :total="props.total"
      :page-sizes="[20, 50, 100]"
      layout="total, sizes, prev, pager, next, jumper"
      @current-change="emit('page-change', $event)"
      @size-change="emit('page-size-change', $event)"
    />
  </div>
</template>
