<script setup lang="ts">
import { onMounted } from 'vue'
import HeaderBar from './components/HeaderBar.vue'
import ProviderList from './components/ProviderList.vue'
import EnvPanel from './components/EnvPanel.vue'
import { useProviders } from '@/composables/useProviders'

const { loadProviders } = useProviders()

onMounted(async () => {
  await loadProviders()
})
</script>

<template>
  <div class="app-container">
    <HeaderBar />
    <main class="main-content p-4 md:p-6 lg:p-8">
      <ProviderList />
    </main>
    <EnvPanel />
  </div>
</template>

<style scoped>
.app-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
}

.main-content {
  flex: 1;
  overflow-y: auto;
}
</style>
