<template>
  <Section :title="`Adminspace - ${runtimeId}`" icon="🔧" section-class="info-section">
    <div class="info-content">
      <div class="adminspace-container">
        <p class="placeholder-text">Adminspace view - Coming soon</p>
        <p class="info-text">
          This view will show adminspace queries and operations for runtime: {{ runtimeId }}
        </p>
      </div>
    </div>
  </Section>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useRoute } from 'vue-router';
import Section from '../components/Section.vue';

const route = useRoute();
const runtimeId = ref(route.params.id as string);
</script>

<style scoped>
.info-content {
  padding: 1rem;
  overflow-y: auto;
  max-height: calc(100vh - 200px);
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
