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
import { LogEntryLevel } from '../types/generated/LogEntryLevel';

interface ActivityLogEntry {
  timestamp: Date;
  type: "info" | "success" | "error" | "data";
  message: string;
  data?: Record<string, any>;
}

interface RuntimeEntry {
  zenohId: string | undefined;
  configId: number;
  wsPort: number;
  logLevel: LogEntryLevel | undefined;
  stopped: boolean;
}

interface ConfigEntry {
  config: ZenohConfig;
  diff: string;
  hasValidationError: boolean;
}

// Singleton state - shared across all instances
const runtimes = reactive<Record<number, RuntimeEntry>>({});
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

  const getRuntimesForConfig = (configId: number): number[] => {
    return Object.keys(runtimes)
      .map(Number)
      .filter((runtimeId) => runtimes[runtimeId].configId === configId);
  };

  const navigateToActivityLog = () => {
    router.push('/nodes');
  };

  const navigateToConfigEdit = (configId: number) => {
    router.push(`/nodes/config/${configId}/edit`);
  };

  const navigateToRuntime = (runtimeId: number) => {
    router.push(`/nodes/runtime/${runtimeId}`);
  };

  const navigateToRuntimeConfig = (runtimeId: number) => {
    router.push(`/nodes/runtime/${runtimeId}/config`);
  };

  const navigateToRuntimeLogs = (runtimeId: number) => {
    router.push(`/nodes/runtime/${runtimeId}/logs`);
  };

  const updateConfig = async (configId: number, edit: ZenohConfigEdit): Promise<boolean> => {
    try {
      // Validate the JSON5 content
      const newConfigJson = await validateConfig(edit.content);

      // Update entry
      configs[configId].config = new ZenohConfig(edit, newConfigJson);
      configs[configId].hasValidationError = false;

      // Update diff
      await updateConfigDiff(configId);

      return true;
    } catch (error) {
      console.error('Failed to update config:', error);
      configs[configId].hasValidationError = true;
      return false;
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
        hasValidationError: false,
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

  const hasConfigValidationError = (configId: number): boolean => {
    return configs[configId]?.hasValidationError || false;
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

  const createRuntimeFromConfig = async (configId: number): Promise<number | null> => {
    const entry = configs[configId];
    if (!entry) throw new Error('Config not found');

    try {
      // Debug: Log the config being used
      console.log('Creating runtime from config:', {
        editContent: entry.config.edit.content,
        configJson: entry.config.configJson,
      });

      addActivityLog('info', `Starting runtime with ${entry.config.mode} config...`);

      // Step 1: Declare runtime (allocates resources)
      console.log('Declaring runtime with config:', entry.config.configJson);
      const runtimeId = await invoke<number>('declare_runtime', {
        config: entry.config.configJson,
      });

      // Step 2: Add to frontend state immediately (before starting)
      runtimes[runtimeId] = {
        zenohId: undefined,
        configId: configId,
        wsPort: 0, // Will be set when runtime starts
        logLevel: LogEntryLevel.INFO,
        stopped: false, // Initially not stopped (starting...)
      };

      // Step 3: Start the runtime
      try {
        console.log('Starting runtime with id:', runtimeId);
        const zenohId = await invoke<string>('start_runtime', {
          runtimeId: runtimeId,
        });

        // Update with zenohId
        runtimes[runtimeId].zenohId = zenohId;
        // wsPort is already allocated by declare_runtime on backend

        addActivityLog('success', `Runtime started: ${zenohId}`);

        return runtimeId;
      } catch (error: any) {
        console.error('Failed to start runtime:', error);
        const errorMsg = error?.message || error?.toString() || 'Unknown error';
        addActivityLog('error', `Failed to start runtime: ${errorMsg}`);

        // Mark as stopped so it stays visible with error logs
        runtimes[runtimeId].stopped = true;

        return runtimeId; // Return the ID so it's visible in the UI
      }
    } catch (error: any) {
      console.error('Failed to declare runtime:', error);
      const errorMsg = error?.message || error?.toString() || 'Unknown error';
      addActivityLog('error', `Failed to declare runtime: ${errorMsg}`);
      throw error;
    }
  };

  const stopRuntime = async (runtimeId: number) => {
    try {
      const runtime = runtimes[runtimeId];
      if (!runtime) {
        addActivityLog('error', `Runtime ${runtimeId} not found`);
        return;
      }

      const { zenohId } = runtime;
      const displayId = zenohId || `#${runtimeId}`;
      addActivityLog('info', `Stopping runtime ${displayId}...`);
      await invoke('zenoh_runtime_stop', { runtimeId });

      // Mark as stopped instead of removing from state
      runtimes[runtimeId].stopped = true;

      addActivityLog('success', `Runtime ${displayId} stopped`);

      // Don't navigate away - keep viewing the stopped runtime's logs
    } catch (error) {
      addActivityLog('error', `Failed to stop runtime: ${error}`);
    }
  };

  const removeRuntime = async (runtimeId: number) => {
    const runtime = runtimes[runtimeId];
    if (!runtime) {
      addActivityLog('error', `Runtime ${runtimeId} not found`);
      return;
    }

    if (!runtime.stopped) {
      const displayId = runtime.zenohId || `#${runtimeId}`;
      addActivityLog('error', `Cannot remove running runtime ${displayId}. Stop it first.`);
      return;
    }

    try {
      // Cleanup logs and remove from backend
      await invoke('zenoh_runtime_cleanup', { runtimeId });

      const displayId = runtime.zenohId || `#${runtimeId}`;
      delete runtimes[runtimeId];
      addActivityLog('info', `Removed runtime ${displayId}`);

      // Navigate away if needed
      const currentPath = router.currentRoute.value.path;
      if (currentPath.includes(String(runtimeId))) {
        navigateToActivityLog();
      }
    } catch (error) {
      addActivityLog('error', `Failed to cleanup runtime: ${error}`);
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
        hasValidationError: false,
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
    hasConfigValidationError,
    createRuntimeFromConfig,
    startRuntimeWithNavigation,
    stopRuntime,
    removeRuntime,
  };
}
