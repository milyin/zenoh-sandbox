<template>
  <div class="zenoh-container">
    <!-- Main Operations Panel -->
    <div class="main-panel">
      <!-- Entity Controls -->
      <div class="entity-panel">
        <!-- Local Section -->
        <Section
          title="Local"
          icon="⚙️"
          section-class="runtimes-section"
        >
          <!-- Config Entity -->
          <Entity
            title="Config"
            :descr="newRuntimeConfig.websocket_port || 'Configure and start'"
          >
            <template #actions>
              <button
                @click="showNewRuntimeEdit"
              >
                Edit
              </button>
              <button
                @click="createNewRuntime"
              >
                Start
              </button>
            </template>

            <!-- Active Runtimes as Sub-entities -->
            <template v-if="runtimes.length > 0" #sub-entities>
              <Entity
                v-for="runtimeId in runtimes"
                :key="runtimeId"
                :title="runtimeId"
                :descr="runtimeConfigs[runtimeId]?.websocket_port || 'Loading...'"
              >
                <template #actions>
                  <button
                    @click="cloneRuntime(runtimeId)"
                  >
                    Clone
                  </button>
                  <button
                    @click="stopRuntime(runtimeId)"
                  >
                    stop
                  </button>
                </template>

                <template #info>
                  <div class="runtime-info-panel">
                    <div class="info-button-group">
                      <button @click="showConfig(runtimeId)">
                        Config
                      </button>
                      <button @click="showLogs(runtimeId)">
                        Logs
                      </button>
                    </div>
                  </div>
                </template>
              </Entity>
            </template>
          </Entity>
        </Section>

        <!-- External Section -->
        <Section
          title="External"
          icon="🌐"
          section-class="external-section"
        >
        </Section>
      </div>

      <!-- Info/Logs/Config Panel -->
      <div class="log-panel">
        <Section
          :title="viewingConfigFor ? `Config - ${viewingConfigFor}` : viewingLogsFor ? `Logs - ${viewingLogsFor}` : editingNewRuntime ? 'Edit Config' : 'Runtime Info'"
          :icon="viewingConfigFor ? '⚙️' : viewingLogsFor ? '📜' : editingNewRuntime ? '✏️' : '📋'"
          section-class="info-section"
        >
          <template #actions v-if="editingNewRuntime">
            <button @click="editingNewRuntime = false">
              ✕ Close
            </button>
          </template>
          <template #actions v-else-if="viewingConfigFor">
            <button @click="refreshConfig" :disabled="isLoadingConfig">
              🔄 Refresh
            </button>
            <button @click="viewingConfigFor = null">
              ✕ Close
            </button>
          </template>
          <template #actions v-else-if="viewingLogsFor">
            <button @click="refreshLogs" :disabled="isLoadingLogs">
              🔄 Refresh
            </button>
            <button @click="loadPrevPage" :disabled="isLoadingLogs || logsPage === 0">
              ← Prev
            </button>
            <button @click="loadNextPage" :disabled="isLoadingLogs">
              Next →
            </button>
            <button @click="viewingLogsFor = null">
              ✕ Close
            </button>
          </template>

          <div class="info-content">
            <!-- Show new runtime edit dialog -->
            <div v-if="editingNewRuntime" class="edit-container">
              <ServerInput
                v-model="newRuntimeConfig.websocket_port!"
                label="WebSocket Port"
                placeholder="e.g., 10000 or 127.0.0.1:10000"
              />
            </div>

            <!-- Show config if viewing -->
            <div v-else-if="viewingConfigFor" class="config-container">
              <div v-if="isLoadingConfig" class="loading">
                Loading config...
              </div>
              <div v-else-if="!configJson" class="empty-config">
                No config available
              </div>
              <pre v-else class="config-json">{{ configJson }}</pre>
            </div>

            <!-- Show logs if viewing -->
            <div v-else-if="viewingLogsFor" class="logs-container">
              <div v-if="isLoadingLogs" class="loading">
                Loading logs...
              </div>
              <div v-else-if="logs.length === 0" class="empty-logs">
                No logs available (page {{ logsPage }})
              </div>
              <div v-else class="log-entries">
                <div v-for="(log, index) in logs" :key="index" class="log-entry" :class="`log-${log.level.toLowerCase()}`">
                  <span class="log-timestamp">{{ new Date(log.timestamp).toLocaleTimeString() }}</span>
                  <span class="log-level">{{ log.level }}</span>
                  <span class="log-target">{{ log.target }}</span>
                  <span class="log-message">{{ log.message }}</span>
                </div>
              </div>
              <div class="logs-footer">
                Page {{ logsPage }} • {{ logs.length }} entries
              </div>
            </div>

            <!-- Show runtime info if not viewing logs or config -->
            <div v-else>
              <div v-if="runtimes.length === 0" class="empty-info">
                No runtimes running. Create a new runtime to get started.
              </div>
              <div v-else class="runtime-summary">
                <p><strong>Active Runtimes:</strong> {{ runtimes.length }}</p>
                <ul>
                  <li v-for="runtimeId in runtimes" :key="runtimeId">
                    <strong>{{ runtimeId }}</strong>:
                    {{ runtimeConfigs[runtimeId]?.websocket_port || 'Loading...' }}
                  </li>
                </ul>
              </div>
            </div>
          </div>
        </Section>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, defineExpose } from 'vue';
