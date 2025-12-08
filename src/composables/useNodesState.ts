import { ref, reactive } from 'vue';
import { useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import {
  type ZenohConfigEdit,
  ZenohConfig,
  createZenohConfig,
  applyZenohConfigEdit,
} from '../types/zenohConfig';

interface ActivityLogEntry {
  timestamp: Date;
  type: "info" | "success" | "error" | "data";
  message: string;
  data?: Record<string, any>;
}

// Singleton state - shared across all instances
const runtimes = ref<string[]>([]);
const configEntries = ref<ZenohConfig[]>([]);
const runtimeConfigs = reactive<Record<string, ZenohConfig>>({});
const runtimeToConfigIndex = reactive<Record<string, number>>({});
const activityLogs = ref<ActivityLogEntry[]>([]);
let initialized = false;

export function useNodesState() {
  const router = useRouter();

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

  const getRuntimesForConfig = (configIndex: number): string[] => {
    return runtimes.value.filter(runtimeId => runtimeToConfigIndex[runtimeId] === configIndex);
  };

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

  const updateConfig = async (index: number, edit: ZenohConfigEdit) => {
    const entry = configEntries.value[index];

    try {
      // Apply edit to get new validated config
      const newConfigJson = await applyZenohConfigEdit(entry.configJson, edit);

      // Update entry
      configEntries.value[index] = new ZenohConfig(edit, newConfigJson);
    } catch (error) {
      console.error('Failed to update config:', error);
      throw error;
    }
  };

  const cloneConfig = async (index: number): Promise<number> => {
    const entry = configEntries.value[index];

    try {
      // Create new config with same mode but different port
      const configJson = await createZenohConfig(entry.edit);

      configEntries.value.push(new ZenohConfig(entry.edit, configJson));
      addActivityLog('success', `Cloned ${entry.edit.mode} config`);
      return configEntries.value.length - 1;
    } catch (error) {
      addActivityLog('error', `Failed to clone config: ${error}`);
      throw error;
    }
  };

  const removeConfig = (index: number) => {
    if (canRemoveConfig(index)) {
      const entry = configEntries.value[index];
      configEntries.value.splice(index, 1);
      addActivityLog('info', `Removed ${entry.edit.mode} config on port ${entry.websocket_port}`);
      // Note: Port is automatically released by Rust when config is dropped
    }
  };

  const canRemoveConfig = (index: number): boolean => {
    return getRuntimesForConfig(index).length === 0 && configEntries.value.length > 1;
  };

  const createRuntimeFromConfig = async (index: number) => {
    const entry = configEntries.value[index];

    try {
      // Create replacement config for next use
      const nextConfigJson = await createZenohConfig(entry.edit);

      // Update entry with next config
      configEntries.value[index] = new ZenohConfig(entry.edit, nextConfigJson);

      addActivityLog('info', `Starting runtime with ${entry.edit.mode} config on port ${entry.websocket_port}...`);

      await new Promise(resolve => setTimeout(resolve, 100));

      try {
        // Start runtime with original config
        const runtimeId = await invoke<string>('zenoh_runtime_start', {
          config: entry.configJson,
        });

        addActivityLog('success', `Runtime started: ${runtimeId} on port ${entry.websocket_port}`);

        // Store runtime config
        runtimeToConfigIndex[runtimeId] = index;
        runtimeConfigs[runtimeId] = entry;
        runtimes.value.push(runtimeId);
      } catch (error) {
        // Restore original config on error
        console.error('Failed to create runtime:', error);
        addActivityLog('error', `Failed to start runtime: ${error}`);
        configEntries.value[index] = entry;
      }
    } catch (error) {
      addActivityLog('error', `Failed to prepare runtime config: ${error}`);
    }
  };

  const stopRuntime = async (runtimeId: string) => {
    try {
      addActivityLog('info', `Stopping runtime ${runtimeId}...`);
      await invoke('zenoh_runtime_stop', { zid: runtimeId });

      // Remove from state
      const index = runtimes.value.indexOf(runtimeId);
      if (index > -1) {
        runtimes.value.splice(index, 1);
      }

      delete runtimeConfigs[runtimeId];
      delete runtimeToConfigIndex[runtimeId];

      addActivityLog('success', `Runtime ${runtimeId} stopped`);

      // Navigate away if needed
      const currentPath = router.currentRoute.value.path;
      if (currentPath.includes(runtimeId)) {
        navigateToActivityLog();
      }
    } catch (error) {
      addActivityLog('error', `Failed to stop runtime: ${error}`);
    }
  };

  const initializeFirstConfig = async () => {
    if (initialized) return;
    initialized = true;

    try {
      const edit: ZenohConfigEdit = { mode: 'peer' };
      const configJson = await createZenohConfig(edit);
      const config = new ZenohConfig(edit, configJson);
      configEntries.value.push(config);
      addActivityLog('info', `Initialized with default peer config on port ${config.websocket_port}`);
    } catch (error) {
      addActivityLog('error', `Failed to initialize config: ${error}`);
    }
  };

  // Auto-initialize on first use
  if (!initialized) {
    initializeFirstConfig();
  }

  return {
    // State
    runtimes,
    configEntries,
    runtimeConfigs,
    runtimeToConfigIndex,
    activityLogs,

    // Methods
    addActivityLog,
    clearActivityLogs,
    getRuntimesForConfig,
    navigateToActivityLog,
    navigateToConfigEdit,
    navigateToRuntimeConfig,
    navigateToRuntimeLogs,
    updateConfig,
    cloneConfig,
    removeConfig,
    canRemoveConfig,
    createRuntimeFromConfig,
    stopRuntime,
  };
}
