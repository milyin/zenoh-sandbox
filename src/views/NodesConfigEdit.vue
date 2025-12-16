<template>
  <Section id="edit" title="Edit Config" icon="✏️" section-class="info-section" background-color="#ffe0e8">
    <div class="info-content">
      <div class="edit-container">
        <div class="button-group">
          <button @click="handleStart" class="primary" :disabled="!!validationError">Start</button>
          <button @click="handleClone">Clone</button>
          <button @click="handleReset" :disabled="hasActiveRuntimes">Reset</button>
          <button @click="handleOpen" :disabled="hasActiveRuntimes">Open</button>
          <button @click="handleRemove" class="danger" :disabled="!canRemove">
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
import Section from "../components/Section.vue";
import { useNodesState } from "../composables/useNodesState";
import {
  type ZenohConfigEdit,
  validateConfig,
  getDefaultConfigJson,
} from "../types/zenohConfig";

interface Props {
  id: number;
}

const props = defineProps<Props>();

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

const configId = computed(() => props.id);

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
    validationError.value = "";

    const newEdit: ZenohConfigEdit = { content: editContent.value };
    const success = await updateConfig(configId.value, newEdit);

    if (success) {
      // Update diff after successful validation
      updateDiff();
    } else {
      validationError.value = "Failed to update config";
    }
  } catch (error: any) {
    validationError.value = error || "Invalid JSON5 or configuration";

    // Update config entry to set validation error flag
    const newEdit: ZenohConfigEdit = { content: editContent.value };
    await updateConfig(configId.value, newEdit);
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
/* View-specific styles only */
.info-content {
  padding: var(--size-xl);
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.edit-container {
  display: flex;
  flex-direction: column;
  gap: var(--size-xl);
  flex: 1;
  min-height: 0;
}

/* View-specific override for split panel */
.split-panel {
  flex: 1;
  min-height: 0;
}

/* JSON5 Editor - specific to this view */
.json5-editor {
  flex: 1;
  padding: var(--size-xl);
  border: none;
  background: var(--input-bg-color, #fff);
  font-family: "Courier New", Courier, monospace;
  font-size: var(--font-size-small);
  line-height: 1.5;
  color: var(--text-color, #333);
  resize: none;
  outline: none;
  white-space: pre;
  overflow: auto;
  min-height: 0;
}

.json5-editor:disabled {
  background: var(--disabled-bg-color, #f5f5f5);
  color: var(--text-muted-color, #6c757d);
  cursor: not-allowed;
}

/* Validation error display - specific to this view */
.validation-error {
  padding: var(--size-xl);
  background: var(--danger-bg-color, #f8d7da);
  color: var(--danger-color, #721c24);
  border-top: 1px solid var(--danger-border-color, #f5c6cb);
  font-size: var(--font-size-small);
  font-weight: 500;
  max-height: 100px;
  overflow-y: auto;
}
</style>
