<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import { Close, FullScreen, Minus } from '@element-plus/icons-vue'
import type { FormInstance, FormRules } from 'element-plus'
import { useConnectionStore } from '@/stores/connection'
import { useSessionStore } from '@/stores/session'
import { ROLE_DEFAULT_ROUTES } from '@/router/routes'
import { ServiceError } from '@/services/send-command'

interface LoginForm {
  ip: string
  username: string
  password: string
}

const session = useSessionStore()
const connection = useConnectionStore()
const router = useRouter()

const formRef = ref<FormInstance | null>(null)
const ipInputRef = ref<{ focus: () => void } | null>(null)
const isSubmitting = ref(false)
const errorMessage = ref('')
const form = reactive<LoginForm>({
  ip: '',
  username: '',
  password: '',
})

function isValidIpv4(value: string): boolean {
  const octets = value.split('.')
  return octets.length === 4 && octets.every((octet) => {
    if (!/^(0|[1-9]\d{0,2})$/.test(octet)) {
      return false
    }
    return Number(octet) <= 255
  })
}

const rules: FormRules = {
  ip: [
    { required: true, message: '请输入装置 IP', trigger: 'blur' },
    {
      validator: (_rule, value: string, callback) => {
        callback(isValidIpv4(value.trim()) ? undefined : new Error('装置 IP 格式错误'))
      },
      trigger: 'blur',
    },
  ],
  username: [{ required: true, message: '请输入用户名', trigger: 'blur' }],
  password: [{ required: true, message: '请输入密码', trigger: 'blur' }],
}

const connectionStatusText = computed(() => {
  if (!isSubmitting.value) {
    return ''
  }
  switch (connection.status) {
    case 'CONNECTING':
      return '正在连接装置...'
    case 'AUTHENTICATING':
      return '正在进行身份验证...'
    case 'CHECK_LICENSE':
      return '正在检查授权状态...'
    case 'LOADING_CONFIG':
      return '正在加载配置...'
    default:
      return ''
  }
})

function handleMinimize(): void {
  void window.desktopApi.window.minimize()
}

function handleMaximize(): void {
  void window.desktopApi.window.maximize()
}

function handleClose(): void {
  void window.desktopApi.window.close()
}

function mapLoginError(error: unknown): string {
  if (error instanceof ServiceError) {
    return error.message
  }
  if (!(error instanceof Error)) {
    return '装置连接失败，请检查网络连接'
  }
  if (error.message === '版本不兼容，请升级管理端') {
    return error.message
  }
  if (error.message.includes('已有管理端连接')) {
    return '装置已有管理端连接，请稍后重试'
  }
  if (error.message.startsWith('Request timeout:')) {
    return '装置响应超时，请检查装置状态或稍后重试'
  }
  if (error.message === '装置 IP 格式错误') {
    return error.message
  }
  if (error.message.includes('certificate') || error.message.includes('指纹')) {
    return error.message
  }
  return error.message || '装置连接失败，请检查网络连接'
}

async function handleSubmit(): Promise<void> {
  if (formRef.value == null || isSubmitting.value) {
    return
  }

  isSubmitting.value = true
  const valid = await formRef.value.validate().catch(() => false)
  if (!valid) {
    isSubmitting.value = false
    return
  }

  errorMessage.value = ''

  try {
    await session.login(form.ip.trim(), form.username.trim(), form.password)
    if (session.authStatus === 'unauthorized' || session.authStatus === 'expired') {
      await router.push('/license')
      return
    }
    if (session.role !== '') {
      await router.push(ROLE_DEFAULT_ROUTES[session.role])
    }
  } catch (error: unknown) {
    errorMessage.value = mapLoginError(error)
  } finally {
    isSubmitting.value = false
  }
}

onMounted(() => {
  ipInputRef.value?.focus()
})
</script>

