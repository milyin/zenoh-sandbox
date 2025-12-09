<template>
  <Section title="Edit Config" icon="✏️" section-class="info-section">
    <div class="info-content">
      <div class="edit-container">
        <div class="button-group">
          <button @click="handleStart" class="action-button primary">Start</button>
          <button @click="handleClone" class="action-button">Clone</button>
          <button @click="handleRemove" class="action-button danger" :disabled="!canRemove">
            Remove
          </button>
        </div>

        <div class="split-panel">
          <!-- Left Panel: JSON5 Editor -->
          <div class="editor-panel">
            <div class="panel-header">
              <span class="panel-title">Edit (JSON5)</span>
              <span v-if="isValidating" class="status-badge validating">Validating...</span>
              <span v-else-if="!validationError" class="status-badge valid">Valid</span>
              <span v-else class="status-badge invalid">Invalid</span>
            </div>
            <textarea
              ref="json5Editor"
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

          <!-- Right Panel: Validated JSON -->
          <div class="json-panel">
            <div class="panel-header">
              <span class="panel-title">Validated Config (Read-Only)</span>
            </div>
            <div class="json-display-wrapper">
              <div class="json-highlight-overlay" ref="highlightOverlay">
                <div
                  v-for="line in jsonLines"
                  :key="line.lineNumber"
                  :class="['json-line-highlight', { changed: line.isChanged }]"
                >{{ line.content }}</div>
              </div>
              <textarea
                ref="jsonDisplay"
                v-model="validatedJsonString"
                class="json-display"
                readonly
                spellcheck="false"
                @scroll="handleJsonScroll"
              ></textarea>
            </div>
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
  validateConfigJson5,
  getDefaultConfigJson,
} from "../types/zenohConfig";
import { compareJsonStrings, type JsonLine } from "../utils/jsonComparison";

const {
  configEntries,
  cloneConfig,
  removeConfig,
  canRemoveConfig,
  getRuntimesForConfig,
  navigateToActivityLog,
  createRuntimeFromConfig,
  navigateToConfigEdit,
  updateConfig,
} = useNodesState();

const route = useRoute();
const configIndex = ref(parseInt(route.params.id as string));

const editContent = ref("");
const validatedJsonString = ref("");
const validationError = ref("");
const isValidating = ref(false);
const jsonLines = ref<JsonLine[]>([]);
const defaultConfigJson = ref("");

const json5Editor = ref<HTMLTextAreaElement | null>(null);
const jsonDisplay = ref<HTMLTextAreaElement | null>(null);
const highlightOverlay = ref<HTMLElement | null>(null);

let validationTimeout: ReturnType<typeof setTimeout> | null = null;
const VALIDATION_DEBOUNCE_MS = 500;

const hasActiveRuntimes = computed(() => {
  return getRuntimesForConfig(configIndex.value).length > 0;
});

const canRemove = computed(() => {
  return canRemoveConfig(configIndex.value);
});

const loadDefaultConfig = async () => {
  try {
    defaultConfigJson.value = await getDefaultConfigJson();
  } catch (error) {
    console.error("Failed to load default config:", error);
    defaultConfigJson.value = "{}";
  }
};

const initializeFromConfig = () => {
  const entry = configEntries.value[configIndex.value];
  if (!entry) return;
  editContent.value = entry.edit.content;
  validatedJsonString.value = JSON.stringify(entry.configJson, null, 2);
  updateHighlighting();
};

const updateHighlighting = () => {
  if (!defaultConfigJson.value) return;
  jsonLines.value = compareJsonStrings(
    defaultConfigJson.value,
    validatedJsonString.value
  );
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
    const validatedConfig = await validateConfigJson5(editContent.value);
    validatedJsonString.value = JSON.stringify(validatedConfig, null, 2);
    updateHighlighting();

    const newEdit: ZenohConfigEdit = { content: editContent.value };
    await updateConfig(configIndex.value, newEdit);

    validationError.value = "";
  } catch (error: any) {
    validationError.value = error.message || "Invalid JSON5 or configuration";
  }
};

const handleJsonScroll = () => {
  if (jsonDisplay.value && highlightOverlay.value) {
    highlightOverlay.value.scrollTop = jsonDisplay.value.scrollTop;
    highlightOverlay.value.scrollLeft = jsonDisplay.value.scrollLeft;
  }
};

const handleStart = async () => {
  if (validationError.value) {
    alert("Cannot start runtime with invalid configuration");
    return;
  }
  try {
    await createRuntimeFromConfig(configIndex.value);
  } catch (error: any) {
    alert(`Failed to start runtime: ${error.message || error}`);
    console.error('Start runtime error:', error);
  }
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

watch(
  () => configEntries.value[configIndex.value],
  (newEntry) => {
    if (newEntry && !isValidating.value) {
      initializeFromConfig();
    }
  },
  { deep: true }
);

onMounted(async () => {
  await loadDefaultConfig();
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

.json-display-wrapper {
  position: relative;
  flex: 1;
  background: var(--input-bg-color, #fff);
  overflow: hidden;
}

.json-highlight-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  padding: 0.75rem;
  pointer-events: none;
  z-index: 1;
  overflow: hidden;
  white-space: pre;
  font-family: "Courier New", Courier, monospace;
  font-size: 0.9rem;
  line-height: 1.5;
  color: transparent;
}

.json-line-highlight {
  white-space: pre;
  transition: background-color 0.2s;
}

.json-line-highlight.changed {
  background-color: var(--highlight-color, #fff3cd);
  border-left: 3px solid var(--warning-color, #ffc107);
  margin-left: -0.75rem;
  padding-left: calc(0.75rem - 3px);
}

.json-display {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  padding: 0.75rem;
  border: none;
  background: transparent;
  font-family: "Courier New", Courier, monospace;
  font-size: 0.9rem;
  line-height: 1.5;
  color: var(--text-color, #333);
  resize: none;
  outline: none;
  white-space: pre;
  overflow: auto;
  z-index: 2;
  cursor: default;
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
  opacity: 0.5;
  cursor: not-allowed;
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