import { invoke } from '@tauri-apps/api/core';

// Import components
import Section from './components/Section.vue';
import Entity from './components/Entity.vue';
import ServerInput from './components/ServerInput.vue';

// Import ZenohConfig types and functions
import {
  ZenohConfig,
  createDefaultZenohConfig,
  nextZenohConfig
} from './types/zenohConfig';

// Constants
const NEW_INSTANCE_ID = '__NEW_INSTANCE__';

// Types for logs
interface LogEntry {
  timestamp: string;
  level: string;
  target: string;
  message: string;
}

const runtimes = ref<string[]>([]);
const selectedRuntime = ref<string | null>(NEW_INSTANCE_ID);
const newRuntimeConfig = ref<ZenohConfig>({ websocket_port: null });
const runtimeConfigs = reactive<Record<string, ZenohConfig>>({});
const runtimeEditsExpanded = reactive<Record<string, boolean>>({});

// New runtime edit state
const editingNewRuntime = ref(false);

// Log viewing state
const viewingLogsFor = ref<string | null>(null);
const logs = ref<LogEntry[]>([]);
const logsPage = ref(0);
const isLoadingLogs = ref(false);

// Config viewing state
const viewingConfigFor = ref<string | null>(null);
const configJson = ref<string | null>(null);
const isLoadingConfig = ref(false);

// Set default configuration for new runtime
const setDefaultConfig = async () => {
  const defaultConfig = createDefaultZenohConfig();
  newRuntimeConfig.value = await nextZenohConfig(defaultConfig);
};

// Load runtimes on mount
const loadRuntimes = async () => {
  try {
    const list = await invoke<string[]>('zenoh_runtime_list');
    runtimes.value = list;

    // Load configs for all runtimes
    for (const runtimeId of list) {
      await loadConfig(runtimeId);
    }
  } catch (error) {
    console.error('Failed to load runtimes:', error);
  }
};

// Load configuration for a specific runtime
const loadConfig = async (runtimeId: string) => {
  try {
    const config = await invoke<ZenohConfig>('zenoh_runtime_config', { zid: runtimeId });
    runtimeConfigs[runtimeId] = { ...config };
  } catch (error) {
    console.error('Failed to load config:', error);
  }
};

// Clone runtime - copy config to new runtime and open edits
const cloneRuntime = (runtimeId: string) => {
  newRuntimeConfig.value = { ...runtimeConfigs[runtimeId] };
  editingNewRuntime.value = true;
};

// Show new runtime edit panel
const showNewRuntimeEdit = () => {
  // Close other panels
  viewingLogsFor.value = null;
  viewingConfigFor.value = null;
  editingNewRuntime.value = true;
};

// Load config JSON for a runtime
const loadConfigJson = async (runtimeId: string) => {
  isLoadingConfig.value = true;
  try {
    const json = await invoke<string>('zenoh_runtime_config_json', { zid: runtimeId });
    configJson.value = json;
  } catch (error) {
    console.error('Failed to load config JSON:', error);
    configJson.value = null;
  } finally {
    isLoadingConfig.value = false;
  }
};

// Show config for a runtime
const showConfig = async (runtimeId: string) => {
  // Close other panels
  viewingLogsFor.value = null;
  editingNewRuntime.value = false;
  viewingConfigFor.value = runtimeId;
  await loadConfigJson(runtimeId);
};

// Refresh current config
const refreshConfig = async () => {
  if (viewingConfigFor.value) {
    await loadConfigJson(viewingConfigFor.value);
  }
};

// Load logs for a runtime
const loadLogs = async (runtimeId: string, page: number = 0) => {
  isLoadingLogs.value = true;
  try {
    const fetchedLogs = await invoke<LogEntry[]>('zenoh_runtime_log', {
      zid: runtimeId,
      page
    });
    logs.value = fetchedLogs;
    logsPage.value = page;
  } catch (error) {
    console.error('Failed to load logs:', error);
    logs.value = [];
  } finally {
    isLoadingLogs.value = false;
  }
};

// Show logs for a runtime
const showLogs = async (runtimeId: string) => {
  // Close other panels
  viewingConfigFor.value = null;
  editingNewRuntime.value = false;
  viewingLogsFor.value = runtimeId;
  await loadLogs(runtimeId, 0);
};

// Refresh current logs
const refreshLogs = async () => {
  if (viewingLogsFor.value) {
    await loadLogs(viewingLogsFor.value, logsPage.value);
  }
};

// Load next/previous page
const loadNextPage = async () => {
  if (viewingLogsFor.value) {
    await loadLogs(viewingLogsFor.value, logsPage.value + 1);
  }
};

const loadPrevPage = async () => {
  if (viewingLogsFor.value && logsPage.value > 0) {
    await loadLogs(viewingLogsFor.value, logsPage.value - 1);
  }
};

