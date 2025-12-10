import { ref, reactive } from 'vue';
import { useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import {
  type ZenohConfigEdit,
  type ZenohConfigJson,
  ZenohConfig,
  createZenohConfig,
  validateConfigJson5,
  getDefaultConfigJson,
  computeConfigDiff,
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
const runtimeToConfigIndex = reactive<Record<string, number>>({});
const runtimePorts = reactive<Record<string, number>>({});
const activityLogs = ref<ActivityLogEntry[]>([]);
const configDiffs = reactive<Record<number, string>>({});
let defaultConfigJson = ref<ZenohConfigJson | null>(null);
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

  const updateConfigDiff = async (index: number) => {
    if (!defaultConfigJson.value) return;

    try {
      const config = configEntries.value[index];
      const diff = await computeConfigDiff(defaultConfigJson.value, config.configJson);

      // Format diff as compact JSON string
      const diffStr = JSON.stringify(diff);
      configDiffs[index] = diffStr === '{}' ? 'default' : diffStr;
    } catch (error) {
      console.error('Failed to compute config diff:', error);
      configDiffs[index] = 'error';
    }
  };

  const getConfigDescription = (index: number): string => {
    const config = configEntries.value[index];
    if (!config) return '';

    const diff = configDiffs[index];
    if (!diff) return '';
    if (diff === 'default') return '';
    if (diff === 'error') return 'error';

    return diff;
  };

  const getConfigDiffFormatted = (index: number): string => {
    const config = configEntries.value[index];
    if (!config || !defaultConfigJson.value) return '';

    const diff = configDiffs[index];
    if (!diff || diff === 'default') return 'No differences from default configuration';
    if (diff === 'error') return 'Error computing differences';

    try {
      // Parse and format the diff JSON
      const diffObj = JSON.parse(diff);
      return JSON.stringify(diffObj, null, 2);
    } catch (error) {
      return diff;
    }
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

  const navigateToRuntime = (runtimeId: string) => {
    router.push(`/nodes/runtime/${runtimeId}`);
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

      // Update diff
      await updateConfigDiff(index);
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
      const newIndex = configEntries.value.length - 1;

      // Update diff for new config
      await updateConfigDiff(newIndex);

      addActivityLog('success', `Cloned ${entry.mode} config`);
      return newIndex;
    } catch (error) {
      addActivityLog('error', `Failed to clone config: ${error}`);
      throw error;
    }
  };

  const removeConfig = (index: number) => {
    if (canRemoveConfig(index)) {
      const entry = configEntries.value[index];
      configEntries.value.splice(index, 1);
      addActivityLog('info', `Removed ${entry.mode} config`);
    }
  };

  const canRemoveConfig = (index: number): boolean => {
    return getRuntimesForConfig(index).length === 0 && configEntries.value.length > 1;
  };

  const startRuntimeWithNavigation = async (index: number) => {
    // Navigate to activity log first to show progress
    navigateToActivityLog();

    try {
      const runtimeId = await createRuntimeFromConfig(index);

      // If runtime started successfully, navigate to it
      if (runtimeId) {
        navigateToRuntime(runtimeId);
      }
    } catch (error: any) {
      // Error is already logged to activity log, stay on activity log page
      console.error('Start runtime error:', error);
    }
  };

  const createRuntimeFromConfig = async (index: number): Promise<string | null> => {
    const entry = configEntries.value[index];

    try {
      // Debug: Log the config being used
      console.log('Creating runtime from config:', {
        editContent: entry.edit.content,
        configJson: entry.configJson,
      });

      addActivityLog('info', `Starting runtime with ${entry.mode} config...`);

      await new Promise(resolve => setTimeout(resolve, 100));

      try {
        // Start runtime with config - returns [runtimeId, port]
        console.log('Invoking zenoh_runtime_start with config:', entry.configJson);
        const [runtimeId, port] = await invoke<[string, number]>('zenoh_runtime_start', {
          config: entry.configJson,
        });

        addActivityLog('success', `Runtime started: ${runtimeId} on port ${port}`);

        // Store runtime config
        runtimeToConfigIndex[runtimeId] = index;
        runtimePorts[runtimeId] = port;
        runtimes.value.push(runtimeId);

        return runtimeId; // Return the runtime ID on success
      } catch (error: any) {
        // Restore original config on error
        configEntries.value[index] = entry;

        throw error;
      }
    } catch (error: any) {
      console.error('Failed to prepare runtime config:', error);
      const errorMsg = error?.message || error?.toString() || 'Unknown error';
      addActivityLog('error', `Failed to prepare runtime config: ${errorMsg}`);
      throw error;
    }
  };

  const stopRuntime = async (runtimeId: string) => {
    try {
      const port = runtimePorts[runtimeId];
      addActivityLog('info', `Stopping runtime ${runtimeId}${port ? ` on port ${port}` : ''}...`);
      await invoke('zenoh_runtime_stop', { zid: runtimeId });

      // Remove from state
      const index = runtimes.value.indexOf(runtimeId);
      if (index > -1) {
        runtimes.value.splice(index, 1);
      }

      delete runtimeToConfigIndex[runtimeId];
      delete runtimePorts[runtimeId];

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
      // Load default config for diff comparisons
      const defaultJsonStr = await getDefaultConfigJson();
      defaultConfigJson.value = JSON.parse(defaultJsonStr);

      // Start with empty config - the backend will apply defaults
      const edit: ZenohConfigEdit = { content: '{}' };
      const [finalEdit, configJson] = await createZenohConfig(edit);
      const config = new ZenohConfig(finalEdit, configJson);
      configEntries.value.push(config);

      // Update diff for initial config
      await updateConfigDiff(0);

      addActivityLog('info', `Initialized with default config`);
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
    runtimeToConfigIndex,
    runtimePorts,
    activityLogs,

    // Methods
    addActivityLog,
    clearActivityLogs,
    getConfigDescription,
    getConfigDiffFormatted,
    getRuntimesForConfig,
    navigateToActivityLog,
    navigateToConfigEdit,
    navigateToRuntime,
    navigateToRuntimeConfig,
    navigateToRuntimeLogs,
    updateConfig,
    cloneConfig,
    removeConfig,
    canRemoveConfig,
    createRuntimeFromConfig,
    startRuntimeWithNavigation,
    stopRuntime,
  };
}
