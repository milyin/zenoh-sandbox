<template>
  <Section title="Edit Config" icon="✏️" section-class="info-section">
    <div class="info-content">
      <div class="edit-container">
        <div class="button-group">
          <button @click="handleStart" class="action-button primary" :disabled="!!validationError">Start</button>
          <button @click="handleClone" class="action-button">Clone</button>
          <button @click="handleReset" class="action-button" :disabled="hasActiveRuntimes">Reset</button>
          <button @click="handleOpen" class="action-button" :disabled="hasActiveRuntimes">Open</button>
          <button @click="handleRemove" class="action-button danger" :disabled="!canRemove">
            Remove
          </button>
        </div>
        <input
          ref="fileInput"
          type="file"
          accept=".json,.json5"
          style="display: none"
          @change="handleFileSelected"
        />

        <div class="split-panel">
          <!-- Left Panel: Editor -->
          <div class="editor-panel">
            <div class="panel-header">
              <span class="panel-title">Edit (JSON5)</span>
              <span v-if="isValidating" class="status-badge validating">Validating...</span>
              <span v-else-if="!validationError" class="status-badge valid">Valid</span>
              <span v-else class="status-badge invalid">Invalid</span>
            </div>
            <textarea
              v-model="editContent"
              class="json5-editor"
              :disabled="hasActiveRuntimes"
              spellcheck="false"
              @input="handleEditInput"
            ></textarea>
            <div v-if="validationError" class="validation-error">
              {{ validationError }}
            </div>
          </div>

          <!-- Right Panel: Config Diff -->
          <div class="json-panel">
            <div class="panel-header">
              <span class="panel-title">Diff from Default</span>
            </div>
            <textarea
              v-model="diffJsonString"
              class="json-display"
              readonly
              spellcheck="false"
            ></textarea>
          </div>
        </div>
      </div>
    </div>
  </Section>
</template>

<script setup lang="ts">
import { ref, watch, computed, onMounted, onUnmounted } from "vue";
import { useRoute } from "vue-router";
import Section from "../components/Section.vue";
import { useNodesState } from "../composables/useNodesState";
import {
  type ZenohConfigEdit,
  validateConfig,
  getDefaultConfigJson,
} from "../types/zenohConfig";

const {
  configs,
  cloneConfig,
  removeConfig,
  canRemoveConfig,
  getRuntimesForConfig,
  navigateToActivityLog,
  startRuntimeWithNavigation,
  navigateToConfigEdit,
  updateConfig,
  getConfigDiffFormatted,
} = useNodesState();

const route = useRoute();
const configId = ref(parseInt(route.params.id as string));

const editContent = ref("");
const diffJsonString = ref("");
const validationError = ref("");
const isValidating = ref(false);
const fileInput = ref<HTMLInputElement | null>(null);

let validationTimeout: ReturnType<typeof setTimeout> | null = null;
const VALIDATION_DEBOUNCE_MS = 500;

const hasActiveRuntimes = computed(() => {
  return getRuntimesForConfig(configId.value).length > 0;
});

const canRemove = computed(() => {
  return canRemoveConfig(configId.value);
});

const initializeFromConfig = () => {
  const entry = configs[configId.value];
  if (!entry) return;

  // Initialize editor with full config
  const fullConfigStr = JSON.stringify(entry.config.configJson, null, 2);
  editContent.value = fullConfigStr;

  // Update diff display
  updateDiff();
};

const updateDiff = () => {
  diffJsonString.value = getConfigDiffFormatted(configId.value);
};

const handleEditInput = () => {
  if (hasActiveRuntimes.value) return;

  if (validationTimeout) {
    clearTimeout(validationTimeout);
  }

  isValidating.value = true;
  validationError.value = "";

  validationTimeout = setTimeout(async () => {
    await validateEdit();
    isValidating.value = false;
  }, VALIDATION_DEBOUNCE_MS);
};

const validateEdit = async () => {
  try {
    // Validate with Zenoh's parser
    await validateConfig(editContent.value);

    const newEdit: ZenohConfigEdit = { content: editContent.value };
    await updateConfig(configId.value, newEdit);

    // Update diff after successful validation
    updateDiff();

    validationError.value = "";
  } catch (error: any) {
    validationError.value = error || "Invalid JSON5 or configuration";
  }
};

const handleStart = async () => {
  // Extra safety check - button should already be disabled
  if (validationError.value) {
    return;
  }

  await startRuntimeWithNavigation(configId.value);
};

const handleClone = async () => {
  const newConfigId = await cloneConfig(configId.value);
  navigateToConfigEdit(newConfigId);
};

const handleRemove = () => {
  if (canRemove.value) {
    removeConfig(configId.value);
    navigateToActivityLog();
  }
};

