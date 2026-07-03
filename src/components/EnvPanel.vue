<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useProviders, type ProviderEnvVars } from '@/composables/useProviders'
import { showToast } from '@/utils/toast'

const {
  activeProvider,
  claudeStatus,
  claudeApplying,
  applyToClaude,
  revertClaudeConfig,
  getProviderEnvVars,
} = useProviders()

// Env vars display state
const providerEnvVars = ref<ProviderEnvVars | null>(null)
const loadingVars = ref(false)

// 小眼睛切换
const visibleKeys = ref<Set<string>>(new Set())
const copiedKeys = ref<Set<string>>(new Set())

const toggleVisible = (name: string) => {
  if (visibleKeys.value.has(name)) {
    visibleKeys.value.delete(name)
  } else {
    visibleKeys.value.add(name)
  }
}

const isVisible = (name: string) => visibleKeys.value.has(name)

// 监听 activeProvider 变化，加载标准 env vars
watch(
  () => activeProvider.value?.id,
  async (newId) => {
    if (!newId) return
    loadingVars.value = true
    try {
      providerEnvVars.value = await getProviderEnvVars(newId)
    } catch {
      providerEnvVars.value = null
    } finally {
      loadingVars.value = false
    }
  },
  { immediate: true }
)

const formatBadgeClass = computed(() => {
  const fmt = providerEnvVars.value?.format ?? ''
  if (fmt === 'Anthropic') return 'badge-anthropic'
  if (fmt.includes('OpenAI')) return 'badge-openai'
  if (fmt.includes('Gemini')) return 'badge-gemini'
  if (fmt.includes('Ollama')) return 'badge-ollama'
  return 'badge-neutral'
})

const envEntries = computed(() => {
  if (!providerEnvVars.value) return []
  return Object.entries(providerEnvVars.value.vars).map(([name, value]) => ({
    name,
    value,
    isSecret: name.toLowerCase().includes('key') || name.toLowerCase().includes('token'),
  }))
})

// Clipboard
const copyToClipboard = async (text: string, label: string, key: string) => {
  try {
    await navigator.clipboard.writeText(text)
    copiedKeys.value.add(key)
    showToast(`${label} 已复制`, 'success')
    setTimeout(() => copiedKeys.value.delete(key), 1500)
  } catch {
    showToast('复制失败', 'error')
  }
}

const copyAll = async () => {
  const lines = envEntries.value.map(
    (e) => `export ${e.name}="${e.value}"`
  )
  if (lines.length === 0) return
  try {
    await navigator.clipboard.writeText(lines.join('\n'))
    showToast('全部环境变量已复制', 'success')
  } catch {
    showToast('复制失败', 'error')
  }
}

// Claude Code 操作
const handleApply = async () => {
  if (!activeProvider.value) return
  const err = await applyToClaude(activeProvider.value.id)
  if (err) {
    showToast(err, 'error')
  } else {
    showToast(`已应用到 Claude Code: ${activeProvider.value.name}`, 'success')
  }
}

const handleRevert = async () => {
  const err = await revertClaudeConfig()
  if (err) {
    showToast(err, 'error')
  } else {
    showToast('已从 Claude Code 移除', 'success')
  }
}
</script>

