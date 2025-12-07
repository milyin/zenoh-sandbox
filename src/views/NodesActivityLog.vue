<template>
  <div class="nodes-view">
    <NodesEntityPanel />
    <div class="content-panel">
      <LogPanel
        title="Activity Log"
        icon="📜"
        :logs="activityLogs"
        :onClear="clearActivityLogs"
        :showClearButton="true"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import LogPanel from '../components/LogPanel.vue';
import NodesEntityPanel from '../components/NodesEntityPanel.vue';
import { useNodesState } from '../composables/useNodesState';

const { activityLogs, clearActivityLogs } = useNodesState();
</script>

<style scoped>
.nodes-view {
  display: flex;
  height: 100%;
  width: 100%;
}

.content-panel {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}
</style>
