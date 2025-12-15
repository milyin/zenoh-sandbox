<template>
  <LogPanel
    id="activity"
    title="Activity Log"
    icon="📜"
    background-color="var(--section-logs-bg, #e2e8f0)"
    :logs="activityLogs"
    :onClear="clearActivityLogs"
    :showClearButton="true"
  />
</template>

<script setup lang="ts">
import LogPanel from '../components/LogPanel.vue';
import { useNodesState } from '../composables/useNodesState';

const { activityLogs, clearActivityLogs } = useNodesState();
</script>
