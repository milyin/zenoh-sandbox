<template>
  <Section title="Edit Config" icon="✏️" section-class="info-section">
    <div class="info-content">
      <div class="edit-container">
        <!-- Split Panel: Controls on Left, JSON on Right -->
        <div class="split-panel">
          <!-- Left Panel: Controls -->
          <div class="controls-panel">
            <h3 class="panel-title">Controls</h3>
            <div class="controls-content">
              <label class="mode-selector-label">
                <span>Zenoh Mode:</span>
                <select
                  v-model="localEdit.mode"
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
          </div>

          <!-- Right Panel: JSON Editor with Highlighting -->
          <div class="json-panel">
            <h3 class="panel-title">JSON Configuration</h3>
            <div class="json-editor-container">
              <div
                v-for="line in jsonLines"
                :key="line.lineNumber"
                :class="['json-line', { changed: line.isChanged }]"
              >
                <span class="line-number">{{ line.lineNumber }}</span>
                <pre class="line-content">{{ line.content }}</pre>
              </div>
              <textarea
                v-model="jsonString"
                class="json-editor-overlay"
                :disabled="hasActiveRuntimes"
                spellcheck="false"
                @input="handleJsonInput"
              ></textarea>
            </div>
            <div v-if="jsonError" class="json-error">{{ jsonError }}</div>
          </div>
        </div>

        <!-- Action Buttons -->
        <div class="button-group">
          <button @click="handleStart" class="action-button primary">
            Start
          </button>
          <button @click="handleClone" class="action-button">Clone</button>
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
import { ref, watch, computed, onMounted } from "vue";
import { useRoute } from "vue-router";
import Section from "../components/Section.vue";
import { useNodesState } from "../composables/useNodesState";
import {
  ZenohConfig,
  type ZenohConfigEdit,
  verifyZenohConfigJson,
  applyZenohConfigEdit,
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
} = useNodesState();

const route = useRoute();
const configIndex = ref(parseInt(route.params.id as string));

// Local state for editing
const localEdit = ref<ZenohConfigEdit>({
  mode: configEntries.value[configIndex.value].edit.mode,
});

const jsonString = ref("");
const jsonError = ref("");
const jsonLines = ref<JsonLine[]>([]);
const previousJsonString = ref("");
let isUpdatingFromControls = false;
let isUpdatingFromJson = false;

// Computed properties
const hasActiveRuntimes = computed(() => {
  return getRuntimesForConfig(configIndex.value).length > 0;
});

const canRemove = computed(() => {
  return canRemoveConfig(configIndex.value);
});

// Initialize JSON string and lines from config
const updateJsonFromConfig = async () => {
  try {
    const entry = configEntries.value[configIndex.value];
    const newJsonString = JSON.stringify(entry.configJson, null, 2);

    // Compare with previous to highlight changes
    if (previousJsonString.value) {
      jsonLines.value = compareJsonStrings(previousJsonString.value, newJsonString);
    } else {
      // First time - no changes
      jsonLines.value = newJsonString.split('\n').map((content, index) => ({
        lineNumber: index + 1,
        content,
        isChanged: false,
      }));
    }

    jsonString.value = newJsonString;
    previousJsonString.value = newJsonString;
  } catch (error) {
    console.error("Failed to get JSON:", error);
    jsonString.value = "{}";
    jsonLines.value = [{
      lineNumber: 1,
      content: "{}",
      isChanged: false,
    }];
  }
};

// Watch for config changes from external sources
watch(
  () => configEntries.value[configIndex.value],
  (newEntry) => {
    if (newEntry && !isUpdatingFromJson && !isUpdatingFromControls) {
      localEdit.value = { ...newEntry.edit };
      updateJsonFromConfig();
    }
  },
  { deep: true }
);

// Handlers
const handleModeChange = async () => {
  if (!hasActiveRuntimes.value) {
    isUpdatingFromControls = true;
    try {
      // Apply edit to config
      const entry = configEntries.value[configIndex.value];
      const newConfigJson = await applyZenohConfigEdit(entry.configJson, localEdit.value);

      // Update state
      configEntries.value[configIndex.value] = new ZenohConfig(localEdit.value, newConfigJson);

      // Update JSON display with highlighting
      await updateJsonFromConfig();
    } catch (error: any) {
      console.error("Failed to update config:", error);
      jsonError.value = error.message || "Failed to update configuration";
      // Revert on error
      localEdit.value = { ...configEntries.value[configIndex.value].edit };
    } finally {
      isUpdatingFromControls = false;
    }
  }
};

