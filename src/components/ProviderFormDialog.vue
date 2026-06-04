<script setup lang="ts">
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useProviders, type Provider } from '@/composables/useProviders'
import { showToast } from '@/utils/toast'

const props = defineProps<{
  show: boolean
  editing: Provider | null
}>()

const emit = defineEmits<{
  'update:show': [value: boolean]
  close: []
}>()

const { addProvider, updateProvider, setProviderHealthStatus } = useProviders()

const formData = ref({
  name: '',
  apiBase: '',
  apiKey: '',
  model: '',
  timeoutMs: 5000,
  customHeaders: [{ key: '', value: '' }]
})

const errors = ref<{ [key: string]: string }>({})
const isTesting = ref(false)
const showApiKey = ref(false)

watch(() => props.editing, (val) => {
  if (val) {
    formData.value = {
      name: val.name,
      apiBase: val.apiBase,
      apiKey: val.apiKey,
      model: val.model || '',
      timeoutMs: val.timeoutMs ?? 5000,
      customHeaders: (Object.entries(val.customHeaders ?? {}) as [string, string][])
        .map(([k, v]) => ({ key: k, value: v }))
    }
  } else {
    formData.value = {
      name: '',
      apiBase: '',
      apiKey: '',
      model: '',
      timeoutMs: 5000,
      customHeaders: [{ key: '', value: '' }]
    }
  }
  errors.value = {}
}, { immediate: true })

const addHeaderRow = () => {
  formData.value.customHeaders.push({ key: '', value: '' })
}

const removeHeaderRow = (index: number) => {
  if (formData.value.customHeaders.length > 1) {
    formData.value.customHeaders.splice(index, 1)
  }
}

const validate = (): boolean => {
  errors.value = {}
  let valid = true

  if (!formData.value.name.trim()) {
    errors.value.name = '名称不能为空'
    valid = false
  }
  if (!formData.value.apiBase.trim()) {
    errors.value.apiBase = 'API Base 不能为空'
    valid = false
  } else {
    try {
      new URL(formData.value.apiBase)
    } catch {
      errors.value.apiBase = '请输入有效的 URL'
      valid = false
    }
  }
  if (!formData.value.apiKey.trim()) {
    errors.value.apiKey = 'API Key 不能为空'
    valid = false
  }

  return valid
}

const handleSubmit = async () => {
  if (!validate()) return

  const providerConfig = {
    id: props.editing?.id || '',
    name: formData.value.name.trim(),
    apiBase: formData.value.apiBase.trim().replace(/\/$/, ''),
    apiKey: formData.value.apiKey.trim(),
    model: formData.value.model.trim() || undefined,
    timeoutMs: formData.value.timeoutMs || 5000,
    customHeaders: formData.value.customHeaders
      .filter(h => h.key.trim())
      .reduce((acc, h) => {
        acc[h.key.trim()] = h.value.trim()
        return acc
      }, {} as Record<string, string>)
  }

  try {
    if (props.editing) {
      await updateProvider(props.editing.id, providerConfig)
      showToast('提供商已更新', 'success')
    } else {
      await addProvider(providerConfig)
      showToast('提供商已添加', 'success')
    }
    emit('update:show', false)
    emit('close')
  } catch (err: any) {
    errors.value.submit = err.message ?? '操作失败'
    showToast(err.message ?? '操作失败', 'error')
  }
}

const handleClose = () => {
  emit('update:show', false)
  emit('close')
}


const handleTestConnection = async () => {
  if (!formData.value.apiBase || !formData.value.apiKey) {
    showToast('请填写 API Base 和 API Key', 'error')
    return
  }

  isTesting.value = true

  try {
    const customHeadersObj = formData.value.customHeaders
      .filter(h => h.key.trim())
      .reduce((acc, h) => {
        acc[h.key.trim()] = h.value.trim()
        return acc
      }, {} as Record<string, string>)

    const result = await invoke<{ status: string; latencyMs?: number; errorMessage?: string }>(
      'test_custom_provider_health',
      {
        apiBase: formData.value.apiBase.trim().replace(/\/$/, ''),
        apiKey: formData.value.apiKey.trim(),
        model: formData.value.model.trim() || null,
        timeoutMs: formData.value.timeoutMs || 5000,
        customHeaders: customHeadersObj,
      }
    )

    if (result.status === 'ok') {
      showToast(`连接成功！(${result.latencyMs ?? '?'}ms)`, 'success')
      // 同步健康状态到 provider，使顶部 HeaderBar 和列表 health badge 更新
      if (props.editing) {
        setProviderHealthStatus(props.editing.id, result)
      }
    } else {
      showToast(result.errorMessage || '测试失败', 'error')
    }
  } catch (err: any) {
    showToast(err.message || '测试失败', 'error')
  } finally {
    isTesting.value = false
  }
}
</script>

