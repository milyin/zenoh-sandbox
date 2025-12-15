<template>
  <Section
    id="nodes"
    title="Nodes"
    icon="⚙️"
    background-color="#fffef0"
    :collapsed="collapsed"
    @update:collapsed="emit('update:collapsed', $event)"
  >
    <div class="nodes-content">
      <NodesEntityPanel />
      <div class="content-panel">
        <router-view :key="$route.fullPath" />
      </div>
    </div>
  </Section>
</template>

<script setup lang="ts">
import { useRoute } from 'vue-router';
import Section from '../components/Section.vue';
import NodesEntityPanel from '../components/NodesEntityPanel.vue';

interface Props {
  collapsed?: boolean;
}

withDefaults(defineProps<Props>(), {
  collapsed: false
});

const emit = defineEmits<{
  (e: 'update:collapsed', value: boolean): void
}>();

const $route = useRoute();
</script>

<style scoped>
.nodes-content {
  display: flex;
  height: 100%;
  width: 100%;
  min-height: 0;
  background: transparent;
}

.content-panel {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  min-width: 0;
}
</style>
