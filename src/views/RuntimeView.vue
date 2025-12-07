<template>
  <div class="runtime-view">
    <div class="runtime-tabs">
      <router-link
        :to="`/nodes/runtime/${runtimeId}/logs`"
        class="runtime-tab"
        active-class="active"
      >
        Logs
      </router-link>
      <router-link
        :to="`/nodes/runtime/${runtimeId}/config`"
        class="runtime-tab"
        active-class="active"
      >
        Config
      </router-link>
      <router-link
        :to="`/nodes/runtime/${runtimeId}/adminspace`"
        class="runtime-tab"
        active-class="active"
      >
        Adminspace
      </router-link>
    </div>

    <div class="runtime-content">
      <router-view :key="$route.fullPath" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useRoute } from 'vue-router';

const route = useRoute();
const runtimeId = computed(() => route.params.id as string);
</script>

<style scoped>
.runtime-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
}

.runtime-tabs {
  display: flex;
  gap: 0;
  background: var(--tabs-bg-color, #f8f9fa);
  border-bottom: 2px solid var(--border-color, #dee2e6);
  flex-shrink: 0;
}

.runtime-tab {
  padding: 0.75rem 1.5rem;
  border: none;
  background: transparent;
  cursor: pointer;
  font-size: 0.95rem;
  font-weight: 500;
  color: var(--text-muted-color, #6c757d);
  transition: all 0.2s;
  border-bottom: 3px solid transparent;
  text-decoration: none;
  text-align: center;
  display: block;
}

.runtime-tab:hover {
  background: var(--tab-hover-bg-color, #e9ecef);
  color: var(--text-color, #333);
}

.runtime-tab.active {
  color: var(--primary-color, #007bff);
  border-bottom-color: var(--primary-color, #007bff);
  background: var(--tab-active-bg-color, #fff);
}

.runtime-content {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
</style>
