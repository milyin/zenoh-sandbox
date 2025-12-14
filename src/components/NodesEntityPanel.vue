<template>
  <div class="entity-panel">
    <!-- Local Section -->
    <Section
      title="Local"
      icon="⚙️"
      section-class="runtimes-section"
    >
      <!-- Config Entities -->
      <Entity
        v-for="(_entry, configId) in configs"
        :key="configId"
        :title="`#${configId}`"
        :descr="getConfigDescription(Number(configId)) || 'default configuration'"
        :selected="selectedConfigId === Number(configId)"
        @title-click="navigateToConfigEdit(Number(configId))"
        @descr-click="navigateToConfigEdit(Number(configId))"
      >
        <template #actions>
          <button
            @click="startRuntimeWithNavigation(Number(configId))"
            :disabled="hasConfigValidationError(Number(configId))"
            :class="{ 'disabled-button': hasConfigValidationError(Number(configId)) }"
          >
            Start
          </button>
        </template>

        <!-- Info section for detailed diff -->
        <template #info>
          <pre class="config-diff">{{ getConfigDiffFormatted(Number(configId)) }}</pre>
        </template>

        <!-- Active Runtimes for this Config as Sub-entities -->
        <template v-if="getRuntimesForConfig(Number(configId)).length > 0" #sub-entities>
          <Entity
            v-for="runtimeId in getRuntimesForConfig(Number(configId))"
            :key="runtimeId"
            :title="`${runtimes[runtimeId]?.zenohId || 'runtime'} (#${runtimeId})`"
            :descr="`WS port: ${runtimes[runtimeId]?.wsPort || 'no WS port'}${runtimes[runtimeId]?.stopped ? ' (stopped)' : ''}`"
            :selected="selectedRuntimeId === runtimeId"
            :class="{ 'stopped-runtime': runtimes[runtimeId]?.stopped }"
            @title-click="navigateToRuntimeLogs(runtimeId)"
          >
            <template #actions>
              <button
                v-if="!runtimes[runtimeId]?.stopped"
                @click="stopRuntime(runtimeId)"
              >
                stop
              </button>
              <button
                v-else
                @click="removeRuntime(runtimeId)"
                class="remove-button"
              >
                remove
              </button>
            </template>
          </Entity>
        </template>
      </Entity>
    </Section>

    <!-- External Section -->
    <Section
      title="External"
      icon="🌐"
      section-class="external-section"
    >
    </Section>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useRoute } from 'vue-router';
import Section from './Section.vue';
import Entity from './Entity.vue';
import { useNodesState } from '../composables/useNodesState';

const {
  runtimes,
  configs,
  getConfigDescription,
  getConfigDiffFormatted,
  getRuntimesForConfig,
  hasConfigValidationError,
  navigateToConfigEdit,
  navigateToRuntimeLogs,
  startRuntimeWithNavigation,
  stopRuntime,
  removeRuntime,
} = useNodesState();

const route = useRoute();

const selectedConfigId = computed(() => {
  if (route.path.includes('/nodes/config/')) {
    const id = route.params.id;
    return id ? Number(id) : null;
  }
  return null;
});

const selectedRuntimeId = computed(() => {
  if (route.path.includes('/nodes/runtime/')) {
    const id = route.params.id;
    return id ? Number(id) : null;
  }
  return null;
});
</script>

<style scoped>
.entity-panel {
  flex: 0 0 40%;
  min-width: 350px;
  overflow-y: auto;
  border-right: 1px solid var(--border-color, #dee2e6);
  background: var(--panel-bg, #f8fafc);
}

.config-diff {
  margin: 0;
  padding: 0.5rem;
  background: var(--code-bg-color, #f8f9fa);
  border: 1px solid var(--border-color, #dee2e6);
  border-radius: 4px;
  font-family: 'Courier New', Courier, monospace;
  font-size: 0.85rem;
  line-height: 1.4;
  overflow-x: auto;
  white-space: pre;
}

button.disabled-button {
  opacity: 0.4;
  cursor: not-allowed;
  background: var(--disabled-bg-color, #e9ecef) !important;
  color: var(--text-muted-color, #6c757d) !important;
}

.stopped-runtime {
  opacity: 0.6;
}

.stopped-runtime :deep(.entity-title),
.stopped-runtime :deep(.entity-descr) {
  color: var(--text-muted-color, #6c757d);
}

button.remove-button {
  background: var(--danger-color, #dc3545);
  color: white;
}

button.remove-button:hover {
  background: var(--danger-hover-color, #c82333);
}
</style>
