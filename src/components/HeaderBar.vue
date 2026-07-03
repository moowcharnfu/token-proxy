<script setup lang="ts">
import { computed } from 'vue'
import { useProviders } from '@/composables/useProviders'
import { showToast } from '@/utils/toast'

const { providers, activeProviderId, activeProvider, setActiveProvider, loading, claudeStatus } = useProviders()

const handleSwitch = async (id: string) => {
  try {
    await setActiveProvider(id)
    const provider = providers.value.find(p => p.id === id)
    showToast(`已切换到 ${provider?.name}`, 'success')
  } catch (err: any) {
    showToast(err.message || '切换失败', 'error')
  }
}

const healthStatus = computed(() => {
  const h = activeProvider.value?.healthStatus
  if (!h) return { key: 'neutral', label: '未测试' }
  if (h.status === 'ok') return { key: 'success', label: `响应 ${h.latencyMs}ms` }
  if (h.status === 'error') return { key: 'error', label: h.errorMessage?.slice(0, 40) || '连接失败' }
  return { key: 'neutral', label: '未测试' }
})
</script>

<template>
  <header class="header">
    <div class="header-left">
      <div class="logo">
        <div class="logo-icon">
          <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </div>
        <span class="logo-text">Token Proxy</span>
      </div>
    </div>

    <div class="header-right">
      <div class="active-provider-card" v-if="!loading">
        <span class="label">当前激活</span>
        <div class="provider-info">
          <span class="status-dot" :class="healthStatus.key"></span>
          <span class="provider-name">{{ activeProvider?.name ?? '无' }}</span>
          <span class="model-name" v-if="activeProvider?.model">
            · {{ activeProvider.model }}
          </span>
          <span class="status-label">{{ healthStatus.label }}</span>
          <span v-if="claudeStatus.configured" class="claude-badge" title="已应用到 Claude Code">
            <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="20 6 9 17 4 12"></polyline>
            </svg>
            Claude
          </span>
        </div>
        <div class="switch-wrapper">
          <select
            v-model="activeProviderId"
            @change="handleSwitch($event.target.value)"
            class="provider-select"
          >
            <option
              v-for="p in providers"
              :key="p.id"
              :value="p.id"
            >
              {{ p.name }}{{ p.model ? ` (${p.model})` : '' }}
            </option>
          </select>
        </div>
      </div>
      <!-- Loading Skeleton -->
      <div class="active-provider-card skeleton-card" v-else>
        <span class="label">当前激活</span>
        <div class="skeleton-info">
          <div class="skeleton-dot"></div>
          <div class="skeleton-line skeleton-name"></div>
        </div>
      </div>
    </div>
  </header>
</template>

<style scoped>
.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 24px;
  background: linear-gradient(180deg, var(--bg-secondary) 0%, var(--bg-primary) 100%);
  border-bottom: 1px solid var(--border);
  -webkit-app-region: drag;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.logo {
  display: flex;
  align-items: center;
  gap: 12px;
}

.logo-icon {
  width: 36px;
  height: 36px;
  border-radius: 10px;
  background: linear-gradient(135deg, var(--gradient-start) 0%, var(--gradient-end) 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
}

.logo-icon svg {
  width: 20px;
  height: 20px;
}

.logo-text {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  letter-spacing: -0.02em;
}

.header-right {
  -webkit-app-region: no-drag;
  min-width: 0;
}

.active-provider-card {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 14px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 10px;
  transition: border-color 0.2s ease;
}

.active-provider-card:hover {
  border-color: var(--border-light);
}

.label {
  font-size: 10px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.06em;
  font-weight: 600;
  flex-shrink: 0;
}

.provider-info {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
}

.provider-name {
  font-weight: 600;
  font-size: 13px;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 140px;
}

.model-name {
  color: var(--text-secondary);
  font-size: 12px;
  white-space: nowrap;
}

.status-label {
  font-size: 11px;
  color: var(--text-muted);
  padding-left: 6px;
  border-left: 1px solid var(--border);
  white-space: nowrap;
}

.switch-wrapper {
  position: relative;
  flex-shrink: 0;
}

.provider-select {
  min-width: 140px;
  padding: 5px 24px 5px 8px;
  font-size: 12px;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--bg-primary);
  color: var(--text-primary);
  cursor: pointer;
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%238a8a9a' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpolyline points='6 9 12 15 18 9'%3E%3C/polyline%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 6px center;
  background-size: 12px;
  transition: border-color 0.2s ease;
}

.provider-select:hover {
  border-color: var(--accent);
}

.provider-select:focus {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 2px var(--accent-glow);
}

/* Loading Skeleton */
.skeleton-card {
  pointer-events: none;
  opacity: 0.7;
}

.skeleton-info {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
}

.skeleton-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--bg-card-hover);
  animation: skeletonPulse 1.5s ease-in-out infinite;
}

.skeleton-line {
  height: 12px;
  border-radius: 4px;
  background: var(--bg-card-hover);
  animation: skeletonPulse 1.5s ease-in-out infinite;
}

.skeleton-name {
  width: 100px;
}

.claude-badge {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 9px;
  font-weight: 600;
  background: var(--success-glow);
  color: var(--success);
  letter-spacing: 0.3px;
  white-space: nowrap;
  flex-shrink: 0;
}
</style>
