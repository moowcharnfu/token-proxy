<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import ProviderItem from './ProviderItem.vue'
import ProviderFormDialog from './ProviderFormDialog.vue'
import { useProviders, type Provider } from '@/composables/useProviders'
import { showToast } from '@/utils/toast'

const { providers, activeProviderId, testProviderHealth, deleteProvider, loading, testingId } = useProviders()

// 激活 provider 改变时自动滚动到该项
watch(activeProviderId, () => {
  nextTick(() => {
    const el = document.querySelector('.provider-active')
    if (el) el.scrollIntoView({ behavior: 'smooth', block: 'center' })
  })
})

const showAddDialog = ref(false)
const editingProvider = ref<Provider | null>(null)
const deleteConfirmId = ref<string | null>(null)

const handleEdit = (provider: Provider) => {
  editingProvider.value = provider
  showAddDialog.value = true
}

const handleAdd = () => {
  editingProvider.value = null
  showAddDialog.value = true
}

const handleDialogClose = () => {
  showAddDialog.value = false
  editingProvider.value = null
}

const confirmDelete = (id: string) => {
  deleteConfirmId.value = id
}

const handleDelete = async () => {
  if (!deleteConfirmId.value) return
  try {
    await deleteProvider(deleteConfirmId.value)
    showToast('提供商已删除', 'success')
  } catch (err: any) {
    showToast(err.message || '删除失败', 'error')
  } finally {
    deleteConfirmId.value = null
  }
}

const cancelDelete = () => {
  deleteConfirmId.value = null
}

const handleTest = async (id: string) => {
  try {
    await testProviderHealth(id)
  } catch (err: any) {
    showToast(err.message || '测试失败', 'error')
  }
}
</script>

<template>
  <div class="provider-list w-full max-w-5xl mx-auto">
    <div class="list-header mb-4 md:mb-6 lg:mb-8">
      <div>
        <h2 class="section-title">提供商列表</h2>
        <p class="section-subtitle">管理你的 AI API Token 提供商</p>
      </div>
      <button class="btn btn-primary" @click="handleAdd">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="12" y1="5" x2="12" y2="19"></line>
          <line x1="5" y1="12" x2="19" y2="12"></line>
        </svg>
        添加提供商
      </button>
    </div>

    <div class="list-items grid grid-cols-1 xl:grid-cols-2 gap-3 md:gap-4">
      <!-- Loading Skeleton -->
      <template v-if="loading">
        <div v-for="i in 3" :key="i" class="skeleton-item">
          <div class="skeleton-icon"></div>
          <div class="skeleton-main">
            <div class="skeleton-line skeleton-title"></div>
            <div class="skeleton-line skeleton-meta"></div>
          </div>
          <div class="skeleton-actions">
            <div class="skeleton-btn"></div>
            <div class="skeleton-btn"></div>
            <div class="skeleton-btn"></div>
          </div>
        </div>
      </template>

      <div v-else-if="providers.length === 0" class="empty-state">
        <div class="empty-icon">
          <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path d="M20 7L10 17L4 11" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </div>
        <h3>暂无提供商</h3>
        <p>添加你的第一个 AI API Token 提供商</p>
        <button class="btn btn-primary empty-cta" @click="handleAdd">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="12" y1="5" x2="12" y2="19"></line>
            <line x1="5" y1="12" x2="19" y2="12"></line>
          </svg>
          添加提供商
        </button>
      </div>

      <ProviderItem
        v-for="p in providers"
        :key="p.id"
        :provider="p"
        :is-testing="testingId === p.id"
        @test="handleTest"
        @edit="handleEdit"
        @delete="confirmDelete"
      />
    </div>

    <ProviderFormDialog
      v-model:show="showAddDialog"
      :editing="editingProvider"
      @close="handleDialogClose"
    />

    <!-- Custom Delete Confirmation Modal -->
    <Teleport to="body">
      <div v-if="deleteConfirmId" class="delete-modal-overlay" @click.self="cancelDelete">
        <div class="delete-modal">
          <div class="delete-modal-icon">
            <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="10"></circle>
              <line x1="15" y1="9" x2="9" y2="15"></line>
              <line x1="9" y1="9" x2="15" y2="15"></line>
            </svg>
          </div>
          <h3 class="delete-modal-title">确认删除</h3>
          <p class="delete-modal-desc">删除后无法恢复，确定要删除这个提供商吗？</p>
          <div class="delete-modal-actions">
            <button class="btn btn-secondary" @click="cancelDelete">取消</button>
            <button class="btn btn-error" @click="handleDelete">删除</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.provider-list {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.list-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  padding-bottom: 20px;
  border-bottom: 1px solid var(--border);
}

