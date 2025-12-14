<template>
  <Section
    id="adminspace"
    title="Adminspace"
    icon="🔧"
    section-class="info-section"
    background-color="var(--section-default-bg, #cbd5e1)"
  >
    <div class="info-content">
      <div class="adminspace-container">
        <p class="placeholder-text">Adminspace view - Coming soon</p>
        <p class="info-text">
          This view will show adminspace queries and operations for runtime: {{ zenohId }}
        </p>
      </div>
    </div>
  </Section>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import Section from '../components/Section.vue';
import { useNodesState } from '../composables/useNodesState';

interface Props {
  runtimeId: number;
}

const props = defineProps<Props>();

const { runtimes } = useNodesState();
const zenohId = computed(() => runtimes[props.runtimeId]?.zenohId || '');
</script>

<style scoped>
.info-content {
  padding: 1rem;
  overflow-y: auto;
  flex: 1;
}

.adminspace-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 3rem 2rem;
  text-align: center;
}

.placeholder-text {
  font-size: 1.5rem;
  color: var(--text-muted-color, #6c757d);
  margin: 0 0 1rem 0;
  font-weight: 500;
}

.info-text {
  color: var(--text-muted-color, #6c757d);
  margin: 0;
  font-size: 1rem;
}
</style>
