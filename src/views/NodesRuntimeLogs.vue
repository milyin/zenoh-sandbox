<template>
  <LogPanel
    :title="`Runtime Logs - ${runtimeId}`"
    icon="📜"
    :logs="runtimeLogs"
    :onLoadMore="hasMoreRuntimeLogs ? loadMoreRuntimeLogs : undefined"
    :onClear="clearRuntimeLogs"
    :showClearButton="true"
  />
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import LogPanel from '../components/LogPanel.vue';

interface LogEntry {
  timestamp: string;
  level: string;
  target: string;
  message: string;
}

const route = useRoute();
const runtimeId = ref(route.params.id as string);
const runtimeLogs = ref<LogEntry[]>([]);
const runtimeLogsPage = ref(0);
const isLoadingRuntimeLogs = ref(false);
const hasMoreRuntimeLogs = ref(true);

const loadRuntimeLogs = async () => {
  if (isLoadingRuntimeLogs.value) return;

  isLoadingRuntimeLogs.value = true;
  try {
    const logs = await invoke<LogEntry[]>('zenoh_runtime_log', {
      zid: runtimeId.value,
      page: runtimeLogsPage.value
    });

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
    const logs = await invoke<LogEntry[]>('zenoh_runtime_log', {
      zid: runtimeId.value,
      page: runtimeLogsPage.value
    });

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
