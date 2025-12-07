<template>
  <LogPanel
    title="Activity Log"
    icon="📜"
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
