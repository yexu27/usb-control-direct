<script setup lang="ts">
interface Props {
  visible: boolean
  title: string
  message: string
  confirmText?: string
  cancelText?: string
  type?: 'warning' | 'danger' | 'info'
}

const props = withDefaults(defineProps<Props>(), {
  confirmText: '确定',
  cancelText: '取消',
  type: 'warning',
})

const emit = defineEmits<{
  confirm: []
  cancel: []
  'update:visible': [value: boolean]
}>()

function close(): void {
  emit('update:visible', false)
}

function handleConfirm(): void {
  emit('confirm')
  close()
}

function handleCancel(): void {
  emit('cancel')
  close()
}
</script>

<template>
  <el-dialog
    :model-value="props.visible"
    :title="props.title"
    width="420px"
    :close-on-click-modal="false"
    @close="close"
  >
    <span>{{ props.message }}</span>
    <template #footer>
      <el-button @click="handleCancel">{{ props.cancelText }}</el-button>
      <el-button :type="props.type === 'danger' ? 'danger' : 'primary'" @click="handleConfirm">
        {{ props.confirmText }}
      </el-button>
    </template>
  </el-dialog>
</template>
