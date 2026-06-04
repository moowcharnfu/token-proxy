import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface Provider {
  id: string
  name: string
  apiBase: string
  apiKey: string
  model?: string
  timeoutMs: number
  customHeaders: Record<string, string>
  healthStatus?: {
    status: 'ok' | 'error' | 'unknown'
    latencyMs?: number
    errorMessage?: string
    lastTested?: number
  }
}

export interface HealthResult {
  status: 'ok' | 'error'
  latencyMs?: number
  errorMessage?: string
}

export interface ClaudeStatus {
  configured: boolean
  envVars: Record<string, string>
  appliedProviderId?: string | null
}

export interface ProviderEnvVars {
  format: string
  vars: Record<string, string>
}

const providers = ref<Provider[]>([])
const activeProviderId = ref<string>('')
const loading = ref(true)
const testingId = ref<string | null>(null)

// Claude Code 集成状态
const claudeStatus = ref<ClaudeStatus>({ configured: false, envVars: {} })
const claudeApplying = ref(false)

export function useProviders() {
  const loadProviders = async () => {
    loading.value = true
    try {
      // 保存现有的 healthStatus，避免 loadProviders 覆盖丢失
      const healthMap = new Map<string, Provider['healthStatus']>()
      for (const p of providers.value) {
        if (p.healthStatus) {
          healthMap.set(p.id, p.healthStatus)
        }
      }

      const result = await invoke<{ providers: Provider[]; active_provider: string }>(
        'list_providers'
      )
      providers.value = result.providers

      // 恢复已有的 healthStatus
      for (const p of providers.value) {
        const saved = healthMap.get(p.id)
        if (saved) {
          p.healthStatus = saved
        }
      }

      activeProviderId.value = result.active_provider
      // 同时加载 Claude 状态
      await loadClaudeStatus()
    } finally {
      loading.value = false
    }
  }

  const activeProvider = computed(() =>
    providers.value.find(p => p.id === activeProviderId.value)
  )

  const addProvider = async (config: Omit<Provider, 'healthStatus'>) => {
    await invoke('add_provider', { config })
    await loadProviders()
  }

  const updateProvider = async (id: string, updates: Partial<Provider>) => {
    await invoke('update_provider', { id, updates })
    await loadProviders()
  }

  const deleteProvider = async (id: string) => {
    await invoke('delete_provider', { id })
    await loadProviders()
  }

  const setActiveProvider = async (id: string) => {
    await invoke('set_active_provider', { id })
    activeProviderId.value = id
  }

  const testProviderHealth = async (id: string): Promise<HealthResult> => {
    testingId.value = id
    try {
      const result = await invoke<HealthResult>('test_provider_health', { id })
      // 直接 mutation 而非 loadProviders()，避免不必要的后端调用
      const provider = providers.value.find(p => p.id === id)
      if (provider) {
        provider.healthStatus = {
          status: result.status,
          latencyMs: result.latencyMs,
          errorMessage: result.errorMessage,
          lastTested: Date.now(),
        }
      }
      return result
    } finally {
      testingId.value = null
    }
  }

  const setProviderHealthStatus = (id: string, result: HealthResult) => {
    const provider = providers.value.find(p => p.id === id)
    if (provider) {
      provider.healthStatus = {
        status: result.status,
        latencyMs: result.latencyMs,
        errorMessage: result.errorMessage,
        lastTested: Date.now(),
      }
    }
  }

  // === Claude Code 集成 ===

  const loadClaudeStatus = async () => {
    try {
      claudeStatus.value = await invoke<ClaudeStatus>('get_claude_status')
    } catch {
      claudeStatus.value = { configured: false, envVars: {} }
    }
  }

  const applyToClaude = async (providerId: string): Promise<string | null> => {
    claudeApplying.value = true
    try {
      await invoke('apply_provider_to_claude', { id: providerId })
      await loadClaudeStatus()
      return null
    } catch (err: any) {
      return err?.message || '应用失败'
    } finally {
      claudeApplying.value = false
    }
  }

  const revertClaudeConfig = async (): Promise<string | null> => {
    claudeApplying.value = true
    try {
      await invoke('revert_claude_config')
      await loadClaudeStatus()
      return null
    } catch (err: any) {
      return err?.message || '移除失败'
    } finally {
      claudeApplying.value = false
    }
  }

  const getProviderEnvVars = async (providerId: string): Promise<ProviderEnvVars> => {
    return await invoke<ProviderEnvVars>('get_provider_env_vars', { id: providerId })
  }

  return {
    providers,
    activeProviderId,
    activeProvider,
    loading,
    testingId,
    claudeStatus,
    claudeApplying,
    loadProviders,
    addProvider,
    updateProvider,
    deleteProvider,
    setActiveProvider,
    testProviderHealth,
    setProviderHealthStatus,
    loadClaudeStatus,
    applyToClaude,
    revertClaudeConfig,
    getProviderEnvVars,
  }
}