<template>
  <div class="login-page auth-page">
    <div class="win-controls">
      <button class="win-btn" title="最小化" @click="handleMinimize">
        <el-icon><Minus /></el-icon>
      </button>
      <button class="win-btn" title="最大化" @click="handleMaximize">
        <el-icon><FullScreen /></el-icon>
      </button>
      <button class="win-btn win-btn-close" title="关闭" @click="handleClose">
        <el-icon><Close /></el-icon>
      </button>
    </div>

    <section class="login-card auth-card app-card" aria-labelledby="login-title">
      <div class="login-header">
        <div class="login-logo" aria-hidden="true">
          <svg viewBox="0 0 16 16" role="img">
            <path d="M8 1 3 4v4c0 3.3 2.2 6.2 5 7 2.8-.8 5-3.7 5-7V4L8 1Z" />
            <path class="login-logo-inner" d="M8 3 5 5v3.5c0 2.2 1.3 4 3 4.6 1.7-.6 3-2.4 3-4.6V5L8 3Z" />
          </svg>
        </div>
        <h1 id="login-title" class="auth-title">USB安全管理系统</h1>
        <p>北京安帝科技有限公司</p>
      </div>

      <el-form
        ref="formRef"
        :model="form"
        :rules="rules"
        label-position="top"
        @submit.prevent="handleSubmit"
      >
        <el-form-item label="用户名" prop="username">
          <el-input
            v-model="form.username"
            data-testid="login-username"
            placeholder="请输入用户名"
            autocomplete="username"
            :disabled="isSubmitting"
          />
        </el-form-item>

        <el-form-item label="密码" prop="password">
          <el-input
            v-model="form.password"
            data-testid="login-password"
            type="password"
            placeholder="请输入密码"
            autocomplete="current-password"
            show-password
            :disabled="isSubmitting"
          />
        </el-form-item>

        <el-form-item label="装置 IP" prop="ip">
          <el-input
            ref="ipInputRef"
            v-model="form.ip"
            data-testid="login-ip"
            placeholder="请输入装置 IP 地址"
            inputmode="decimal"
            maxlength="15"
            :disabled="isSubmitting"
          />
        </el-form-item>

        <el-alert
          v-if="errorMessage"
          class="login-error"
          :title="errorMessage"
          type="error"
          show-icon
          :closable="false"
        />

        <el-button
          class="login-button"
          data-testid="login-submit"
          type="primary"
          native-type="submit"
          :loading="isSubmitting"
          :disabled="isSubmitting"
          @click="handleSubmit"
        >
          {{ isSubmitting ? '登录中...' : '登 录' }}
        </el-button>
      </el-form>

      <div v-if="connectionStatusText" class="connection-status" aria-live="polite">
        <span class="connection-dot" aria-hidden="true" />
        <span>{{ connectionStatusText }}</span>
      </div>
    </section>
  </div>
</template>

<style scoped lang="scss">
@use '@/styles/tokens' as *;

.login-page {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100vh;
  overflow: hidden;
  color: var(--andi-text);
  font-family: var(--andi-font-family);
  background: linear-gradient(135deg, var(--andi-page-bg), #f5f7fa);
  user-select: none;
  -webkit-app-region: drag;
}

.win-controls {
  position: absolute;
  z-index: 10;
  top: 0;
  right: 0;
  display: flex;
  -webkit-app-region: no-drag;
}

.win-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 28px;
  color: var(--andi-window-control-light-color);
  font-size: $font-size-body;
  background: transparent;
  border: none;
  cursor: pointer;

  &:hover {
    background: var(--andi-window-control-light-hover-bg);
  }
}

.win-btn-close:hover {
  color: var(--andi-white);
  background: var(--andi-window-control-close-hover-bg);
}

.login-card {
  width: 420px;
  padding: 36px 32px;
  background: var(--andi-white);
  border-radius: var(--andi-radius-window);
  box-shadow: var(--andi-shadow-window);
  user-select: text;
  -webkit-app-region: no-drag;
}

.login-header {
  margin-bottom: $spacing-7;
  text-align: center;

  h1 {
    margin: 0;
    color: var(--andi-text);
    font-size: 17px;
    font-weight: 700;
  }

  p {
    margin: $spacing-1 0 0;
    color: var(--andi-text-secondary);
    font-size: 12px;
  }
}

.login-logo {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 56px;
  height: 56px;
  margin: 0 auto $spacing-5;
  color: var(--andi-white);
  background: linear-gradient(135deg, var(--andi-header-start), var(--andi-header-end));
  border-radius: 10px;

  svg {
    width: 28px;
    height: 28px;
    fill: currentcolor;
  }

  .login-logo-inner {
    fill: var(--andi-blue);
  }
}

.login-error {
  margin-bottom: $spacing-5;
  color: var(--andi-danger);
}

.login-button {
  width: 100%;
}

.connection-status {
  display: flex;
  align-items: center;
  justify-content: center;
  margin-top: $spacing-6;
  color: var(--andi-text-light);
  font-size: $font-size-sm;
  gap: $spacing-2;
}

.connection-dot {
  width: $connection-dot-size;
  height: $connection-dot-size;
  background: var(--andi-warning);
  border-radius: 50%;
  animation: pulse 1.5s infinite;
}

@keyframes pulse {
  0%,
  100% {
    opacity: 1;
  }

  50% {
    opacity: 0.3;
  }
}
</style>