const handleJsonInput = async () => {
  if (hasActiveRuntimes.value) return;

  jsonError.value = "";
  isUpdatingFromJson = true;

  try {
    // Parse JSON
    const parsedJson = JSON.parse(jsonString.value);

    // Verify through Rust
    const [edit, configJson] = await verifyZenohConfigJson(parsedJson);

    // Update state with new verified config
    configEntries.value[configIndex.value] = new ZenohConfig(edit, configJson);
    localEdit.value = edit;

    // Update highlighting
    const newJsonString = JSON.stringify(configJson, null, 2);
    jsonLines.value = compareJsonStrings(previousJsonString.value, newJsonString);
    previousJsonString.value = newJsonString;

  } catch (error: any) {
    // Show error but don't revert - allow user to continue editing
    jsonError.value = error.message || "Invalid JSON or configuration";

    // Still update the visual highlighting to show current state
    jsonLines.value = jsonString.value.split('\n').map((content, index) => ({
      lineNumber: index + 1,
      content,
      isChanged: true, // Mark all as changed when invalid
    }));
  } finally {
    isUpdatingFromJson = false;
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

// Initialize JSON on mount
onMounted(() => {
  updateJsonFromConfig();
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

/* Split Panel Layout */
.split-panel {
  display: flex;
  gap: 1rem;
  min-height: 400px;
}

.controls-panel {
  flex: 0 0 300px;
  display: flex;
  flex-direction: column;
  border: 1px solid var(--border-color, #dee2e6);
  border-radius: 4px;
  background: var(--panel-bg-color, #f8f9fa);
  padding: 1rem;
}

.json-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  border: 1px solid var(--border-color, #dee2e6);
  border-radius: 4px;
  background: var(--panel-bg-color, #f8f9fa);
  padding: 1rem;
}

.panel-title {
  margin: 0 0 1rem 0;
  font-size: 1rem;
  font-weight: 600;
  color: var(--text-color, #333);
  border-bottom: 2px solid var(--primary-color, #007bff);
  padding-bottom: 0.5rem;
}

/* Controls Panel */
.controls-content {
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

/* JSON Editor with Highlighting */
.json-editor-container {
  position: relative;
  flex: 1;
  background: var(--input-bg-color, #fff);
  border: 1px solid var(--border-color, #dee2e6);
  border-radius: 4px;
  overflow: auto;
  font-family: "Courier New", Courier, monospace;
  font-size: 0.9rem;
  line-height: 1.5;
}

.json-line {
  display: flex;
  padding: 0 0.5rem;
  white-space: pre;
  transition: background-color 0.2s;
}

.json-line.changed {
  background-color: var(--highlight-color, #fff3cd);
  border-left: 3px solid var(--warning-color, #ffc107);
}

.line-number {
  user-select: none;
  color: var(--text-muted-color, #6c757d);
  min-width: 3em;
  text-align: right;
  padding-right: 1em;
  border-right: 1px solid var(--border-color, #dee2e6);
}

.line-content {
  margin: 0;
  padding-left: 1em;
  flex: 1;
  font-family: inherit;
  font-size: inherit;
  line-height: inherit;
  white-space: pre;
}

.json-editor-overlay {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  padding: 0.5rem;
  padding-left: calc(3em + 1em + 1em + 0.5rem);
  border: none;
  background: transparent;
  font-family: "Courier New", Courier, monospace;
  font-size: 0.9rem;
  line-height: 1.5;
  color: var(--text-color, #333);
  resize: none;
  outline: none;
  caret-color: var(--text-color, #333);
}

.json-editor-overlay:disabled {
  cursor: not-allowed;
}

.json-error {
  margin-top: 0.5rem;
  padding: 0.5rem;
  color: var(--danger-color, #dc3545);
  background-color: var(--danger-bg-color, #f8d7da);
  border: 1px solid var(--danger-border-color, #f5c6cb);
  border-radius: 4px;
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

/* Responsive Design */
@media (max-width: 768px) {
  .split-panel {
    flex-direction: column;
  }

  .controls-panel {
    flex: 1;
  }
}
</style>
