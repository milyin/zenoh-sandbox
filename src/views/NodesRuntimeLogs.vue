<template>
  <LogPanel
    id="logs"
    title="Logs"
    icon="📜"
    background-color="var(--section-logs-bg, #e2e8f0)"
    :logs="runtimeLogs"
    :showLogLevelFilter="true"
    :selectedLogLevel="selectedLogLevel"
    @update:selectedLogLevel="onLogLevelChange"
    :onLoadMore="hasMoreRuntimeLogs ? loadMoreRuntimeLogs : undefined"
    :onClear="clearRuntimeLogs"
    :showClearButton="true"
  />
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import LogPanel from '../components/LogPanel.vue';
import { useNodesState } from '../composables/useNodesState';
import { LogEntryLevel } from '../types/generated/LogEntryLevel';

interface Props {
  runtimeId: number;
}

const props = defineProps<Props>();

interface LogEntry {
  timestamp: string;
  level: number;
  target: string;
  message: string;
}

const { runtimes } = useNodesState();

const runtimeLogs = ref<LogEntry[]>([]);
const runtimeLogsPage = ref(0);
const isLoadingRuntimeLogs = ref(false);
const hasMoreRuntimeLogs = ref(true);

const selectedLogLevel = computed({
  get: () => runtimes[props.runtimeId]?.logLevel,
  set: (value) => {
    if (runtimes[props.runtimeId]) {
      runtimes[props.runtimeId].logLevel = value;
    }
  }
});

const onLogLevelChange = (level: LogEntryLevel | undefined) => {
  selectedLogLevel.value = level;
  clearRuntimeLogs();
  loadRuntimeLogs();
};

const loadRuntimeLogs = async () => {
  if (isLoadingRuntimeLogs.value) return;

  isLoadingRuntimeLogs.value = true;
  try {
    const params: { runtimeId: number; page: number; level?: number } = {
      runtimeId: props.runtimeId,
      page: runtimeLogsPage.value
    };
    if (selectedLogLevel.value !== undefined) {
      params.level = selectedLogLevel.value;
    }
    const logs = await invoke<LogEntry[]>('zenoh_runtime_log', params);

    if (logs.length === 0) {
      hasMoreRuntimeLogs.value = false;
    } else {
      runtimeLogs.value.push(...logs);
    }
  } catch (error) {
    console.error('Failed to load runtime logs:', error);
  } finally {
    isLoadingRuntimeLogs.value = false;
  }
};

const loadMoreRuntimeLogs = async (_currentCount: number): Promise<LogEntry[]> => {
  if (isLoadingRuntimeLogs.value) return [];

  isLoadingRuntimeLogs.value = true;
  try {
    runtimeLogsPage.value++;
    const params: { runtimeId: number; page: number; level?: number } = {
      runtimeId: props.runtimeId,
      page: runtimeLogsPage.value
    };
    if (selectedLogLevel.value !== undefined) {
      params.level = selectedLogLevel.value;
    }
    const logs = await invoke<LogEntry[]>('zenoh_runtime_log', params);

    if (logs.length === 0) {
      hasMoreRuntimeLogs.value = false;
    } else {
      runtimeLogs.value.push(...logs);
    }
    return logs;
  } catch (error) {
    console.error('Failed to load runtime logs:', error);
    return [];
  } finally {
    isLoadingRuntimeLogs.value = false;
  }
};

const clearRuntimeLogs = () => {
  runtimeLogs.value = [];
  runtimeLogsPage.value = 0;
  hasMoreRuntimeLogs.value = true;
};

onMounted(() => {
  loadRuntimeLogs();
});
</script>
