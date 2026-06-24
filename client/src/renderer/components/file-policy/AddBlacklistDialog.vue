<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import type { FormInstance, FormRules } from 'element-plus'

interface BlacklistFormValue {
  extension: string
  description: string
}

const props = defineProps<{
  visible: boolean
  submitting: boolean
  errorMessage?: string
}>()
const emit = defineEmits<{
  'update:visible': [value: boolean]
  submit: [value: BlacklistFormValue]
}>()

const formRef = ref<FormInstance | null>(null)
const blacklistForm = reactive<BlacklistFormValue>({ extension: '', description: '' })
const isValidating = ref(false)
const isBusy = computed(() => props.submitting || isValidating.value)
const extensionPattern = /^\.[A-Za-z0-9][A-Za-z0-9_-]*$/
let validationEpoch = 0

function validateExtension(
  _rule: unknown,
  value: string,
  callback: (error?: Error) => void,
): void {
  const extension = value.trim()
  callback(
    extensionPattern.test(extension) ? undefined : new Error('请输入正确的文件后缀'),
  )
}

const rules: FormRules = {
  extension: [
    { required: true, message: '请输入文件后缀', trigger: 'blur' },
    { validator: validateExtension, trigger: 'blur' },
  ],
}

function reset(): void {
  validationEpoch += 1
  isValidating.value = false
  blacklistForm.extension = ''
  blacklistForm.description = ''
  formRef.value?.clearValidate()
}

function close(): void {
  if (isBusy.value) {
    return
  }
  emit('update:visible', false)
}

function handleVisibilityChange(visible: boolean): void {
  if (isBusy.value && !visible) {
    return
  }
  emit('update:visible', visible)
}

function handleBeforeClose(done: () => void): void {
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
    if (!valid || currentEpoch !== validationEpoch) {
      return
    }
    emit('submit', {
      extension: blacklistForm.extension.trim().toLowerCase(),
      description: blacklistForm.description.trim(),
    })
  } finally {
    if (currentEpoch === validationEpoch) {
      isValidating.value = false
    }
  }
}

watch(
  () => props.visible,
  (visible) => {
    if (!visible) {
      reset()
    }
  },
)
</script>

<template>
  <el-dialog
    :model-value="props.visible"
    title="添加黑名单"
    width="440px"
    :close-on-click-modal="false"
    :close-on-press-escape="!isBusy"
    :show-close="!isBusy"
    :before-close="handleBeforeClose"
    @update:model-value="handleVisibilityChange"
  >
    <el-alert
      v-if="props.errorMessage"
      class="dialog-error"
      type="error"
      :title="props.errorMessage"
      :closable="false"
      show-icon
    />
    <el-form ref="formRef" :model="blacklistForm" :rules="rules" label-width="80px">
      <el-form-item label="后缀名" prop="extension">
        <el-input
          v-model="blacklistForm.extension"
          data-testid="blacklist-extension-input"
          placeholder="例如 .zip"
          maxlength="32"
          :disabled="isBusy"
        />
      </el-form-item>
      <el-form-item label="说明" prop="description">
        <el-input
          v-model="blacklistForm.description"
          data-testid="blacklist-description-input"
          placeholder="请输入说明"
          maxlength="100"
          :disabled="isBusy"
        />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button data-testid="blacklist-cancel" :disabled="isBusy" @click="close">
        取消
      </el-button>
      <el-button
        type="primary"
        data-testid="blacklist-submit"
        :disabled="isBusy"
        :loading="isBusy"
        @click="handleSubmit"
      >
        确定
      </el-button>
    </template>
  </el-dialog>
</template>

<style scoped lang="scss">
@use '@/styles/tokens' as *;

.dialog-error {
  margin-bottom: $spacing-4;
}
</style>