.section-title {
  font-size: 20px;
  font-weight: 600;
  color: var(--text-primary);
  letter-spacing: -0.02em;
}

.section-subtitle {
  font-size: 13px;
  color: var(--text-muted);
  margin-top: 4px;
}

.list-items .provider-item {
  animation: itemSlideIn 0.35s cubic-bezier(0.4, 0, 0.2, 1) both;
}

.list-items .provider-item:nth-child(1) { animation-delay: 0.05s; }
.list-items .provider-item:nth-child(2) { animation-delay: 0.1s; }
.list-items .provider-item:nth-child(3) { animation-delay: 0.15s; }
.list-items .provider-item:nth-child(4) { animation-delay: 0.2s; }
.list-items .provider-item:nth-child(5) { animation-delay: 0.25s; }

@keyframes itemSlideIn {
  from { opacity: 0; transform: translateY(12px); }
  to { opacity: 1; transform: translateY(0); }
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  text-align: center;
}

.empty-icon {
  width: 64px;
  height: 64px;
  border-radius: 16px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
  margin-bottom: 16px;
}

.empty-icon svg {
  width: 32px;
  height: 32px;
}

.empty-state h3 {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 6px;
}

.empty-state p {
  font-size: 13px;
  color: var(--text-muted);
  margin-bottom: 20px;
}

.empty-cta {
  margin: 0 auto;
}

.empty-state .empty-icon {
  opacity: 0.6;
  margin-bottom: 20px;
}

/* Loading Skeleton */
.skeleton-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 12px;
  animation: skeletonPulse 1.5s ease-in-out infinite;
}

.skeleton-icon {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  background: var(--bg-card-hover);
  flex-shrink: 0;
}

.skeleton-main {
  flex: 1;
  margin-left: 16px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 0;
}

.skeleton-line {
  height: 14px;
  border-radius: 4px;
  background: var(--bg-card-hover);
}

.skeleton-title {
  width: 60%;
}

.skeleton-meta {
  width: 40%;
}

.skeleton-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
  margin-left: 16px;
}

.skeleton-btn {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  background: var(--bg-card-hover);
}

/* Delete Confirmation Modal */
.delete-modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1001;
  animation: modalFadeIn 0.2s ease-out;
}

.delete-modal {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 16px;
  padding: 28px 32px;
  width: 400px;
  max-width: 90vw;
  text-align: center;
  animation: modalSlideIn 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.delete-modal-icon {
  width: 56px;
  height: 56px;
  border-radius: 50%;
  background: var(--error-glow);
  border: 1px solid rgba(239, 68, 68, 0.3);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--error);
  margin: 0 auto 16px;
}

.delete-modal-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 8px;
}

.delete-modal-desc {
  font-size: 14px;
  color: var(--text-secondary);
  margin-bottom: 24px;
  line-height: 1.5;
}

.delete-modal-actions {
  display: flex;
  gap: 12px;
  justify-content: center;
}

.btn-error {
  background: var(--error);
  color: white;
  border: none;
}

.btn-error:hover {
  background: #dc2626;
  box-shadow: 0 4px 20px var(--error-glow);
}

@keyframes modalFadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes modalSlideIn {
  from { opacity: 0; transform: scale(0.95) translateY(-10px); }
  to { opacity: 1; transform: scale(1) translateY(0); }
}
</style>
