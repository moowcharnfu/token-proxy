<script setup lang="ts">
import { computed } from 'vue'
import { useProviders, type Provider } from '@/composables/useProviders'

const props = defineProps<{
  provider: Provider
  isTesting?: boolean
}>()

const emit = defineEmits<{
  test: [id: string]
  edit: [provider: Provider]
  delete: [id: string]
}>()

const { activeProviderId, claudeStatus } = useProviders()

const isActive = computed(() => props.provider.id === activeProviderId.value)

const isClaudeApplied = computed(() => {
  return claudeStatus.value.configured && claudeStatus.value.appliedProviderId === props.provider.id
})

const statusConfig = computed(() => {
  const h = props.provider.healthStatus
  if (!h) return { color: 'neutral', label: '未测试', title: '' }
  if (h.status === 'ok') return { color: 'success', label: `${h.latencyMs}ms`, title: '' }
  if (h.status === 'error') {
    const msg = h.errorMessage || '连接失败'
    return { color: 'error', label: '连接失败', title: msg }
  }
  return { color: 'neutral', label: '未测试', title: '' }
})
</script>

<template>
  <div class="provider-item" :class="{ 'provider-active': isActive }">
    <div class="provider-main">
      <div class="provider-icon">
        {{ provider.name.charAt(0).toUpperCase() }}
      </div>
      <div class="provider-info">
        <div class="provider-header">
          <span class="provider-name">{{ provider.name }}</span>
          <span v-if="isActive" class="active-badge">当前</span>
          <span v-if="isClaudeApplied" class="claude-badge">Claude Code</span>
          <span class="provider-model" v-if="provider.model">{{ provider.model }}</span>
        </div>
        <div class="provider-meta">
          <span class="provider-id">{{ provider.id }}</span>
          <span class="health-badge" :class="`badge-${statusConfig.color}`" :title="statusConfig.title">
            <!-- Health: ok -->
            <svg v-if="statusConfig.color === 'success'" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" class="health-icon">
              <polyline points="20 6 9 17 4 12"></polyline>
            </svg>
            <!-- Health: error -->
            <svg v-else-if="statusConfig.color === 'error'" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" class="health-icon">
              <line x1="18" y1="6" x2="6" y2="18"></line>
              <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
            <!-- Health: neutral -->
            <svg v-else width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" class="health-icon">
              <circle cx="12" cy="12" r="10"></circle>
            </svg>
            {{ statusConfig.label }}
          </span>
        </div>
      </div>
    </div>

    <div class="provider-actions">
      <button class="btn btn-ghost btn-icon test-btn" :class="{ 'btn-testing': isTesting }" @click="emit('test', provider.id)" :title="isTesting ? '测试中...' : '测试连通性'" :disabled="isTesting">
        <svg v-if="!isTesting" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="10"></circle>
          <polyline points="12 6 12 12 16 14"></polyline>
        </svg>
        <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="animate-spin">
          <circle cx="12" cy="12" r="10"></circle>
          <path d="M12 6v6l4 2"></path>
        </svg>
      </button>
      <button class="btn btn-ghost btn-icon edit-btn" @click="emit('edit', provider)" title="编辑">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
          <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
        </svg>
      </button>
      <button class="btn btn-ghost btn-icon delete-btn" @click="emit('delete', provider.id)" title="删除">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="3 6 5 6 21 6"></polyline>
          <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
.provider-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 12px;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
}

.provider-item:hover {
  background: var(--bg-card-hover);
  border-color: var(--accent);
  box-shadow: 0 4px 20px rgba(99, 102, 241, 0.08);
}

/* 激活态高亮 */
.provider-active {
  border-color: var(--accent);
  background: linear-gradient(135deg, var(--bg-card) 0%, var(--accent-glow) 100%);
}

.provider-active::before {
  content: '';
  position: absolute;
  left: 0;
  top: 8px;
  bottom: 8px;
  width: 3px;
  background: var(--accent);
  border-radius: 0 3px 3px 0;
}

.provider-main {
  display: flex;
  align-items: center;
  gap: 16px;
  flex: 1;
}

.provider-icon {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  background: linear-gradient(135deg, var(--gradient-start) 0%, var(--gradient-end) 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-weight: 600;
  font-size: 18px;
  flex-shrink: 0;
}

.provider-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.provider-header {
  display: flex;
  align-items: center;
  gap: 8px;
}

.provider-name {
  font-weight: 600;
  font-size: 15px;
  color: var(--text-primary);
}

.active-badge {
  display: inline-flex;
  align-items: center;
  padding: 1px 6px;
  border-radius: 4px;
  font-size: 10px;
  font-weight: 600;
  background: var(--accent-glow);
  color: var(--accent);
  letter-spacing: 0.3px;
}

.claude-badge {
  display: inline-flex;
  align-items: center;
  padding: 1px 6px;
  border-radius: 4px;
  font-size: 10px;
  font-weight: 600;
  background: rgba(255, 255, 255, 0.08);
  color: var(--text-secondary);
  letter-spacing: 0.3px;
}

.provider-model {
  color: var(--text-secondary);
  font-size: 13px;
}

.provider-meta {
  display: flex;
  align-items: center;
  gap: 12px;
}

.provider-id {
  font-size: 12px;
  color: var(--text-muted);
  font-family: 'SF Mono', 'Fira Code', monospace;
}

.health-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 3px 8px;
  border-radius: 6px;
  font-size: 11px;
  font-weight: 500;
}

.health-icon {
  flex-shrink: 0;
}

.badge-success {
  color: var(--success);
  background: var(--success-glow);
}

.badge-error {
  color: var(--error);
  background: var(--error-glow);
}

.badge-neutral {
  color: var(--text-secondary);
  background: rgba(255, 255, 255, 0.06);
}

.provider-actions {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}

.test-btn:hover:not(:disabled) {
  color: var(--success);
  background: var(--success-glow);
}

.test-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-testing {
  color: var(--success) !important;
}

.edit-btn:hover {
  color: var(--accent);
  background: var(--accent-glow);
}

.delete-btn:hover {
  color: var(--error);
  background: var(--error-glow);
}

</style>