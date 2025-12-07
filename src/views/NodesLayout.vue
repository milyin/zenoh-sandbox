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
            :titleLink="`/nodes/config/${index}`"
            @title-click="navigateToConfigEdit(index)"
          >
            <template #actions>
              <button
                @click="navigateToConfigEdit(index)"
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
                :titleLink="`/nodes/runtime/${runtimeId}`"
                @title-click="navigateToRuntimeLogs(runtimeId)"
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
                      <button @click="navigateToRuntimeConfig(runtimeId)">
                        Config
                      </button>
                      <button @click="navigateToRuntimeLogs(runtimeId)">
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

      <!-- Right Panel - Router View -->
      <div class="log-panel">
        <router-view
          :key="$route.fullPath"
          :configEntries="configEntries"
          :runtimeConfigs="runtimeConfigs"
          :activityLogs="activityLogs"
          :onClearActivityLogs="clearActivityLogs"
          :onUpdateConfig="updateConfig"
          @navigate-to-activity-log="navigateToActivityLog"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, provide } from 'vue';
import { useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';

// Import components
import Section from '../components/Section.vue';
import Entity from '../components/Entity.vue';

// Import ZenohConfig types and functions
import {
  type ZenohConfig,
  createZenohConfig,
  cleanupConfig
} from '../types/zenohConfig';

// Types for activity logs
interface ActivityLogEntry {
  timestamp: Date;
  type: "info" | "success" | "error" | "data";
  message: string;
  data?: Record<string, any>;
}

const router = useRouter();

const runtimes = ref<string[]>([]);
const configEntries = ref<ZenohConfig[]>([]);
const runtimeConfigs = reactive<Record<string, ZenohConfig>>({});

// Track which config index created which runtime
const runtimeToConfigIndex = reactive<Record<string, number>>({});

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

// Provide state to child components
provide('configEntries', configEntries);
provide('runtimeConfigs', runtimeConfigs);
provide('activityLogs', activityLogs);
provide('addActivityLog', addActivityLog);

const clearActivityLogs = () => {
  activityLogs.value = [];
};

// Get runtimes for a specific config index
const getRuntimesForConfig = (configIndex: number): string[] => {
  return runtimes.value.filter(runtimeId => runtimeToConfigIndex[runtimeId] === configIndex);
};

// Navigation methods
const navigateToActivityLog = () => {
  router.push('/nodes');
};

const navigateToConfigEdit = (index: number) => {
  router.push(`/nodes/config/${index}/edit`);
};

const navigateToRuntimeConfig = (runtimeId: string) => {
  router.push(`/nodes/runtime/${runtimeId}/config`);
};

const navigateToRuntimeLogs = (runtimeId: string) => {
  router.push(`/nodes/runtime/${runtimeId}/logs`);
};

// Config management methods
const updateConfig = (index: number, config: ZenohConfig) => {
  configEntries.value[index] = config;
};

const cloneConfig = async (index: number) => {
  const config = configEntries.value[index];
  const newConfig = await createZenohConfig(config.mode);
  configEntries.value.push(newConfig);
  addActivityLog('success', `Cloned ${config.mode} config`);
};

const removeConfig = (index: number) => {
  if (canRemoveConfig(index)) {
    const config = configEntries.value[index];
    cleanupConfig(config);
    configEntries.value.splice(index, 1);
    addActivityLog('info', `Removed ${config.mode} config`);
  }
};

const canRemoveConfig = (index: number): boolean => {
  // Can't remove if it has active runtimes
  return getRuntimesForConfig(index).length === 0;
};

const createRuntimeFromConfig = async (index: number) => {
  const configToUse = configEntries.value[index];

  try {
    // Create a new config for the next runtime (port assignment happens automatically)
    const nextConfig = await createZenohConfig(configToUse.mode);
    configEntries.value[index] = nextConfig;

    addActivityLog('info', `Starting runtime with ${configToUse.mode} config...`);

    // Small delay to avoid Tauri concurrency issues
    await new Promise(resolve => setTimeout(resolve, 100));

    try {
      const runtimeId = await invoke<string>('zenoh_runtime_start', { config: configToUse });
      addActivityLog('success', `Runtime started: ${runtimeId} on port ${configToUse.websocket_port}`);

      // Track which config created this runtime
      runtimeToConfigIndex[runtimeId] = index;

      // Add to runtimeConfigs to reserve the port
      runtimeConfigs[runtimeId] = { ...configToUse };

      // Clean up the config that was used to start the runtime
      cleanupConfig(configToUse);

      // Update runtimes list
      runtimes.value.push(runtimeId);
    } catch (error) {
      console.error('Failed to create runtime:', error);
      addActivityLog('error', `Failed to start runtime: ${error}`);

      // On error, restore the original config and cleanup the unused nextConfig
      cleanupConfig(nextConfig);
      configEntries.value[index] = configToUse;
    }
  } catch (error) {
    addActivityLog('error', `Failed to prepare runtime config: ${error}`);
  }
};

const stopRuntime = async (runtimeId: string) => {
  try {
    addActivityLog('info', `Stopping runtime ${runtimeId}...`);
    await invoke('zenoh_runtime_stop', { zid: runtimeId });

    // Remove from runtimes list
    const index = runtimes.value.indexOf(runtimeId);
    if (index > -1) {
      runtimes.value.splice(index, 1);
    }

    // Clean up config
    const config = runtimeConfigs[runtimeId];
    if (config) {
      cleanupConfig(config);
      delete runtimeConfigs[runtimeId];
    }

    // Clean up mapping
    delete runtimeToConfigIndex[runtimeId];

    addActivityLog('success', `Runtime ${runtimeId} stopped`);

    // Navigate to activity log if we were viewing this runtime
    const currentPath = router.currentRoute.value.path;
    if (currentPath.includes(runtimeId)) {
      navigateToActivityLog();
    }
  } catch (error) {
    addActivityLog('error', `Failed to stop runtime: ${error}`);
  }
};

// Initialize first config entry
const initializeFirstConfig = async () => {
  try {
    const defaultConfig = await createZenohConfig('peer');
    configEntries.value.push(defaultConfig);
    addActivityLog('info', 'Initialized with default peer config');
  } catch (error) {
    addActivityLog('error', `Failed to initialize config: ${error}`);
  }
};

// Initialize on mount
initializeFirstConfig();
</script>

<style scoped>
.zenoh-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
}

.main-panel {
  display: flex;
  flex: 1;
  overflow: hidden;
  gap: 0;
}

.entity-panel {
  flex: 0 0 40%;
  min-width: 350px;
  overflow-y: auto;
  border-right: 1px solid var(--border-color, #dee2e6);
}

.log-panel {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}

.runtime-info-panel {
  padding: 0.5rem;
}

.info-button-group {
  display: flex;
  gap: 0.5rem;
  justify-content: flex-start;
}

.info-button-group button {
  padding: 0.4rem 0.8rem;
  font-size: 0.85rem;
}
</style>
