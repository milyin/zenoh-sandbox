<template>
  <div class="zenoh-container">
    <!-- Main Operations Panel -->
    <div class="main-panel">
      <!-- Entity Controls -->
      <div class="entity-panel">
        <!-- Runtimes Section -->
        <Section
          title="Runtimes"
          icon="⚙️"
          section-class="runtimes-section"
        >
          <!-- New Runtime Entity -->
          <Entity
            title="New Runtime"
            :descr="newRuntimeConfig.websocket_port || 'Configure and start'"
            v-model:editsExpanded="newRuntimeEditsExpanded"
          >
            <template #actions>
              <button
                @click="createNewRuntime"
              >
                Start
              </button>
            </template>

            <template #edits>
              <ServerInput
                v-model="newRuntimeConfig.websocket_port"
                label="WebSocket Port"
                placeholder="e.g., 10000 or 127.0.0.1:10000"
              />
            </template>

            <!-- Active Runtimes as Sub-entities -->
            <template v-if="runtimes.length > 0" #sub-entities>
              <Entity
                v-for="runtimeId in runtimes"
                :key="runtimeId"
                :title="runtimeId"
                :descr="runtimeConfigs[runtimeId]?.websocket_port || 'Loading...'"
              >
                <template #actions>
                  <button
                    @click="cloneRuntime(runtimeId)"
                  >
                    Clone
                  </button>
                  <button
                    @click="stopRuntime(runtimeId)"
                  >
                    stop
                  </button>
                </template>

                <template #info>
                  <div class="runtime-info-panel">
                    <div class="info-button-group">
                      <button @click="showConfig(runtimeId)">
                        Config
                      </button>
                      <button @click="showLogs(runtimeId)">
                        Logs
                      </button>
                    </div>
                  </div>
                </template>
              </Entity>
            </template>
          </Entity>
        </Section>
      </div>

      <!-- Info Panel (placeholder for future use) -->
      <div class="log-panel">
        <Section
          title="Runtime Info"
          icon="📋"
          section-class="info-section"
        >
          <div class="info-content">
            <div v-if="runtimes.length === 0" class="empty-info">
              No runtimes running. Create a new runtime to get started.
            </div>
            <div v-else class="runtime-summary">
              <p><strong>Active Runtimes:</strong> {{ runtimes.length }}</p>
              <ul>
                <li v-for="runtimeId in runtimes" :key="runtimeId">
                  <strong>{{ runtimeId }}</strong>:
                  {{ runtimeConfigs[runtimeId]?.websocket_port || 'Loading...' }}
                </li>
              </ul>
            </div>
          </div>
        </Section>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, defineExpose } from 'vue';
import { invoke } from '@tauri-apps/api/core';

// Import components
import Section from './components/Section.vue';
import Entity from './components/Entity.vue';
import ServerInput from './components/ServerInput.vue';

// Import ZenohConfig types and functions
import {
  ZenohConfig,
  createDefaultZenohConfig,
  nextZenohConfig,
  DEFAULT_WEBSOCKET_PORT
} from './types/zenohConfig';

// Constants
const NEW_INSTANCE_ID = '__NEW_INSTANCE__';

const runtimes = ref<string[]>([]);
const selectedRuntime = ref<string | null>(NEW_INSTANCE_ID);
const newRuntimeConfig = ref<ZenohConfig>({ websocket_port: null });
const runtimeConfigs = reactive<Record<string, ZenohConfig>>({});
const newRuntimeEditsExpanded = ref(false);
const runtimeEditsExpanded = reactive<Record<string, boolean>>({});

// Set default configuration for new runtime
const setDefaultConfig = async () => {
  const defaultConfig = createDefaultZenohConfig();
  newRuntimeConfig.value = await nextZenohConfig(defaultConfig);
};

