import { ref, reactive } from 'vue';
import { useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import {
  type ZenohConfig,
  createZenohConfig,
  cleanupConfig
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
    return getRuntimesForConfig(index).length === 0;
  };

  const createRuntimeFromConfig = async (index: number) => {
    const configToUse = configEntries.value[index];

    try {
      const nextConfig = await createZenohConfig(configToUse.mode);
      configEntries.value[index] = nextConfig;

      addActivityLog('info', `Starting runtime with ${configToUse.mode} config...`);

      await new Promise(resolve => setTimeout(resolve, 100));

      try {
        const runtimeId = await invoke<string>('zenoh_runtime_start', { config: configToUse });
        addActivityLog('success', `Runtime started: ${runtimeId} on port ${configToUse.websocket_port}`);

        runtimeToConfigIndex[runtimeId] = index;
        runtimeConfigs[runtimeId] = { ...configToUse };
        cleanupConfig(configToUse);
        runtimes.value.push(runtimeId);
      } catch (error) {
        console.error('Failed to create runtime:', error);
        addActivityLog('error', `Failed to start runtime: ${error}`);
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

      const index = runtimes.value.indexOf(runtimeId);
      if (index > -1) {
        runtimes.value.splice(index, 1);
      }

      const config = runtimeConfigs[runtimeId];
      if (config) {
        cleanupConfig(config);
        delete runtimeConfigs[runtimeId];
      }

      delete runtimeToConfigIndex[runtimeId];

      addActivityLog('success', `Runtime ${runtimeId} stopped`);

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
      const defaultConfig = await createZenohConfig('peer');
      configEntries.value.push(defaultConfig);
      addActivityLog('info', 'Initialized with default peer config');
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
