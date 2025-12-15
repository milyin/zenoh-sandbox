<template>
  <SectionTabGroup
    :activeTab="activeTab"
    v-model:collapsed="isCollapsed"
    header-background-color="transparent"
    :boxed="true"
    @update:activeTab="onTabChange"
  >
    <NodesConfigEdit :id="configId" />
    <NodesActivityLogSection />
  </SectionTabGroup>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import SectionTabGroup from '../components/SectionTabGroup.vue';
import NodesConfigEdit from './NodesConfigEdit.vue';
import NodesActivityLogSection from './NodesActivityLogSection.vue';

const route = useRoute();
const router = useRouter();

const configId = computed(() => parseInt(route.params.id as string));

// Determine initial active tab from route
const getActiveTabFromRoute = (): string => {
  const path = route.path;
  if (path.includes('/activity')) return 'activity';
  return 'edit';
};

const activeTab = ref(getActiveTabFromRoute());
const isCollapsed = ref(false);

// Update route when tab changes
const onTabChange = (tabId: string) => {
  activeTab.value = tabId;
  router.push(`/nodes/config/${configId.value}/${tabId}`);
};

// Watch route changes to sync tab state
watch(() => route.path, () => {
  activeTab.value = getActiveTabFromRoute();
});
</script>