<template>
  <Teleport to="body">
    <div v-if="show" class="dialog-overlay" @click.self="handleClose">
      <div class="dialog">
        <div class="dialog-header">
          <h3>{{ editing ? '编辑提供商' : '添加提供商' }}</h3>
          <button class="btn-close" @click="handleClose" title="关闭">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="18" y1="6" x2="6" y2="18"></line>
              <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
          </button>
        </div>

        <div class="dialog-body">
          <div class="form-group">
            <label>名称 <span class="required">*</span></label>
            <input
              v-model="formData.name"
              type="text"
              placeholder="例如：OpenAI"
              class="input"
              :class="{ 'input-error': errors.name }"
            />
            <span class="error-msg" v-if="errors.name">{{ errors.name }}</span>
          </div>

          <div class="form-group">
            <label>API Base URL <span class="required">*</span></label>
            <input
              v-model="formData.apiBase"
              type="text"
              placeholder="https://api.openai.com/v1"
              class="input"
              :class="{ 'input-error': errors.apiBase }"
            />
            <span class="error-msg" v-if="errors.apiBase">{{ errors.apiBase }}</span>
          </div>

          <div class="form-group">
            <label>API Key <span class="required">*</span></label>
            <div class="input-group">
              <input
                v-model="formData.apiKey"
                :type="showApiKey ? 'text' : 'password'"
                placeholder="sk-..."
                class="input"
                :class="{ 'input-error': errors.apiKey }"
              />
              <button
                type="button"
                class="btn btn-ghost btn-eye-toggle"
                @click="showApiKey = !showApiKey"
                :title="showApiKey ? '隐藏' : '显示'"
              >
                <svg v-if="!showApiKey" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path>
                  <circle cx="12" cy="12" r="3"></circle>
                </svg>
                <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"></path>
                  <line x1="1" y1="1" x2="23" y2="23"></line>
                </svg>
              </button>
              <button
                type="button"
                class="btn btn-secondary btn-test-conn"
                @click="handleTestConnection"
                :disabled="isTesting"
                title="测试 API 连接是否可用"
              >
                <svg v-if="isTesting" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="animate-spin">
                  <circle cx="12" cy="12" r="10"></circle>
                  <path d="M12 6v6l4 2"></path>
                </svg>
                <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <circle cx="12" cy="12" r="10"></circle>
                  <polyline points="12 6 12 12 16 14"></polyline>
                </svg>
                {{ isTesting ? '测试中...' : '测试' }}
              </button>
            </div>
            <span class="error-msg" v-if="errors.apiKey">{{ errors.apiKey }}</span>
          </div>

          <div class="form-group">
            <label>默认模型</label>
            <input
              v-model="formData.model"
              type="text"
              placeholder="gpt-4o"
              class="input"
            />
          </div>

          <div class="form-group">
            <label>超时 (ms)</label>
            <input
              v-model.number="formData.timeoutMs"
              type="number"
              min="1000"
              max="30000"
              class="input"
            />
          </div>

          <div class="form-group">
            <label>自定义 Header</label>
            <div v-for="(header, index) in formData.customHeaders" :key="index" class="header-row">
              <input
                v-model="header.key"
                type="text"
                placeholder="Key"
                class="input header-key"
              />
              <input
                v-model="header.value"
                type="text"
                placeholder="Value"
                class="input header-value"
              />
              <button
                v-if="formData.customHeaders.length > 1"
                class="btn btn-ghost btn-icon btn-remove-header"
                @click="removeHeaderRow(index)"
                title="移除这个自定义 Header"
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <line x1="18" y1="6" x2="6" y2="18"></line>
                  <line x1="6" y1="6" x2="18" y2="18"></line>
                </svg>
              </button>
            </div>
            <button class="btn btn-ghost btn-add-header" @click="addHeaderRow" title="添加一个新的自定义 Header">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="12" y1="5" x2="12" y2="19"></line>
                <line x1="5" y1="12" x2="19" y2="12"></line>
              </svg>
              添加 Header
            </button>
          </div>

          <div class="error-msg submit-error" v-if="errors.submit">
            {{ errors.submit }}
          </div>
        </div>

        <div class="dialog-footer">
          <button class="btn btn-secondary" @click="handleClose" title="取消编辑并关闭">取消</button>
          <button class="btn btn-primary" @click="handleSubmit" title="保存当前设置">保存</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.dialog {
  width: 520px;
  max-width: 95vw;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 16px;
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
  display: flex;
  flex-direction: column;
  max-height: 90vh;
  animation: dialogIn 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

@keyframes dialogIn {
  from { opacity: 0; transform: scale(0.95) translateY(10px); }
  to { opacity: 1; transform: scale(1) translateY(0); }
}

.dialog-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px;
  border-bottom: 1px solid var(--border);
}

.dialog-header h3 {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
}

.btn-close {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}

.btn-close:hover {
  background: var(--bg-card);
  color: var(--text-primary);
}

.dialog-body {
  padding: 24px;
  overflow-y: auto;
}

.form-group {
  margin-bottom: 20px;
}

.form-group label {
  display: block;
  font-size: 13px;
  color: var(--text-secondary);
  margin-bottom: 8px;
  font-weight: 500;
}

.required {
  color: var(--error);
}

.input-group {
  display: flex;
  gap: 8px;
  align-items: stretch;
}

.input-group .input {
  flex: 1;
  min-width: 0;
}

.btn-test-conn {
  flex-shrink: 0;
  padding: 10px 16px;
  min-width: 72px;
}

.btn-eye-toggle {
  flex-shrink: 0;
  width: 40px;
  padding: 0;
  color: var(--text-muted);
  border-radius: 8px;
}

.btn-eye-toggle:hover {
  color: var(--accent);
  background: var(--accent-glow);
}

.input-error {
  border-color: var(--error);
}

.input-error:focus {
  box-shadow: 0 0 0 3px var(--error-glow);
}

.error-msg {
  font-size: 12px;
  color: var(--error);
  margin-top: 6px;
}

.submit-error {
  margin-top: 16px;
  padding: 12px;
  background: var(--error-glow);
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: 8px;
  font-size: 13px;
  color: var(--error);
}

.header-row {
  display: flex;
  gap: 8px;
  margin-bottom: 8px;
  align-items: center;
}

.header-key,
.header-value {
  flex: 1;
}

.btn-remove-header {
  flex-shrink: 0;
  color: var(--text-muted);
}

.btn-remove-header:hover {
  color: var(--error);
}

.btn-add-header {
  margin-top: 8px;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 20px 24px;
  border-top: 1px solid var(--border);
}
</style>
