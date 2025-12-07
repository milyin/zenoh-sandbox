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
          <!-- Config Entities -->
          <Entity
            v-for="(config, index) in configEntries"
            :key="index"
            title="Config"
            :descr="`Mode: ${config.mode}`"
          >
            <template #actions>
              <button
                @click="showConfigEdit(index)"
              >
                Edit
              </button>
              <button
                @click="cloneConfig(index)"
              >
                Clone
              </button>
              <button
                @click="removeConfig(index)"
                :disabled="!canRemoveConfig(index)"
              >
                Remove
              </button>
              <button
                @click="createRuntimeFromConfig(index)"
              >
                Start
              </button>
            </template>

            <!-- Active Runtimes for this Config as Sub-entities -->
            <template v-if="getRuntimesForConfig(index).length > 0" #sub-entities>
              <Entity
                v-for="runtimeId in getRuntimesForConfig(index)"
                :key="runtimeId"
                :title="runtimeId"
                :descr="`Port: ${runtimeConfigs[runtimeId]?.websocket_port || 'Loading...'}`"
              >
                <template #actions>
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

      <!-- Info/Config/Activity Panel -->
      <div class="log-panel">
        <!-- Show config edit dialog -->
        <Section v-if="editingConfigIndex !== null" title="Edit Config" icon="✏️" section-class="info-section">
          <template #actions>
            <button @click="editingConfigIndex = null">
              ✕ Close
            </button>
          </template>
          <div class="info-content">
            <div class="edit-container">
              <label class="mode-selector-label">
                <span>Zenoh Mode:</span>
                <select v-model="configEntries[editingConfigIndex].mode" class="mode-selector">
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

        <!-- Show config viewer -->
        <Section v-else-if="viewingConfigFor" :title="`Config - ${viewingConfigFor}`" icon="⚙️" section-class="info-section">
          <template #actions>
            <button @click="refreshConfig" :disabled="isLoadingConfig">
              🔄 Refresh
            </button>
            <button @click="viewingConfigFor = null">
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

        <!-- Show runtime logs viewer -->
        <LogPanel
          v-else-if="viewingLogsFor"
          :title="`Runtime Logs - ${viewingLogsFor}`"
          icon="📜"
          :logs="runtimeLogs"
          :onLoadMore="hasMoreRuntimeLogs ? loadMoreRuntimeLogs : undefined"
          :onClear="clearRuntimeLogs"
          :showClearButton="true"
        >
          <template #actions>
            <button @click="viewingLogsFor = null">
              ✕ Close
            </button>
          </template>
        </LogPanel>

        <!-- Default: Show Activity Log using universal LogPanel -->
        <LogPanel
          v-else
          title="Activity Log"
          icon="📜"
          :logs="activityLogs"
          :onClear="clearActivityLogs"
          :showClearButton="true"
        />
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
import LogPanel from './components/LogPanel.vue';