// Load runtimes on mount
const loadRuntimes = async () => {
  try {
    const list = await invoke<string[]>('zenoh_runtime_list');
    runtimes.value = list;

    // Load configs for all runtimes
    for (const runtimeId of list) {
      await loadConfig(runtimeId);
    }
  } catch (error) {
    console.error('Failed to load runtimes:', error);
  }
};

// Load configuration for a specific runtime
const loadConfig = async (runtimeId: string) => {
  try {
    const config = await invoke<ZenohConfig>('zenoh_runtime_config', { zid: runtimeId });
    runtimeConfigs[runtimeId] = { ...config };
  } catch (error) {
    console.error('Failed to load config:', error);
  }
};

// Clone runtime - copy config to new runtime and open edits
const cloneRuntime = (runtimeId: string) => {
  newRuntimeConfig.value = { ...runtimeConfigs[runtimeId] };
  newRuntimeEditsExpanded.value = true;
};

// Show config for an runtime (placeholder for future implementation)
const showConfig = (runtimeId: string) => {
  console.log('Show config for runtime:', runtimeId);
  // TODO: Implementation will be added in next step
};

// Show logs for an runtime (placeholder for future implementation)
const showLogs = (runtimeId: string) => {
  console.log('Show logs for runtime:', runtimeId);
  // TODO: Implementation will be added in next step
};

// Create new runtime with configured settings
const createNewRuntime = async () => {
  console.log('🚀 Starting runtime with config:', newRuntimeConfig.value);
  try {
    // Use the exact config from the dialog (user's responsibility to enter correct values)
    console.log('📞 Calling zenoh_runtime_start...');
    const newRuntimeId = await invoke<string>('zenoh_runtime_start', { config: newRuntimeConfig.value });
    console.log('✅ Runtime started successfully:', newRuntimeId);
    await loadRuntimes();

    // After successful creation, prepare next free port for next invocation
    newRuntimeConfig.value = await nextZenohConfig(newRuntimeConfig.value);

    // Optionally select the new runtime
    selectedRuntime.value = newRuntimeId;
  } catch (error) {
    console.error('❌ Failed to create runtime:', error);
    alert(`Failed to create runtime: ${error}`);
  }
};

// stop (delete) an runtime
const stopRuntime = async (runtimeId: string) => {
  try {
    await invoke('zenoh_runtime_stop', { zid: runtimeId });
    if (selectedRuntime.value === runtimeId) {
      selectedRuntime.value = NEW_INSTANCE_ID;
    }
    delete runtimeConfigs[runtimeId];
    delete runtimeEditsExpanded[runtimeId];
    await loadRuntimes();
  } catch (error) {
    console.error('Failed to stop runtime:', error);
    alert(`Failed to stop runtime: ${error}`);
  }
};

// Expose selectedRuntime for parent component to persist
defineExpose({
  selectedRuntime
});

// Initial load - load runtimes first, then set default config
(async () => {
  await loadRuntimes();
  await setDefaultConfig();
})();
</script>

<style scoped>
/* Essential 2-panel layout styles */
.zenoh-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
}

.main-panel {
  display: flex;
  flex: 1;
  overflow: hidden;
  gap: 0.5rem;
}

.entity-panel {
  width: 40%;
  overflow-y: auto;
}

.log-panel {
  width: 60%;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.info-section {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
}

.info-content {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
  padding: var(--spacing-content);
}

.empty-info {
  text-align: center;
  color: var(--log-neutral-color);
  padding: 2rem;
}

.runtime-summary {
  font-size: var(--font-size-normal);
}

.runtime-summary ul {
  margin-top: 1rem;
  margin-left: 1.5rem;
}

.runtime-summary li {
  margin-bottom: 0.5rem;
  font-family: monospace;
}

.runtime-info-panel {
  display: flex;
  flex-direction: column;
  gap: var(--size-md);
}

.info-button-group {
  display: flex;
  gap: var(--size-md);
  flex-wrap: wrap;
}

.info-button-group button {
  flex: 1;
  min-width: calc(var(--base-font-size) * 5);
}
</style>
