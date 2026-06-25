<script setup lang="ts">
import { reactive, ref, watch } from 'vue'
import { type FormInstance, type FormRules } from 'element-plus'
import { useSessionStore } from '@/stores/session'
import { changePassword } from '@/services/user-service'
import { ServiceError } from '@/services/send-command'
import { validatePasswordComplexity } from '@/utils/password-validator'
import { errorMessage, showSuccessToast } from '@/utils/operation-feedback'

const props = defineProps<{ visible: boolean }>()
const emit = defineEmits<{ 'update:visible': [value: boolean] }>()

const session = useSessionStore()
const formRef = ref<FormInstance | null>(null)
const isSubmitting = ref(false)
const submitErrorMessage = ref('')
const passwordForm = reactive({
  oldPassword: '',
  newPassword: '',
  confirmPassword: '',
})

function validateNewPassword(
  _rule: unknown,
  value: string,
  callback: (error?: Error) => void,
): void {
  const result = validatePasswordComplexity(value)
  callback(result.valid ? undefined : new Error(result.message))
}

function validateConfirmPassword(
  _rule: unknown,
  value: string,
  callback: (error?: Error) => void,
): void {
  callback(value === passwordForm.newPassword ? undefined : new Error('两次输入的密码不一致'))
}

const rules: FormRules = {
  oldPassword: [{ required: true, message: '请输入旧密码', trigger: 'blur' }],
  newPassword: [
    { required: true, message: '请输入新密码', trigger: 'blur' },
    { validator: validateNewPassword, trigger: 'blur' },
  ],
  confirmPassword: [
    { required: true, message: '请再次输入新密码', trigger: 'blur' },
    { validator: validateConfirmPassword, trigger: 'blur' },
  ],
}

function resetForm(): void {
  passwordForm.oldPassword = ''
  passwordForm.newPassword = ''
  passwordForm.confirmPassword = ''
  submitErrorMessage.value = ''
  formRef.value?.clearValidate()
}

function close(): void {
  if (isSubmitting.value) {
    return
  }
  emit('update:visible', false)
}

function handleVisibilityChange(visible: boolean): void {
  if (visible) {
    emit('update:visible', true)
    return
  }
  close()
}

async function handleSubmit(): Promise<void> {
  if (formRef.value == null || isSubmitting.value) {
    return
  }

  isSubmitting.value = true
  submitErrorMessage.value = ''
  const isValid = await formRef.value.validate().catch(() => false)
  if (!isValid) {
    submitErrorMessage.value = '请检查密码填写内容'
    isSubmitting.value = false
    return
  }

  try {
    await changePassword(
      session.token,
      passwordForm.oldPassword,
      passwordForm.newPassword,
      passwordForm.confirmPassword,
    )
    showSuccessToast('密码修改成功')
    resetForm()
    emit('update:visible', false)
  } catch (error: unknown) {
    if (error instanceof ServiceError && error.kind === 'unauthenticated') {
      return
    }
    submitErrorMessage.value = errorMessage(error, '密码修改失败')
  } finally {
    isSubmitting.value = false
  }
}

watch(
  () => props.visible,
  (isVisible) => {
    if (!isVisible) {
      resetForm()
    }
  },
)
</script>

<template>
  <el-dialog
    :model-value="props.visible"
    title="修改密码"
    width="420px"
    :close-on-click-modal="false"
    :close-on-press-escape="!isSubmitting"
    :show-close="!isSubmitting"
    @update:model-value="handleVisibilityChange"
  >
    <el-alert
      v-if="submitErrorMessage"
      class="dialog-error"
      type="error"
      :title="submitErrorMessage"
      :closable="false"
      show-icon
    />

    <el-form ref="formRef" :model="passwordForm" :rules="rules" label-width="80px">
      <el-form-item label="旧密码" prop="oldPassword">
        <el-input
          v-model="passwordForm.oldPassword"
          type="password"
          show-password
          placeholder="请输入旧密码"
          :disabled="isSubmitting"
        />
      </el-form-item>
      <el-form-item label="新密码" prop="newPassword">
        <el-input
          v-model="passwordForm.newPassword"
          type="password"
          show-password
          placeholder="请输入新密码"
          :disabled="isSubmitting"
        />
      </el-form-item>
      <el-form-item label="确认密码" prop="confirmPassword">
        <el-input
          v-model="passwordForm.confirmPassword"
          type="password"
          show-password
          placeholder="请再次输入新密码"
          :disabled="isSubmitting"
        />
      </el-form-item>
    </el-form>

    <template #footer>
      <el-button :disabled="isSubmitting" @click="close">取消</el-button>
      <el-button type="primary" :loading="isSubmitting" @click="handleSubmit">确定</el-button>
    </template>
  </el-dialog>
</template>

<style scoped lang="scss">
@use '@/styles/tokens' as *;

.dialog-error {
  margin-bottom: $spacing-4;
}
</style>
