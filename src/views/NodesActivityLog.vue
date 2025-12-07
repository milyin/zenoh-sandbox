<template>
  <LogPanel
    title="Activity Log"
    icon="📜"
    :logs="activityLogs"
    :onClear="onClearActivityLogs"
    :showClearButton="true"
  />
</template>

<script setup lang="ts">
import LogPanel from '../components/LogPanel.vue';

interface ActivityLogEntry {
  timestamp: Date;
  type: "info" | "success" | "error" | "data";
  message: string;
  data?: Record<string, any>;
}

interface Props {
  activityLogs: ActivityLogEntry[];
  onClearActivityLogs: () => void;
}

defineProps<Props>();
</script>
