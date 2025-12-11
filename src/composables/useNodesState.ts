import { ref, reactive } from 'vue';
import { useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import {
  type ZenohConfigEdit,
  type ZenohConfigJson,
  ZenohConfig,
  createZenohConfig,
  validateConfig,
  getDefaultConfigJson,
  computeConfigDiff,
} from '../types/zenohConfig';

interface ActivityLogEntry {
  timestamp: Date;
  type: "info" | "success" | "error" | "data";
  message: string;
  data?: Record<string, any>;
}

interface RuntimeEntry {
  configId: number;
  wsPort: number;
}

interface ConfigEntry {
  config: ZenohConfig;
  diff: string;
}

// Singleton state - shared across all instances
const runtimes = reactive<Record<string, RuntimeEntry>>({});
const configs = reactive<Record<number, ConfigEntry>>({});
let nextConfigId = 0;
const activityLogs = ref<ActivityLogEntry[]>([]);
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

  const updateConfigDiff = async (configId: number) => {
    if (!defaultConfigJson.value) return;

    try {
      const entry = configs[configId];
      if (!entry) return;

      const diff = await computeConfigDiff(defaultConfigJson.value, entry.config.configJson);

      // Format diff as compact JSON string
      const diffStr = JSON.stringify(diff);
      configs[configId].diff = diffStr === '{}' ? 'default' : diffStr;
    } catch (error) {
      console.error('Failed to compute config diff:', error);
      configs[configId].diff = 'error';
    }
  };

  const getConfigDescription = (configId: number): string => {
    const entry = configs[configId];
    if (!entry) return '';

    const diff = entry.diff;
    if (!diff) return '';
    if (diff === 'default') return '';
    if (diff === 'error') return 'error';

    return diff;
  };

  const getConfigDiffFormatted = (configId: number): string => {
    const entry = configs[configId];
    if (!entry || !defaultConfigJson.value) return '';

    const diff = entry.diff;
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

  const getRuntimesForConfig = (configId: number): string[] => {
    return Object.keys(runtimes).filter(
      (runtimeId) => runtimes[runtimeId].configId === configId
    );
  };

  const navigateToActivityLog = () => {
    router.push('/nodes');
  };

  const navigateToConfigEdit = (configId: number) => {
    router.push(`/nodes/config/${configId}/edit`);
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

  const updateConfig = async (configId: number, edit: ZenohConfigEdit) => {
    try {
      // Validate the JSON5 content
      const newConfigJson = await validateConfig(edit.content);

      // Update entry
      configs[configId].config = new ZenohConfig(edit, newConfigJson);

      // Update diff
      await updateConfigDiff(configId);
    } catch (error) {
      console.error('Failed to update config:', error);
      throw error;
    }
  };

  const cloneConfig = async (configId: number): Promise<number> => {
    const entry = configs[configId];
    if (!entry) throw new Error('Config not found');

    try {
      // Create new config with same content
      const [newEdit, configJson] = await createZenohConfig(entry.config.edit);

      const newConfigId = nextConfigId++;
      configs[newConfigId] = {
        config: new ZenohConfig(newEdit, configJson),
        diff: '',
      };

      // Update diff for new config
      await updateConfigDiff(newConfigId);

      addActivityLog('success', `Cloned ${entry.config.mode} config`);
      return newConfigId;
    } catch (error) {
      addActivityLog('error', `Failed to clone config: ${error}`);
      throw error;
    }
  };

  const removeConfig = (configId: number) => {
    if (canRemoveConfig(configId)) {
      const entry = configs[configId];
      delete configs[configId];
      addActivityLog('info', `Removed ${entry.config.mode} config`);
    }
  };

  const canRemoveConfig = (configId: number): boolean => {
    return getRuntimesForConfig(configId).length === 0 && Object.keys(configs).length > 1;
  };

  const startRuntimeWithNavigation = async (configId: number) => {
    // Navigate to activity log first to show progress
    navigateToActivityLog();

    try {
      const runtimeId = await createRuntimeFromConfig(configId);

      // If runtime started successfully, navigate to it
      if (runtimeId) {
        navigateToRuntime(runtimeId);
      }
    } catch (error: any) {
      // Error is already logged to activity log, stay on activity log page
      console.error('Start runtime error:', error);
    }
  };

  const createRuntimeFromConfig = async (configId: number): Promise<string | null> => {
    const entry = configs[configId];
    if (!entry) throw new Error('Config not found');

    try {
      // Debug: Log the config being used
      console.log('Creating runtime from config:', {
        editContent: entry.config.edit.content,
        configJson: entry.config.configJson,
      });

      addActivityLog('info', `Starting runtime with ${entry.config.mode} config...`);

      await new Promise(resolve => setTimeout(resolve, 100));

      try {
        // Start runtime with config - returns [runtimeId, port]
        console.log('Invoking zenoh_runtime_start with config:', entry.config.configJson);
        const [runtimeId, port] = await invoke<[string, number]>('zenoh_runtime_start', {
          config: entry.config.configJson,
        });

        addActivityLog('success', `Runtime started: ${runtimeId} on port ${port}`);

        // Store runtime entry
        runtimes[runtimeId] = {
          configId: configId,
          wsPort: port,
        };

        return runtimeId; // Return the runtime ID on success
      } catch (error: any) {
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
      const runtime = runtimes[runtimeId];
      const port = runtime?.wsPort;
      addActivityLog('info', `Stopping runtime ${runtimeId}${port ? ` on port ${port}` : ''}...`);
      await invoke('zenoh_runtime_stop', { zid: runtimeId });

      // Remove from state
      delete runtimes[runtimeId];

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

      const configId = nextConfigId++;
      configs[configId] = {
        config,
        diff: '',
      };

      // Update diff for initial config
      await updateConfigDiff(configId);

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
    configs,
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