// Import ZenohConfig types and functions
import {
  type ZenohConfig,
  createZenohConfig,
  cleanupConfig
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

// Types for activity logs
interface ActivityLogEntry {
  timestamp: Date;
  type: "info" | "success" | "error" | "data";
  message: string;
  data?: Record<string, any>;
}

const runtimes = ref<string[]>([]);
const selectedRuntime = ref<string | null>(NEW_INSTANCE_ID);
const configEntries = ref<ZenohConfig[]>([]);
const runtimeConfigs = reactive<Record<string, ZenohConfig>>({});
const runtimeEditsExpanded = reactive<Record<string, boolean>>({});

// Track which config index created which runtime
const runtimeToConfigIndex = reactive<Record<string, number>>({});

// Config edit state
const editingConfigIndex = ref<number | null>(null);

// Activity log state
const activityLogs = ref<ActivityLogEntry[]>([]);

const addActivityLog = (type: ActivityLogEntry['type'], message: string, data?: Record<string, any>) => {
  activityLogs.value.unshift({
    timestamp: new Date(),
    type,
    message,
    data
  });
  // Limit to 500 entries
  if (activityLogs.value.length > 500) {
    activityLogs.value.splice(500);
  }
};

const clearActivityLogs = () => {
  activityLogs.value = [];
};

// Runtime log viewing state
const viewingLogsFor = ref<string | null>(null);
const runtimeLogs = ref<LogEntry[]>([]);
const runtimeLogsPage = ref(0);
const isLoadingRuntimeLogs = ref(false);
const hasMoreRuntimeLogs = ref(true);

// Config viewing state
const viewingConfigFor = ref<string | null>(null);
const configJson = ref<string | null>(null);
const isLoadingConfig = ref(false);

// Get runtimes for a specific config index
const getRuntimesForConfig = (configIndex: number): string[] => {
  return runtimes.value.filter(runtimeId => runtimeToConfigIndex[runtimeId] === configIndex);
};

// Initialize first config entry with a free port
const initializeFirstConfig = async () => {
  const config = await createZenohConfig("peer");
  configEntries.value[0] = config;
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

// Clone a config entry (creates a new config with the same mode but a free port)
const cloneConfig = async (index: number) => {
  const originalConfig = configEntries.value[index];
  const clonedConfig = await createZenohConfig(originalConfig.mode);
  configEntries.value.push(clonedConfig);
};

// Remove a config entry (only if it's not the last one and has no runtimes)
const removeConfig = (index: number) => {
  if (canRemoveConfig(index)) {
    configEntries.value.splice(index, 1);

    // Update runtime-to-config mappings for configs after the removed one
    for (const runtimeId in runtimeToConfigIndex) {
      if (runtimeToConfigIndex[runtimeId] > index) {
        runtimeToConfigIndex[runtimeId]--;
      }
    }

    // If we were editing this config, close the edit panel
    if (editingConfigIndex.value === index) {
      editingConfigIndex.value = null;
    } else if (editingConfigIndex.value !== null && editingConfigIndex.value > index) {
      // Adjust the editing index if needed
      editingConfigIndex.value--;
    }
  }
};

// Check if a config entry can be removed
const canRemoveConfig = (index: number): boolean => {
  // Can't remove the last config
  if (configEntries.value.length <= 1) {
    return false;
  }
  // Can't remove a config that has active runtimes
  const configRuntimes = getRuntimesForConfig(index);
  return configRuntimes.length === 0;
};

// Show config edit panel
const showConfigEdit = (index: number) => {
  // Close other panels
  viewingLogsFor.value = null;
  viewingConfigFor.value = null;
  editingConfigIndex.value = index;
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
  editingConfigIndex.value = null;
  viewingConfigFor.value = runtimeId;
  await loadConfigJson(runtimeId);
};

// Refresh current config
const refreshConfig = async () => {
  if (viewingConfigFor.value) {
    await loadConfigJson(viewingConfigFor.value);
  }
};

// Load runtime logs
const loadRuntimeLogs = async (runtimeId: string, page: number = 0) => {
  isLoadingRuntimeLogs.value = true;
  try {
    const fetchedLogs = await invoke<LogEntry[]>('zenoh_runtime_log', {
      zid: runtimeId,
      page
    });

    if (page === 0) {
      // First page, replace logs
      runtimeLogs.value = fetchedLogs;
    } else {
      // Subsequent pages, append logs
      runtimeLogs.value = [...runtimeLogs.value, ...fetchedLogs];
    }

    runtimeLogsPage.value = page;
    hasMoreRuntimeLogs.value = fetchedLogs.length === 100; // Assuming page size is 100
  } catch (error) {
    console.error('Failed to load runtime logs:', error);
    addActivityLog('error', `Failed to load logs for ${runtimeId}`, { error: String(error) });
    runtimeLogs.value = [];
  } finally {
    isLoadingRuntimeLogs.value = false;
  }
};

// Show logs for a runtime
const showLogs = async (runtimeId: string) => {
  // Close other panels
  editingConfigIndex.value = null;
  viewingConfigFor.value = null;
  viewingLogsFor.value = runtimeId;
  runtimeLogsPage.value = 0;
  hasMoreRuntimeLogs.value = true;
  await loadRuntimeLogs(runtimeId, 0);
};

// Load more runtime logs (pagination callback)
const loadMoreRuntimeLogs = async () => {
  if (viewingLogsFor.value) {
    await loadRuntimeLogs(viewingLogsFor.value, runtimeLogsPage.value + 1);
  }
  return [];
};

// Clear runtime logs
const clearRuntimeLogs = () => {
  runtimeLogs.value = [];
  runtimeLogsPage.value = 0;
  hasMoreRuntimeLogs.value = true;
};

// Track which configs are currently starting a runtime (prevent double-click)
const startingConfigs = new Set<ZenohConfig>();

// Create new runtime from a config entry
const createRuntimeFromConfig = async (index: number) => {
  const configToUse = configEntries.value[index];

  // Prevent double-clicking on the same config object
  if (startingConfigs.has(configToUse)) {
    console.warn('⚠️ Runtime is already starting for this config');
    return;
  }

  startingConfigs.add(configToUse);

  try {
    // Create a new config for the next runtime (port assignment happens automatically)
    const nextConfig = await createZenohConfig(configToUse.mode);
    configEntries.value[index] = nextConfig;

    console.log('🚀 Starting runtime with config:', configToUse);
    addActivityLog('info', `Starting runtime with config`, { config: configToUse });

    try {
      console.log('📞 Calling zenoh_runtime_start...');
      const newRuntimeId = await invoke<string>('zenoh_runtime_start', { config: configToUse });
      console.log('✅ Runtime started successfully:', newRuntimeId);
      addActivityLog('success', `Runtime started: ${newRuntimeId} on port ${configToUse.websocket_port}`);

      // Track which config created this runtime
      runtimeToConfigIndex[newRuntimeId] = index;

      await loadRuntimes();

      // No manual cleanup needed - configToUse will be automatically cleaned up
      // when we create the next config (cleanupRunningConfigs() finds it in runtime list)

      // Optionally select the new runtime
      selectedRuntime.value = newRuntimeId;
    } catch (error) {
      console.error('❌ Failed to create runtime:', error);
      addActivityLog('error', `Failed to start runtime`, { error: String(error) });
      alert(`Failed to create runtime: ${error}`);

      // On error, restore the original config and cleanup the unused nextConfig
      cleanupConfig(nextConfig);
      configEntries.value[index] = configToUse;
    }
  } finally {
    // Always clear the starting flag for this specific config object
    startingConfigs.delete(configToUse);
  }
};

// stop (delete) an runtime
const stopRuntime = async (runtimeId: string) => {
  addActivityLog('info', `Stopping runtime: ${runtimeId}`);
  try {
    await invoke('zenoh_runtime_stop', { zid: runtimeId });
    addActivityLog('success', `Runtime stopped: ${runtimeId}`);

    if (selectedRuntime.value === runtimeId) {
      selectedRuntime.value = NEW_INSTANCE_ID;
    }
    delete runtimeConfigs[runtimeId];
    delete runtimeEditsExpanded[runtimeId];
    delete runtimeToConfigIndex[runtimeId];
    await loadRuntimes();
  } catch (error) {
    console.error('Failed to stop runtime:', error);
    addActivityLog('error', `Failed to stop runtime: ${runtimeId}`, { error: String(error) });
    alert(`Failed to stop runtime: ${error}`);
  }
};

// Expose selectedRuntime for parent component to persist
defineExpose({
  selectedRuntime
});

// Initial load - load runtimes first, then initialize first config
(async () => {
  await loadRuntimes();
  await initializeFirstConfig();
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

.mode-selector-label {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  font-size: var(--font-size-normal);
}

.mode-selector-label span {
  font-weight: 500;
}

.mode-selector {
  padding: 0.5rem;
  font-size: var(--font-size-normal);
  border: 1px solid var(--border-color, #dee2e6);
  border-radius: 4px;
  background: var(--input-bg-color, white);
  cursor: pointer;
}

.mode-selector:focus {
  outline: 2px solid var(--primary-color, #0d6efd);
  outline-offset: 2px;
}

.mode-description {
  margin-top: 1rem;
  color: var(--log-neutral-color, #666);
  font-size: 0.9rem;
  font-style: italic;
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
