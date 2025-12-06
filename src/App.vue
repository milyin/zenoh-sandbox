<template>
  <div class="app-container">
    <!-- Tab Navigation -->
    <div class="tabs">
      <button
        class="tab-button"
        :class="{ active: activeTab === 'runtimes' }"
        @click="activeTab = 'runtimes'"
      >
        Zenoh nodes
      </button>
      <button
        class="tab-button"
        :class="{ active: activeTab === 'sessions' }"
        @click="activeTab = 'sessions'"
      >
        Sessions
      </button>
    </div>

    <!-- Tab Content -->
    <div class="tab-content">
      <RuntimesTab v-show="activeTab === 'runtimes'" ref="runtimesTabRef" />
      <SessionsTab v-show="activeTab === 'sessions'" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import RuntimesTab from './RuntimesTab.vue';
import SessionsTab from './SessionsTab.vue';

const activeTab = ref<'runtimes' | 'sessions'>('runtimes');
const runtimesTabRef = ref<InstanceType<typeof RuntimesTab> | null>(null);
</script>

<style scoped>
.app-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100vw;
}

.tabs {
  display: flex;
  gap: 0;
  background: var(--tabs-bg-color, #f8f9fa);
  border-bottom: 2px solid var(--border-color, #dee2e6);
}

.tab-button {
  flex: 1;
  padding: 1rem 2rem;
  border: none;
  background: transparent;
  cursor: pointer;
  font-size: 1rem;
  font-weight: 500;
  color: var(--text-muted-color, #6c757d);
  transition: all 0.2s;
  border-bottom: 3px solid transparent;
}

.tab-button:hover {
  background: var(--tab-hover-bg-color, #e9ecef);
  color: var(--text-color, #333);
}

.tab-button.active {
  color: var(--primary-color, #007bff);
  border-bottom-color: var(--primary-color, #007bff);
  background: var(--tab-active-bg-color, #fff);
}

.tab-content {
  flex: 1;
  overflow: hidden;
}
</style>
