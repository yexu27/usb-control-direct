<script setup lang="ts">
import { Loading } from '@element-plus/icons-vue'

interface Props {
  visible: boolean
  title: string
  message?: string
}

const props = withDefaults(defineProps<Props>(), {
  message: '正在处理，请稍候...',
})
</script>

<template>
  <el-dialog
    :model-value="props.visible"
    :title="props.title"
    width="360px"
    :close-on-click-modal="false"
    :close-on-press-escape="false"
    :show-close="false"
  >
    <div class="progress-content">
      <el-icon class="is-loading" :size="24">
        <Loading />
      </el-icon>
      <span>{{ props.message }}</span>
    </div>
  </el-dialog>
</template>

<style scoped lang="scss">
.progress-content {
  display: flex;
  align-items: center;
  gap: $spacing-5;
  padding: $spacing-5 0;
}
</style>
