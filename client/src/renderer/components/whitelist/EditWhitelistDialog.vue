<script setup lang="ts">
import { computed, onBeforeUnmount, reactive, ref, watch } from 'vue'
import type { FormInstance, FormRules } from 'element-plus'

type WhitelistPermission = 'readonly' | 'readwrite'

interface EditWhitelistFormValue {
  description: string
  permission: WhitelistPermission
}

const props = defineProps<{
  visible: boolean
  serialNumber: string
  currentDescription: string
  currentPermission: WhitelistPermission
  submitting: boolean
  errorMessage?: string
}>()
const emit = defineEmits<{
  'update:visible': [value: boolean]
  submit: [value: EditWhitelistFormValue]
}>()
const formRef = ref<FormInstance | null>(null)
const form = reactive({ description: '', permission: 'readonly' as WhitelistPermission })
const isValidating = ref(false)
const isBusy = computed(() => props.submitting || isValidating.value)
let validationEpoch = 0
const rules: FormRules = {
  permission: [{ required: true, message: '请选择权限', trigger: 'change' }],
}

function reset(): void {
  validationEpoch += 1
  isValidating.value = false
  form.description = props.currentDescription
  form.permission = props.currentPermission
  formRef.value?.clearValidate()
}
function close(): void {
  if (!isBusy.value) {
    emit('update:visible', false)
  }
}
function updateVisible(visible: boolean): void {
  if (!visible && isBusy.value) {
    return
  }
  emit('update:visible', visible)
}
function beforeClose(done: () => void): void {
  if (!isBusy.value) {
    done()
  }
}

async function handleSubmit(): Promise<void> {
  if (formRef.value == null || isBusy.value) {
    return
  }
  const currentEpoch = validationEpoch
  isValidating.value = true
  try {
    const valid = await formRef.value.validate().catch(() => false)
    if (!valid || currentEpoch !== validationEpoch ||
      (form.permission !== 'readonly' && form.permission !== 'readwrite')) {
      return
    }
    emit('submit', {
      description: form.description.trim(),
      permission: form.permission,
    })
  } finally {
    if (currentEpoch === validationEpoch) {
      isValidating.value = false
    }
  }
}

watch(() => props.visible, reset, { immediate: true })
onBeforeUnmount(reset)
defineExpose({ handleSubmit })
</script>

<template>
  <el-dialog
    :model-value="props.visible" title="修改白名单设备" width="480px"
    :close-on-click-modal="false" :close-on-press-escape="!isBusy" :show-close="!isBusy"
    :before-close="beforeClose" @update:model-value="updateVisible"
  >
    <el-alert
      v-if="props.errorMessage"
      class="dialog-error"
      type="error"
      :title="props.errorMessage"
      :closable="false"
      show-icon
    />
    <el-form ref="formRef" :model="form" :rules="rules" label-width="80px">
      <el-form-item label="序列号">
        <el-input :model-value="props.serialNumber" disabled />
      </el-form-item>
      <el-form-item label="描述" prop="description">
        <el-input
          v-model="form.description" data-testid="whitelist-edit-description-input"
          maxlength="100" :disabled="isBusy"
        />
      </el-form-item>
      <el-form-item label="权限" prop="permission">
        <el-radio-group v-model="form.permission" :disabled="isBusy">
          <el-radio value="readonly">只读</el-radio>
          <el-radio value="readwrite">读写</el-radio>
        </el-radio-group>
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button data-testid="whitelist-edit-cancel" :disabled="isBusy" @click="close">取消</el-button>
      <el-button
        type="primary" data-testid="whitelist-edit-submit"
        :disabled="isBusy" :loading="isBusy" @click="handleSubmit"
      >确定</el-button>
    </template>
  </el-dialog>
</template>

<style scoped lang="scss">
.dialog-error {
  margin-bottom: $spacing-4;
}
</style>
