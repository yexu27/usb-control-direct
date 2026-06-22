<script setup lang="ts">
import { computed, onBeforeUnmount, reactive, ref, watch } from 'vue'
import type { FormInstance, FormRules } from 'element-plus'

type AddSource = 'device' | 'management'
type WhitelistPermission = 'readonly' | 'readwrite'

interface CandidateDevice {
  serialNumber: string
  vid: string
  pid: string
  deviceName: string
  capacityBytes: number
  deviceType: 'storage'
  addable: boolean
  unavailableReason: string
}

interface AddWhitelistFormValue {
  candidate: CandidateDevice
  description: string
  permission: WhitelistPermission
}

const props = defineProps<{
  visible: boolean
  source: AddSource
  candidates: CandidateDevice[]
  loading: boolean
  submitting: boolean
}>()
const emit = defineEmits<{
  'update:visible': [value: boolean]
  submit: [value: AddWhitelistFormValue]
}>()

const formRef = ref<FormInstance | null>(null)
const form = reactive({ serialNumber: '', description: '', permission: 'readonly' as WhitelistPermission })
const isValidating = ref(false)
const isLocked = computed(() => props.submitting || isValidating.value)
const isBusy = computed(() => props.loading || isLocked.value)
let validationEpoch = 0

const selectedCandidate = computed(() =>
  props.candidates.find((candidate) => candidate.serialNumber === form.serialNumber),
)
const rules: FormRules = {
  serialNumber: [{ required: true, message: '请选择U盘设备', trigger: 'change' }],
  permission: [{ required: true, message: '请选择权限', trigger: 'change' }],
}

function candidateLabel(candidate: CandidateDevice): string {
  return `${candidate.deviceName || '未命名设备'}（${candidate.serialNumber.trim() || '--'}）`
}

function reset(): void {
  validationEpoch += 1
  isValidating.value = false
  form.serialNumber = ''
  form.description = ''
  form.permission = 'readonly'
  formRef.value?.clearValidate()
}

function close(): void {
  if (!isLocked.value) {
    emit('update:visible', false)
  }
}

function updateVisible(visible: boolean): void {
  if (!visible && isLocked.value) {
    return
  }
  emit('update:visible', visible)
}

function beforeClose(done: () => void): void {
  if (!isLocked.value) {
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
    const candidate = selectedCandidate.value
    if (
      !valid || currentEpoch !== validationEpoch || candidate == null ||
      !candidate.addable || candidate.serialNumber.trim() === '' ||
      (form.permission !== 'readonly' && form.permission !== 'readwrite')
    ) {
      return
    }
    emit('submit', {
      candidate,
      description: form.description.trim(),
      permission: form.permission,
    })
  } finally {
    if (currentEpoch === validationEpoch) {
      isValidating.value = false
    }
  }
}

watch(() => props.visible, (visible) => {
  if (!visible) {
    reset()
  }
})
onBeforeUnmount(reset)

defineExpose({ handleSubmit })
</script>

<template>
  <el-dialog
    :model-value="props.visible"
    :title="props.source === 'device' ? '装置端添加' : '管理端添加'"
    width="520px"
    :close-on-click-modal="false"
    :close-on-press-escape="!isLocked"
    :show-close="!isLocked"
    :before-close="beforeClose"
    @update:model-value="updateVisible"
  >
    <el-form ref="formRef" :model="form" :rules="rules" label-width="90px">
      <el-form-item label="U盘设备" prop="serialNumber">
        <el-select
          v-model="form.serialNumber"
          data-testid="candidate-select"
          placeholder="请选择U盘设备"
          :loading="props.loading"
          :disabled="isBusy"
          style="width: 100%"
        >
          <el-option
            v-for="candidate in props.candidates"
            :key="`${candidate.serialNumber}-${candidate.vid}-${candidate.pid}`"
            :value="candidate.serialNumber"
            :label="candidateLabel(candidate)"
            :disabled="!candidate.addable || candidate.serialNumber.trim() === ''"
          >
            <span>{{ candidateLabel(candidate) }}</span>
            <span v-if="!candidate.addable || candidate.serialNumber.trim() === ''" class="reason">
              {{ candidate.unavailableReason || '设备标识异常' }}
            </span>
          </el-option>
        </el-select>
      </el-form-item>
      <el-form-item label="描述" prop="description">
        <el-input
          v-model="form.description" data-testid="whitelist-description-input"
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
      <el-button data-testid="whitelist-add-cancel" :disabled="isLocked" @click="close">取消</el-button>
      <el-button
        type="primary" data-testid="whitelist-add-submit"
        :disabled="isBusy" :loading="isBusy" @click="handleSubmit"
      >
        确定
      </el-button>
    </template>
  </el-dialog>
</template>

<style scoped lang="scss">
.reason {
  margin-left: $spacing-3;
  color: $color-danger;
}
</style>
