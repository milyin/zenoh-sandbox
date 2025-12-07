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
        v-for="(config, index) in configEntries"
        :key="index"
        title="Config"
        :descr="`Mode: ${config.mode}`"
        :titleLink="`/nodes/config/${index}`"
        @title-click="navigateToConfigEdit(index)"
      >
        <template #actions>
          <button
            @click="createRuntimeFromConfig(index)"
          >
            Start
          </button>
        </template>

        <!-- Active Runtimes for this Config as Sub-entities -->
        <template v-if="getRuntimesForConfig(index).length > 0" #sub-entities>
          <Entity
            v-for="runtimeId in getRuntimesForConfig(index)"
            :key="runtimeId"
            :title="runtimeId"
            :descr="`Port: ${runtimeConfigs[runtimeId]?.websocket_port || 'Loading...'}`"
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

            <template #info>
              <div class="runtime-info-panel">
                <div class="info-button-group">
                  <button @click="navigateToRuntimeConfig(runtimeId)">
                    Config
                  </button>
                  <button @click="navigateToRuntimeLogs(runtimeId)">
                    Logs
                  </button>
                </div>
              </div>
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
  runtimeConfigs,
  getRuntimesForConfig,
  navigateToConfigEdit,
  navigateToRuntimeConfig,
  navigateToRuntimeLogs,
  createRuntimeFromConfig,
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

.runtime-info-panel {
  padding: 0.5rem;
}

.info-button-group {
  display: flex;
  gap: 0.5rem;
  justify-content: flex-start;
}

.info-button-group button {
  padding: 0.4rem 0.8rem;
  font-size: 0.85rem;
}
</style>
