<template>
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

<script setup lang="ts">
import { computed } from 'vue';
import { useRoute } from 'vue-router';

interface Breadcrumb {
  label: string;
  path: string;
}

const route = useRoute();

const breadcrumbs = computed<Breadcrumb[]>(() => {
  const crumbs: Breadcrumb[] = [
    { label: 'Home', path: '/' }
  ];

  const pathSegments = route.path.split('/').filter(Boolean);

  if (pathSegments.length === 0) {
    return crumbs;
  }

  // Handle /nodes
  if (pathSegments[0] === 'nodes') {
    crumbs.push({ label: 'Nodes', path: '/nodes' });

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

  // Handle /sessions
  if (pathSegments[0] === 'sessions') {
    crumbs.push({ label: 'Sessions', path: '/sessions' });
  }

  return crumbs;
});
</script>

<style scoped>
.breadcrumb {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem 1.5rem;
  background: var(--breadcrumb-bg-color, #f8f9fa);
  border-bottom: 1px solid var(--border-color, #dee2e6);
  font-size: 0.95rem;
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