const handleReset = async () => {
  if (hasActiveRuntimes.value) return;

  try {
    const defaultConfig = await getDefaultConfigJson();
    editContent.value = defaultConfig;
    handleEditInput();
  } catch (error) {
    console.error('Failed to load default config:', error);
    validationError.value = 'Failed to load default configuration';
  }
};

const handleOpen = () => {
  if (hasActiveRuntimes.value) return;
  fileInput.value?.click();
};

const handleFileSelected = async (event: Event) => {
  const target = event.target as HTMLInputElement;
  const file = target.files?.[0];

  if (!file) return;

  try {
    const text = await file.text();
    editContent.value = text;
    handleEditInput();

    // Reset the file input so the same file can be selected again
    target.value = '';
  } catch (error) {
    console.error('Failed to read file:', error);
    validationError.value = 'Failed to read file';
  }
};

watch(
  () => configs[configId.value],
  (newEntry) => {
    if (newEntry && !isValidating.value) {
      initializeFromConfig();
    }
  },
  { deep: true }
);

onMounted(() => {
  initializeFromConfig();
});

onUnmounted(() => {
  if (validationTimeout) {
    clearTimeout(validationTimeout);
  }
});
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

/* Action Buttons */
.button-group {
  display: flex;
  gap: 0.75rem;
  padding-bottom: 1rem;
  border-bottom: 1px solid var(--border-color, #dee2e6);
}

/* Split Panel Layout */
.split-panel {
  display: flex;
  gap: 1rem;
  min-height: 400px;
  max-height: calc(100vh - 300px);
}

/* Panel Headers */
.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.5rem 0.75rem;
  background: var(--header-bg-color, #e9ecef);
  border-bottom: 1px solid var(--border-color, #dee2e6);
  border-radius: 4px 4px 0 0;
  font-size: 0.85rem;
  font-weight: 600;
}

.panel-title {
  color: var(--text-muted-color, #6c757d);
}

/* Status Badges */
.status-badge {
  padding: 0.25rem 0.5rem;
  border-radius: 3px;
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
}

.status-badge.validating {
  background: var(--info-bg-color, #d1ecf1);
  color: var(--info-color, #0c5460);
}

.status-badge.valid {
  background: var(--success-bg-color, #d4edda);
  color: var(--success-color, #155724);
}

.status-badge.invalid {
  background: var(--danger-bg-color, #f8d7da);
  color: var(--danger-color, #721c24);
}

/* Editor Panel (Left) */
.editor-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  border: 1px solid var(--border-color, #dee2e6);
  border-radius: 4px;
  background: var(--panel-bg-color, #f8f9fa);
  overflow: hidden;
}

.json5-editor {
  flex: 1;
  padding: 0.75rem;
  border: none;
  background: var(--input-bg-color, #fff);
  font-family: "Courier New", Courier, monospace;
  font-size: 0.9rem;
  line-height: 1.5;
  color: var(--text-color, #333);
  resize: none;
  outline: none;
  white-space: pre;
  overflow: auto;
  min-height: 400px;
}

.json5-editor:disabled {
  background: var(--disabled-bg-color, #f5f5f5);
  color: var(--text-muted-color, #6c757d);
  cursor: not-allowed;
}

.validation-error {
  padding: 0.75rem;
  background: var(--danger-bg-color, #f8d7da);
  color: var(--danger-color, #721c24);
  border-top: 1px solid var(--danger-border-color, #f5c6cb);
  font-size: 0.85rem;
  font-weight: 500;
  max-height: 100px;
  overflow-y: auto;
}

/* JSON Display Panel (Right) */
.json-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  border: 1px solid var(--border-color, #dee2e6);
  border-radius: 4px;
  background: var(--panel-bg-color, #f8f9fa);
  overflow: hidden;
}

.json-display {
  flex: 1;
  padding: 0.75rem;
  border: none;
  background: var(--input-bg-color, #fff);
  font-family: "Courier New", Courier, monospace;
  font-size: 0.9rem;
  line-height: 1.5;
  color: var(--text-color, #333);
  resize: none;
  outline: none;
  white-space: pre;
  overflow: auto;
  cursor: default;
  min-height: 400px;
}

/* Action Buttons */
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
  opacity: 0.4;
  cursor: not-allowed;
  background: var(--disabled-bg-color, #e9ecef) !important;
  color: var(--text-muted-color, #6c757d) !important;
  border-color: var(--border-color, #dee2e6) !important;
}

/* Responsive Design */
@media (max-width: 768px) {
  .split-panel {
    flex-direction: column;
    max-height: none;
  }

  .editor-panel {
    flex: 0 0 auto;
  }

  .json-panel {
    min-height: 300px;
  }
}
</style>
