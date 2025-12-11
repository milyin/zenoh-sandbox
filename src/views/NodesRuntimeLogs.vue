<template>
  <LogPanel
    :title="`Runtime Logs - ${zenohId}`"
    icon="📜"
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
import { useRoute } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import LogPanel from '../components/LogPanel.vue';
import { useNodesState } from '../composables/useNodesState';
import { LogEntryLevel } from '../types/generated/LogEntryLevel';

interface LogEntry {
  timestamp: string;
  level: string;
  target: string;
  message: string;
}

const route = useRoute();
const { runtimes } = useNodesState();

const runtimeId = ref(parseInt(route.params.id as string));
const zenohId = computed(() => runtimes[runtimeId.value]?.zenohId || '');

const runtimeLogs = ref<LogEntry[]>([]);
const runtimeLogsPage = ref(0);
const isLoadingRuntimeLogs = ref(false);
const hasMoreRuntimeLogs = ref(true);

// Log level filtering - undefined means no filtering (show all logs)
const selectedLogLevel = ref<LogEntryLevel | undefined>(undefined);

const onLogLevelChange = (level: LogEntryLevel | undefined) => {
  selectedLogLevel.value = level;
  // Reload logs with new level filter
  clearRuntimeLogs();
  loadRuntimeLogs();
};

const loadRuntimeLogs = async () => {
  if (isLoadingRuntimeLogs.value || !zenohId.value) return;

  isLoadingRuntimeLogs.value = true;
  try {
    const params: { zid: string; page: number; level?: number } = {
      zid: zenohId.value,
      page: runtimeLogsPage.value
    };
    // Only include level if it's defined (null/undefined means no filter)
    if (selectedLogLevel.value !== undefined) {
      params.level = selectedLogLevel.value;
    }
    console.log('Loading logs with params:', params);
    const logs = await invoke<LogEntry[]>('zenoh_runtime_log', params);
    console.log('Received logs:', logs.length);

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
  if (isLoadingRuntimeLogs.value || !zenohId.value) return [];

  isLoadingRuntimeLogs.value = true;
  try {
    runtimeLogsPage.value++;
    const params: { zid: string; page: number; level?: number } = {
      zid: zenohId.value,
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
