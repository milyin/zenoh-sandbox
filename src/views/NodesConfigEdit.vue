<template>
  <Section title="Edit Config" icon="✏️" section-class="info-section">
    <div class="info-content">
      <div class="edit-container">
        <!-- Tabs -->
        <div class="tabs">
          <button
            :class="['tab', { active: activeTab === 'dialog' }]"
            @click="activeTab = 'dialog'"
          >
            Dialog
          </button>
          <button
            :class="['tab', { active: activeTab === 'json' }]"
            @click="activeTab = 'json'"
          >
            JSON
          </button>
        </div>

        <!-- Tab Content -->
        <div class="tab-content">
          <!-- Dialog Tab -->
          <div v-if="activeTab === 'dialog'" class="dialog-tab">
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
          </div>

          <!-- JSON Tab -->
          <div v-if="activeTab === 'json'" class="json-tab">
            <textarea
              v-model="jsonString"
              class="json-editor"
              :disabled="hasActiveRuntimes"
              spellcheck="false"
            ></textarea>
            <div class="json-actions">
              <button
                @click="handleApplyJson"
                class="action-button primary"
                :disabled="hasActiveRuntimes"
              >
                Apply
              </button>
              <span v-if="jsonError" class="json-error">{{ jsonError }}</span>
              <span v-if="jsonSuccess" class="json-success">{{ jsonSuccess }}</span>
            </div>
          </div>
        </div>

        <!-- Action Buttons (outside tabs) -->
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
import { ZenohConfig } from '../types/zenohConfig';

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
const localConfig = ref<ZenohConfig>(configEntries.value[configIndex.value].clone());

const activeTab = ref<'dialog' | 'json'>('dialog');
const jsonString = ref('');
const jsonError = ref('');
const jsonSuccess = ref('');

// Initialize JSON string from config
const updateJsonString = () => {
  jsonString.value = localConfig.value.toJsonString(true);
};

// Call on mount
updateJsonString();

const hasActiveRuntimes = computed(() => {
  return getRuntimesForConfig(configIndex.value).length > 0;
});

const canRemove = computed(() => {
  return canRemoveConfig(configIndex.value);
});

// Watch for config changes
watch(() => configEntries.value[configIndex.value], (newConfig) => {
  if (newConfig) {
    localConfig.value = newConfig.clone();
    updateJsonString();
  }
}, { deep: true });

// Update JSON string when switching to JSON tab
watch(activeTab, (newTab) => {
  if (newTab === 'json') {
    updateJsonString();
    jsonError.value = '';
    jsonSuccess.value = '';
  }
});

const handleModeChange = () => {
  if (!hasActiveRuntimes.value) {
    updateConfig(configIndex.value, localConfig.value);
  }
};

const handleApplyJson = async () => {
  jsonError.value = '';
  jsonSuccess.value = '';

  try {
    // Parse JSON
    const parsedJson = JSON.parse(jsonString.value);

    // Verify and update config
    await localConfig.value.setJson(parsedJson);

    // Update in state
    updateConfig(configIndex.value, localConfig.value);

    jsonSuccess.value = 'Configuration applied successfully!';

    // Clear success message after 3 seconds
    setTimeout(() => {
      jsonSuccess.value = '';
    }, 3000);
  } catch (error: any) {
    jsonError.value = error.message || 'Invalid JSON or configuration';
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

/* Tabs */
.tabs {
  display: flex;
  gap: 0.25rem;
  border-bottom: 2px solid var(--border-color, #dee2e6);
  margin-bottom: 1rem;
}

.tab {
  padding: 0.6rem 1.2rem;
  background: transparent;
  border: none;
  border-bottom: 2px solid transparent;
  color: var(--text-muted-color, #6c757d);
  font-size: 0.95rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  margin-bottom: -2px;
}

.tab:hover {
  color: var(--text-color, #333);
  background: var(--button-hover-bg-color, #f8f9fa);
}

.tab.active {
  color: var(--primary-color, #007bff);
  border-bottom-color: var(--primary-color, #007bff);
}

/* Tab Content */
.tab-content {
  min-height: 200px;
}

.dialog-tab,
.json-tab {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

/* Dialog Tab */
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

/* JSON Tab */
.json-editor {
  width: 100%;
  min-height: 300px;
  padding: 0.75rem;
  border: 1px solid var(--border-color, #dee2e6);
  border-radius: 4px;
  font-family: 'Courier New', Courier, monospace;
  font-size: 0.9rem;
  line-height: 1.5;
  background: var(--input-bg-color, #fff);
  color: var(--text-color, #333);
  resize: vertical;
}

.json-editor:disabled {
  background: var(--disabled-bg-color, #f5f5f5);
  color: var(--text-muted-color, #6c757d);
  cursor: not-allowed;
}

.json-actions {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.json-error {
  color: var(--danger-color, #dc3545);
  font-size: 0.9rem;
  font-weight: 500;
}

.json-success {
  color: var(--success-color, #28a745);
  font-size: 0.9rem;
  font-weight: 500;
}

/* Action Buttons */
.button-group {
  display: flex;
  gap: 0.75rem;
  margin-top: 1.5rem;
  padding-top: 1.5rem;
  border-top: 1px solid var(--border-color, #dee2e6);
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
