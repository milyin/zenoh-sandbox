<template>
  <div class="app-container">
    <!-- Breadcrumb Navigation -->
    <Breadcrumb />

    <!-- Content -->
    <div class="content">
      <router-view />
    </div>
  </div>
</template>

<script setup lang="ts">
import Breadcrumb from './components/Breadcrumb.vue';
</script>

<style scoped>
.app-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100vw;
}

.content {
  flex: 1;
  overflow: hidden;
}
</style>
