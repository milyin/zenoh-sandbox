<template>
  <div class="nodes-view">
    <NodesEntityPanel />
    <div class="content-panel">
      <router-view :key="$route.fullPath" />
    </div>
  </div>
</template>

<script setup lang="ts">
import NodesEntityPanel from '../components/NodesEntityPanel.vue';
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