<template>
  <div class="env-panel">
    <div class="panel-content p-4 md:p-6 lg:p-8">
      <!-- Header -->
      <div class="panel-header">
        <div class="panel-title">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
            <line x1="3" y1="9" x2="21" y2="9"></line>
            <line x1="9" y1="21" x2="9" y2="9"></line>
          </svg>
          <span>环境变量</span>
          <span v-if="providerEnvVars" class="format-badge" :class="formatBadgeClass">
            {{ providerEnvVars.format }}
          </span>
        </div>
        <div class="panel-actions">
          <button class="btn btn-ghost btn-sm btn-copy-all" @click="copyAll" title="复制全部环境变量">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"></path>
              <rect x="8" y="2" width="8" height="4" rx="1" ry="1"></rect>
            </svg>
            全部复制
          </button>
        </div>
      </div>

      <!-- Loading -->
      <div v-if="loadingVars" class="env-items">
        <div class="env-item env-item-skeleton">
          <div class="skeleton-line skeleton-name"></div>
          <div class="skeleton-line skeleton-value"></div>
        </div>
      </div>

      <!-- Env Vars -->
      <div v-else-if="envEntries.length > 0" class="env-items">
        <div
          v-for="(env, idx) in envEntries"
          :key="env.name"
          class="env-item"
          :style="{ animationDelay: `${idx * 0.05}s` }"
        >
          <code class="env-name">{{ env.name }}=</code>
          <span class="env-value">
            {{ env.isSecret && !isVisible(env.name) ? (env.value ? '••••••••' : '') : env.value }}
          </span>
          <button
            v-if="env.isSecret"
            class="btn btn-ghost btn-eye"
            @click="toggleVisible(env.name)"
            :title="isVisible(env.name) ? '隐藏' : '显示'"
          >
            <svg v-if="!isVisible(env.name)" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path>
              <circle cx="12" cy="12" r="3"></circle>
            </svg>
            <svg v-else width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"></path>
              <line x1="1" y1="1" x2="23" y2="23"></line>
            </svg>
          </button>
          <button
            class="btn btn-ghost btn-copy"
            :class="{ 'btn-copy-success': copiedKeys.has(env.name) }"
            @click="copyToClipboard(`${env.name}=${env.value}`, env.name, env.name)"
          >
            <svg v-if="!copiedKeys.has(env.name)" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"></path>
              <rect x="8" y="2" width="8" height="4" rx="1" ry="1"></rect>
            </svg>
            <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="20 6 9 17 4 12"></polyline>
            </svg>
            <span class="btn-copy-text">{{ copiedKeys.has(env.name) ? '已复制' : '复制' }}</span>
          </button>
        </div>
      </div>

      <!-- Empty -->
      <div v-else class="env-items">
        <div class="env-item env-empty">
          <span class="env-empty-text">未激活任何提供商</span>
        </div>
      </div>

      <!-- Claude Code Integration Section -->
      <div class="claude-section">
        <div class="claude-header">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="10"></circle>
            <path d="M12 16v-4"></path>
            <path d="M12 8h.01"></path>
          </svg>
          <span>Claude Code 集成</span>
          <span v-if="claudeStatus.configured" class="claude-status status-on">
            <span class="status-dot status-dot-on"></span> 已应用
          </span>
          <span v-else class="claude-status status-off">
            <span class="status-dot status-dot-off"></span> 未应用
          </span>
        </div>
        <div class="claude-actions">
          <button
            class="btn btn-primary btn-sm"
            :disabled="!activeProvider || claudeApplying"
            @click="handleApply"
          >
            <svg v-if="claudeApplying" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="animate-spin">
              <circle cx="12" cy="12" r="10"></circle>
              <path d="M12 6v6l4 2"></path>
            </svg>
            <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="20 6 9 17 4 12"></polyline>
            </svg>
            {{ claudeApplying ? '应用中...' : '应用到 Claude Code' }}
          </button>
          <button
            class="btn btn-ghost btn-sm"
            :disabled="!claudeStatus.configured || claudeApplying"
            @click="handleRevert"
          >
            移除配置
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.env-panel {
  background: var(--bg-secondary);
  border-top: 1px solid var(--border);
}

.panel-content {
  max-width: 1200px;
  margin: 0 auto;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.panel-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: var(--text-secondary);
  font-weight: 500;
}

.panel-title svg {
  color: var(--text-muted);
}

.panel-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.btn-copy-all {
  padding: 4px 10px;
  font-size: 12px;
  gap: 4px;
}

.btn-copy-all:hover {
  color: var(--accent);
  background: var(--accent-glow);
}

/* Format Badge */
.format-badge {
  display: inline-flex;
  align-items: center;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.3px;
}

.badge-anthropic {
  background: rgba(200, 120, 60, 0.15);
  color: #d4874a;
}

