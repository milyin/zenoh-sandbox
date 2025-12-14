<template>
  <Section
    id="config"
    title="Config"
    icon="⚙️"
    section-class="info-section"
  >
    <template #actions>
      <button @click="refreshConfig" :disabled="isLoadingConfig">
        🔄 Refresh
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
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import Section from '../components/Section.vue';
import { useNodesState } from '../composables/useNodesState';

interface Props {
  runtimeId: number;
}

const props = defineProps<Props>();

const { runtimes } = useNodesState();
const zenohId = computed(() => runtimes[props.runtimeId]?.zenohId || '');

const configJson = ref<string | null>(null);
const isLoadingConfig = ref(false);

const loadConfig = async () => {
  if (!zenohId.value) return;

  isLoadingConfig.value = true;
  try {
    const config = await invoke<string>('zenoh_runtime_config_json', { zid: zenohId.value });
    configJson.value = config;
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

onMounted(() => {
  loadConfig();
});
</script>

<style scoped>
.info-content {
  padding: 1rem;
  overflow-y: auto;
  flex: 1;
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
