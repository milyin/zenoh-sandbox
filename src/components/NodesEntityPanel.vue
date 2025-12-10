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
        @title-click="navigateToConfigEdit(Number(configId))"
        @descr-click="navigateToConfigEdit(Number(configId))"
      >
        <template #actions>
          <button
            @click="startRuntimeWithNavigation(Number(configId))"
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
            :title="runtimeId"
            :descr="`WS port: ${runtimes[runtimeId]?.wsPort || 'no WS port'}`"
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
  runtimes,
  configs,
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
