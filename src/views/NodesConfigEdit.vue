<template>
  <div class="nodes-view">
    <NodesEntityPanel />
    <div class="content-panel">
      <Section title="Edit Config" icon="✏️" section-class="info-section">
        <template #actions>
          <button @click="navigateToActivityLog">
            ✕ Close
          </button>
        </template>
        <div class="info-content">
          <div class="edit-container">
            <label class="mode-selector-label">
              <span>Zenoh Mode:</span>
              <select v-model="localConfig.mode" @change="handleModeChange" class="mode-selector">
                <option value="peer">Peer</option>
                <option value="router">Router</option>
                <option value="client">Client</option>
              </select>
            </label>
            <p class="mode-description">
              Port will be automatically assigned by the system.
            </p>
          </div>
        </div>
      </Section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { useRoute } from 'vue-router';
import Section from '../components/Section.vue';
import NodesEntityPanel from '../components/NodesEntityPanel.vue';
import { useNodesState } from '../composables/useNodesState';
import type { ZenohConfig } from '../types/zenohConfig';

const { configEntries, updateConfig, navigateToActivityLog } = useNodesState();

const route = useRoute();
const configIndex = ref(parseInt(route.params.id as string));
const localConfig = ref<ZenohConfig>({ ...configEntries.value[configIndex.value] });

// Watch for config changes
watch(() => configEntries.value[configIndex.value], (newConfig) => {
  if (newConfig) {
    localConfig.value = { ...newConfig };
  }
}, { deep: true });

const handleModeChange = () => {
  updateConfig(configIndex.value, localConfig.value);
};
</script>

<style scoped>
.nodes-view {
  display: flex;
  height: 100%;
  width: 100%;
}

.content-panel {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}

.info-content {
  padding: 1rem;
  overflow-y: auto;
  max-height: calc(100vh - 200px);
}

.edit-container {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.mode-selector-label {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  font-weight: 500;
}

.mode-selector {
  padding: 0.5rem;
  border: 1px solid var(--border-color, #dee2e6);
  border-radius: 4px;
  font-size: 1rem;
  background: var(--input-bg-color, #fff);
  color: var(--text-color, #333);
}

.mode-description {
  color: var(--text-muted-color, #6c757d);
  font-size: 0.9rem;
  margin: 0;
}
</style>