// Create new runtime with configured settings
const createNewRuntime = async () => {
  console.log('🚀 Starting runtime with config:', newRuntimeConfig.value);
  try {
    // Use the exact config from the dialog (user's responsibility to enter correct values)
    console.log('📞 Calling zenoh_runtime_start...');
    const newRuntimeId = await invoke<string>('zenoh_runtime_start', { config: newRuntimeConfig.value });
    console.log('✅ Runtime started successfully:', newRuntimeId);
    await loadRuntimes();

    // After successful creation, prepare next free port for next invocation
    newRuntimeConfig.value = await nextZenohConfig(newRuntimeConfig.value);

    // Optionally select the new runtime
    selectedRuntime.value = newRuntimeId;
  } catch (error) {
    console.error('❌ Failed to create runtime:', error);
    alert(`Failed to create runtime: ${error}`);
  }
};

// stop (delete) an runtime
const stopRuntime = async (runtimeId: string) => {
  try {
    await invoke('zenoh_runtime_stop', { zid: runtimeId });
    if (selectedRuntime.value === runtimeId) {
      selectedRuntime.value = NEW_INSTANCE_ID;
    }
    delete runtimeConfigs[runtimeId];
    delete runtimeEditsExpanded[runtimeId];
    await loadRuntimes();
  } catch (error) {
    console.error('Failed to stop runtime:', error);
    alert(`Failed to stop runtime: ${error}`);
  }
};

// Expose selectedRuntime for parent component to persist
defineExpose({
  selectedRuntime
});

// Initial load - load runtimes first, then set default config
(async () => {
  await loadRuntimes();
  await setDefaultConfig();
})();
</script>

<style scoped>
/* Essential 2-panel layout styles */
.zenoh-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
}

.main-panel {
  display: flex;
  flex: 1;
  overflow: hidden;
  gap: 0.5rem;
}

.entity-panel {
  width: 40%;
  overflow-y: auto;
}

.log-panel {
  width: 60%;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.info-section {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
}

.info-content {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
  padding: var(--spacing-content);
}

.empty-info {
  text-align: center;
  color: var(--log-neutral-color);
  padding: 2rem;
}

.runtime-summary {
  font-size: var(--font-size-normal);
}

.runtime-summary ul {
  margin-top: 1rem;
  margin-left: 1.5rem;
}

.runtime-summary li {
  margin-bottom: 0.5rem;
  font-family: monospace;
}

.runtime-info-panel {
  display: flex;
  flex-direction: column;
  gap: var(--size-md);
}

.info-button-group {
  display: flex;
  gap: var(--size-md);
  flex-wrap: wrap;
}

.info-button-group button {
  flex: 1;
  min-width: calc(var(--base-font-size) * 5);
}

/* Logs styling */
.logs-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  font-family: monospace;
  font-size: 0.85rem;
}

.loading,
.empty-logs {
  text-align: center;
  color: var(--log-neutral-color);
  padding: 2rem;
}

.log-entries {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.log-entry {
  display: grid;
  grid-template-columns: auto auto 1fr 2fr;
  gap: 0.5rem;
  padding: 0.25rem 0.5rem;
  border-left: 3px solid transparent;
  font-size: 0.8rem;
  word-break: break-word;
}

.log-entry:hover {
  background: var(--hover-bg-color, #f0f0f0);
}

.log-timestamp {
  color: var(--log-neutral-color, #666);
  white-space: nowrap;
}

.log-level {
  font-weight: bold;
  white-space: nowrap;
}

.log-target {
  color: var(--log-neutral-color, #666);
  overflow: hidden;
  text-overflow: ellipsis;
}

.log-message {
  word-break: break-word;
}

/* Log level colors */
.log-trace { border-left-color: #999; }
.log-trace .log-level { color: #999; }

.log-debug { border-left-color: #0d6efd; }
.log-debug .log-level { color: #0d6efd; }

.log-info { border-left-color: #198754; }
.log-info .log-level { color: #198754; }

.log-warn { border-left-color: #ffc107; }
.log-warn .log-level { color: #ffc107; }

.log-error { border-left-color: #dc3545; }
.log-error .log-level { color: #dc3545; }

.logs-footer {
  padding: 0.5rem;
  border-top: 1px solid var(--border-color, #dee2e6);
  text-align: center;
  font-size: 0.75rem;
  color: var(--log-neutral-color, #666);
}

/* Edit panel styling */
.edit-container {
  padding: 1rem;
}

/* Config panel styling */
.config-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  font-family: monospace;
  font-size: 0.85rem;
}

.empty-config {
  text-align: center;
  color: var(--log-neutral-color, #666);
  padding: 2rem;
}

.config-json {
  flex: 1;
  overflow: auto;
  margin: 0;
  padding: 1rem;
  background: var(--code-bg-color, #f8f9fa);
  border-radius: 4px;
  white-space: pre-wrap;
  word-break: break-word;
  font-size: 0.8rem;
  line-height: 1.4;
}
</style>
