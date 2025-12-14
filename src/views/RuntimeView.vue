<template>
  <SectionTabGroup
    :activeTab="activeTab"
    v-model:collapsed="isCollapsed"
    header-background-color="var(--section-default-bg, #cbd5e1)"
    @update:activeTab="onTabChange"
  >
    <NodesRuntimeLogs :runtimeId="runtimeId" />
    <NodesRuntimeConfig :runtimeId="runtimeId" />
    <NodesRuntimeAdminspace :runtimeId="runtimeId" />
  </SectionTabGroup>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import SectionTabGroup from '../components/SectionTabGroup.vue';
import NodesRuntimeLogs from './NodesRuntimeLogs.vue';
import NodesRuntimeConfig from './NodesRuntimeConfig.vue';
import NodesRuntimeAdminspace from './NodesRuntimeAdminspace.vue';

const route = useRoute();
const router = useRouter();

const runtimeId = computed(() => parseInt(route.params.id as string));

// Determine initial active tab from route
const getActiveTabFromRoute = (): string => {
  const path = route.path;
  if (path.includes('/config')) return 'config';
  if (path.includes('/adminspace')) return 'adminspace';
  return 'logs';
};

const activeTab = ref(getActiveTabFromRoute());
const isCollapsed = ref(false);

// Update route when tab changes
const onTabChange = (tabId: string) => {
  activeTab.value = tabId;
  router.push(`/nodes/runtime/${runtimeId.value}/${tabId}`);
};

// Watch route changes to sync tab state
watch(() => route.path, () => {
  activeTab.value = getActiveTabFromRoute();
});
</script>
