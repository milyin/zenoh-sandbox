<template>
  <Section title="Edit Config" icon="✏️" section-class="info-section">
    <div class="info-content">
      <div class="edit-container">
        <label class="mode-selector-label">
          <span>Zenoh Mode:</span>
          <select
            v-model="localConfig.mode"
            @change="handleModeChange"
            class="mode-selector"
            :disabled="hasActiveRuntimes"
          >
            <option value="peer">Peer</option>
            <option value="router">Router</option>
            <option value="client">Client</option>
          </select>
        </label>
        <div class="button-group">
          <button @click="handleStart" class="action-button primary">
            Start
          </button>
          <button @click="handleClone" class="action-button">
            Clone
          </button>
          <button
            @click="handleRemove"
            class="action-button danger"
            :disabled="!canRemove"
          >
            Remove
          </button>
        </div>
      </div>
    </div>
  </Section>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue';
import { useRoute } from 'vue-router';
import Section from '../components/Section.vue';
import { useNodesState } from '../composables/useNodesState';
import type { ZenohConfig } from '../types/zenohConfig';

const {
  configEntries,
  updateConfig,
  cloneConfig,
  removeConfig,
  canRemoveConfig,
  getRuntimesForConfig,
  navigateToActivityLog,
  createRuntimeFromConfig,
  navigateToConfigEdit
} = useNodesState();

const route = useRoute();
const configIndex = ref(parseInt(route.params.id as string));
const localConfig = ref<ZenohConfig>({ ...configEntries.value[configIndex.value] });

const hasActiveRuntimes = computed(() => {
  return getRuntimesForConfig(configIndex.value).length > 0;
});

const canRemove = computed(() => {
  return canRemoveConfig(configIndex.value);
});

// Watch for config changes
watch(() => configEntries.value[configIndex.value], (newConfig) => {
  if (newConfig) {
    localConfig.value = { ...newConfig };
  }
}, { deep: true });

const handleModeChange = () => {
  if (!hasActiveRuntimes.value) {
    updateConfig(configIndex.value, localConfig.value);
  }
};

const handleStart = async () => {
  await createRuntimeFromConfig(configIndex.value);
};

const handleClone = async () => {
  const newConfigIndex = await cloneConfig(configIndex.value);
  navigateToConfigEdit(newConfigIndex);
};

const handleRemove = () => {
  if (canRemove.value) {
    removeConfig(configIndex.value);
    navigateToActivityLog();
  }
};
</script>

<style scoped>
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

.mode-selector:disabled {
  background: var(--disabled-bg-color, #f5f5f5);
  color: var(--text-muted-color, #6c757d);
  cursor: not-allowed;
}

.mode-description {
  color: var(--text-muted-color, #6c757d);
  font-size: 0.9rem;
  margin: 0;
}

.readonly-notice {
  color: var(--warning-color, #ff9800);
  font-size: 0.9rem;
  margin: 0;
  font-weight: 500;
}

.button-group {
  display: flex;
  gap: 0.75rem;
  margin-top: 1.5rem;
}

.action-button {
  padding: 0.6rem 1.2rem;
  border: 1px solid var(--border-color, #dee2e6);
  border-radius: 4px;
  font-size: 0.95rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  background: var(--button-bg-color, #fff);
  color: var(--text-color, #333);
}

.action-button:hover:not(:disabled) {
  background: var(--button-hover-bg-color, #f8f9fa);
  border-color: var(--primary-color, #007bff);
}

.action-button.primary {
  background: var(--primary-color, #007bff);
  color: white;
  border-color: var(--primary-color, #007bff);
}

.action-button.primary:hover:not(:disabled) {
  background: var(--primary-hover-color, #0056b3);
  border-color: var(--primary-hover-color, #0056b3);
}

.action-button.danger {
  color: var(--danger-color, #dc3545);
}

.action-button.danger:hover:not(:disabled) {
  background: var(--danger-color, #dc3545);
  color: white;
  border-color: var(--danger-color, #dc3545);
}

.action-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
