import { ref, reactive } from 'vue';
import { useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import {
  type ZenohConfigEdit,
  ZenohConfig,
  createZenohConfig,
  validateConfigJson5,
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
    try {
      // Validate the JSON5 content
      const newConfigJson = await validateConfigJson5(edit.content);

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
      // Create new config with same content but different port
      const [newEdit, configJson] = await createZenohConfig(entry.edit);

      configEntries.value.push(new ZenohConfig(newEdit, configJson));
      addActivityLog('success', `Cloned ${entry.mode} config`);
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
      addActivityLog('info', `Removed ${entry.mode} config on port ${entry.websocket_port}`);
      // Note: Port is automatically released by Rust when config is dropped
    }
  };

  const canRemoveConfig = (index: number): boolean => {
    return getRuntimesForConfig(index).length === 0 && configEntries.value.length > 1;
  };

  const createRuntimeFromConfig = async (index: number): Promise<string | null> => {
    const entry = configEntries.value[index];

    try {
      // Debug: Log the config being used
      console.log('Creating runtime from config:', {
        editContent: entry.edit.content,
        configJson: entry.configJson,
        port: entry.websocket_port
      });

      // Create replacement config for next use
      const [nextEdit, nextConfigJson] = await createZenohConfig(entry.edit);

      // Update entry with next config
      configEntries.value[index] = new ZenohConfig(nextEdit, nextConfigJson);

      addActivityLog('info', `Starting runtime with ${entry.mode} config on port ${entry.websocket_port}...`);

      await new Promise(resolve => setTimeout(resolve, 100));

      try {
        // Start runtime with original config
        console.log('Invoking zenoh_runtime_start with config:', entry.configJson);
        const runtimeId = await invoke<string>('zenoh_runtime_start', {
          config: entry.configJson,
        });

        addActivityLog('success', `Runtime started: ${runtimeId} on port ${entry.websocket_port}`);

        // Store runtime config
        runtimeToConfigIndex[runtimeId] = index;
        runtimeConfigs[runtimeId] = entry;
        runtimes.value.push(runtimeId);

        return runtimeId; // Return the runtime ID on success
      } catch (error: any) {
        // Restore original config on error
        console.error('Failed to create runtime:', error);
        const errorMsg = error?.message || error?.toString() || 'Unknown error';
        addActivityLog('error', `Failed to start runtime: ${errorMsg}`);
        configEntries.value[index] = entry;
        throw error; // Re-throw so the UI can show it
      }
    } catch (error: any) {
      console.error('Failed to prepare runtime config:', error);
      const errorMsg = error?.message || error?.toString() || 'Unknown error';
      addActivityLog('error', `Failed to prepare runtime config: ${errorMsg}`);
      throw error; // Re-throw so the UI can show it
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
      // Start with empty config - the backend will apply defaults
      const edit: ZenohConfigEdit = { content: '{}' };
      const [finalEdit, configJson] = await createZenohConfig(edit);
      const config = new ZenohConfig(finalEdit, configJson);
      configEntries.value.push(config);
      addActivityLog('info', `Initialized with default config on port ${config.websocket_port}`);
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
