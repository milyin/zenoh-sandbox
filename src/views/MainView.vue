<template>
  <SectionTabGroup
    v-model:activeTab="activeTab"
    v-model:collapsed="collapsed"
    header-background-color="var(--titlebar-bg, #e8e8e8)"
  >
    <template #actions>
      <!-- Breadcrumbs on the right -->
      <nav class="breadcrumb">
        <router-link
          v-for="(crumb, index) in breadcrumbs"
          :key="index"
          :to="crumb.path"
          class="breadcrumb-item"
          :class="{ active: index === breadcrumbs.length - 1 }"
        >
          {{ crumb.label }}
          <span v-if="index < breadcrumbs.length - 1" class="separator">→</span>
        </router-link>
      </nav>
    </template>

    <NodesSection />
    <SessionsSection />
  </SectionTabGroup>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import SectionTabGroup from '../components/SectionTabGroup.vue';
import NodesSection from './NodesSection.vue';
import SessionsSection from './SessionsSection.vue';

interface Breadcrumb {
  label: string;
  path: string;
}

const route = useRoute();
const router = useRouter();

const activeTab = ref('nodes');
const collapsed = ref(false);

// Sync active tab with route
watch(() => route.path, (path) => {
  if (path.startsWith('/nodes')) {
    activeTab.value = 'nodes';
  } else if (path.startsWith('/sessions')) {
    activeTab.value = 'sessions';
  }
}, { immediate: true });

// Navigate when tab changes
watch(activeTab, (tab) => {
  if (tab === 'nodes' && !route.path.startsWith('/nodes')) {
    router.push('/nodes');
  } else if (tab === 'sessions' && !route.path.startsWith('/sessions')) {
    router.push('/sessions');
  }
});

const breadcrumbs = computed<Breadcrumb[]>(() => {
  const crumbs: Breadcrumb[] = [];
  const pathSegments = route.path.split('/').filter(Boolean);

  if (pathSegments.length === 0) {
    return crumbs;
  }

  // Handle /nodes
  if (pathSegments[0] === 'nodes') {
    // Handle /nodes/config/:id/edit
    if (pathSegments[1] === 'config' && pathSegments[2]) {
      crumbs.push({ label: `Config ${pathSegments[2]}`, path: `/nodes/config/${pathSegments[2]}` });
      if (pathSegments[3] === 'edit') {
        crumbs.push({ label: 'Edit', path: route.path });
      }
    }

    // Handle /nodes/runtime/:id/...
    if (pathSegments[1] === 'runtime' && pathSegments[2]) {
      const runtimeId = pathSegments[2];
      const shortId = runtimeId.length > 16 ? `${runtimeId.slice(0, 8)}...${runtimeId.slice(-4)}` : runtimeId;
      crumbs.push({ label: `Runtime ${shortId}`, path: `/nodes/runtime/${runtimeId}` });

      if (pathSegments[3] === 'logs') {
        crumbs.push({ label: 'Logs', path: route.path });
      } else if (pathSegments[3] === 'config') {
        crumbs.push({ label: 'Config', path: route.path });
      }
    }
  }

  return crumbs;
});
</script>

<style scoped>
.breadcrumb {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.9rem;
}

.breadcrumb-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  text-decoration: none;
  color: var(--primary-color, #007bff);
  transition: color 0.2s;
}

.breadcrumb-item:hover {
  color: var(--primary-hover-color, #0056b3);
  text-decoration: underline;
}

.breadcrumb-item.active {
  color: var(--text-color, #333);
  font-weight: 500;
  pointer-events: none;
}

.separator {
  color: var(--text-muted-color, #6c757d);
  user-select: none;
}
</style>