.badge-openai {
  background: rgba(0, 166, 126, 0.15);
  color: #00a67e;
}

.badge-gemini {
  background: rgba(66, 133, 244, 0.15);
  color: #4285f4;
}

.badge-ollama {
  background: rgba(255, 255, 255, 0.1);
  color: var(--text-secondary);
}

.badge-neutral {
  background: rgba(255, 255, 255, 0.08);
  color: var(--text-muted);
}

/* Env Items */
.env-items {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.env-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 10px;
  font-family: 'SF Mono', 'Fira Code', monospace;
  font-size: 13px;
  transition: all 0.2s ease;
  animation: envSlideIn 0.3s cubic-bezier(0.4, 0, 0.2, 1) both;
}

.env-item:hover {
  border-color: var(--border-light);
  background: var(--bg-card-hover);
}

@keyframes envSlideIn {
  from { opacity: 0; transform: translateY(8px); }
  to { opacity: 1; transform: translateY(0); }
}

.env-name {
  color: var(--accent);
  min-width: 220px;
  font-weight: 500;
  flex-shrink: 0;
}

.env-value {
  color: var(--text-secondary);
  flex: 1;
  letter-spacing: 0.5px;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
}

.btn-copy, .btn-eye {
  flex-shrink: 0;
  color: var(--text-muted);
  transition: all 0.2s ease;
}

.btn-eye {
  width: 36px;
  height: 36px;
  padding: 0;
  border-radius: 8px;
}

.btn-eye svg {
  transition: transform 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.btn-eye:active svg {
  transform: scale(0.9);
}

.btn-copy {
  min-width: 72px;
  padding: 6px 12px;
  gap: 6px;
}

.btn-copy-text {
  transition: opacity 0.15s ease;
}

.btn-eye:hover {
  color: var(--accent);
  background: var(--accent-glow);
}

.btn-copy:hover {
  color: var(--accent);
  background: var(--accent-glow);
}

.btn-copy-success {
  color: var(--success) !important;
  background: var(--success-glow) !important;
}

.btn-copy-success .btn-copy-text {
  font-weight: 500;
}

/* Skeleton */
.env-item-skeleton {
  display: flex;
  gap: 12px;
  padding: 16px;
}

.skeleton-line {
  height: 14px;
  background: var(--border);
  border-radius: 4px;
  animation: pulse 1.5s ease-in-out infinite;
}

.skeleton-name {
  width: 220px;
}

.skeleton-value {
  flex: 1;
}

@keyframes pulse {
  0%, 100% { opacity: 0.3; }
  50% { opacity: 0.7; }
}

/* Empty */
.env-empty {
  justify-content: center;
  padding: 20px;
}

.env-empty-text {
  color: var(--text-muted);
  font-size: 13px;
  font-family: inherit;
}

/* Claude Code Section */
.claude-section {
  margin-top: 16px;
  padding: 12px 16px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  animation: envSlideIn 0.3s cubic-bezier(0.4, 0, 0.2, 1) 0.2s both;
}

.claude-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: var(--text-secondary);
}

.claude-header svg {
  color: var(--text-muted);
}

.claude-status {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  font-weight: 500;
  padding: 2px 8px;
  border-radius: 4px;
}

.status-on {
  color: var(--success);
  background: var(--success-glow);
}

.status-off {
  color: var(--text-muted);
  background: transparent;
  border: 1px solid var(--border);
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
}

.status-dot-on {
  background: var(--success);
}

.status-dot-off {
  background: var(--text-muted);
}

.claude-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.claude-actions .btn-primary {
  padding: 6px 14px;
  font-size: 12px;
  gap: 6px;
  border: none;
}

.claude-actions .btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.claude-actions .btn-ghost {
  padding: 6px 14px;
  font-size: 12px;
  color: var(--text-muted);
}

.claude-actions .btn-ghost:hover:not(:disabled) {
  color: var(--error);
  background: var(--error-glow);
}

.claude-actions .btn-ghost:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

</style>