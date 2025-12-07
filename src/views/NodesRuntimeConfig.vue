<template>
  <Section :title="`Config - ${runtimeId}`" icon="⚙️" section-class="info-section">
    <template #actions>
      <button @click="refreshConfig" :disabled="isLoadingConfig">
        🔄 Refresh
      </button>
      <button @click="$emit('navigate-to-activity-log')">
        ✕ Close
      </button>
    </template>
    <div class="info-content">
      <div v-if="isLoadingConfig" class="loading">
        Loading config...
      </div>
      <div v-else-if="!configJson" class="empty-config">
        No config available
      </div>
      <pre v-else class="config-json">{{ configJson }}</pre>
    </div>
  </Section>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import Section from '../components/Section.vue';

defineEmits<{
  'navigate-to-activity-log': []
}>();

const route = useRoute();
const runtimeId = ref(route.params.id as string);
const configJson = ref<string | null>(null);
const isLoadingConfig = ref(false);

const loadConfig = async () => {
  isLoadingConfig.value = true;
  try {
    const config = await invoke<any>('get_runtime_config', { id: runtimeId.value });
    configJson.value = JSON.stringify(config, null, 2);
  } catch (error) {
    console.error('Failed to load config:', error);
    configJson.value = null;
  } finally {
    isLoadingConfig.value = false;
  }
};

const refreshConfig = () => {
  loadConfig();
};

// Load config on mount
onMounted(() => {
  loadConfig();
});
</script>

<style scoped>
.info-content {
  padding: 1rem;
  overflow-y: auto;
  max-height: calc(100vh - 200px);
}

.loading,
.empty-config {
  padding: 2rem;
  text-align: center;
  color: var(--text-muted-color, #6c757d);
}

.config-json {
  background: var(--code-bg-color, #f8f9fa);
  padding: 1rem;
  border-radius: 4px;
  overflow-x: auto;
  font-size: 0.9rem;
  line-height: 1.5;
  margin: 0;
}
</style>
