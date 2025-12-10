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
        v-for="(_config, index) in configEntries"
        :key="index"
        title="Config"
        :descr="getConfigDescription(index)"
        :titleLink="`/nodes/config/${index}`"
        @title-click="navigateToConfigEdit(index)"
      >
        <template #actions>
          <button
            @click="startRuntimeWithNavigation(index)"
          >
            Start
          </button>
        </template>

        <!-- Info section for detailed diff -->
        <template #info>
          <pre class="config-diff">{{ getConfigDiffFormatted(index) }}</pre>
        </template>

        <!-- Active Runtimes for this Config as Sub-entities -->
        <template v-if="getRuntimesForConfig(index).length > 0" #sub-entities>
          <Entity
            v-for="runtimeId in getRuntimesForConfig(index)"
            :key="runtimeId"
            :title="runtimeId"
            :descr="`WS port: ${runtimePorts[runtimeId] || 'no WS port'}`"
            :titleLink="`/nodes/runtime/${runtimeId}`"
            @title-click="navigateToRuntimeLogs(runtimeId)"
          >
            <template #actions>
              <button
                @click="stopRuntime(runtimeId)"
              >
                stop
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
import Section from './Section.vue';
import Entity from './Entity.vue';
import { useNodesState } from '../composables/useNodesState';

const {
  configEntries,
  runtimePorts,
  getConfigDescription,
  getConfigDiffFormatted,
  getRuntimesForConfig,
  navigateToConfigEdit,
  navigateToRuntimeLogs,
  startRuntimeWithNavigation,
  stopRuntime,
} = useNodesState();
</script>

<style scoped>
.entity-panel {
  flex: 0 0 40%;
  min-width: 350px;
  overflow-y: auto;
  border-right: 1px solid var(--border-color, #dee2e6);
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
</style>
