<template>
  <div class="zenoh-container">
    <!-- Main Operations Panel -->
    <div class="main-panel">
      <!-- Entity Controls -->
      <div class="entity-panel">
        <!-- Instances Section -->
        <Section
          title="Instances"
          icon="⚙️"
          section-class="instances-section"
        >
          <!-- New Instance Entity -->
          <Entity
            title="New Instance"
            :descr="newInstanceConfig.websocket_port || 'Configure and start'"
            v-model:editsExpanded="newInstanceEditsExpanded"
          >
            <template #actions>
              <button
                @click="createNewInstance"
              >
                Start
              </button>
            </template>

            <template #edits>
              <ServerInput
                v-model="newInstanceConfig.websocket_port"
                label="WebSocket Port"
                placeholder="e.g., 10000 or 127.0.0.1:10000"
              />
            </template>

            <!-- Active Instances as Sub-entities -->
            <template v-if="instances.length > 0" #sub-entities>
              <Entity
                v-for="instanceId in instances"
                :key="instanceId"
                :title="instanceId"
                :descr="instanceConfigs[instanceId]?.websocket_port || 'Loading...'"
              >
                <template #actions>
                  <button
                    @click="cloneInstance(instanceId)"
                  >
                    Clone
                  </button>
                  <button
                    @click="stopInstance(instanceId)"
                  >
                    stop
                  </button>
                </template>

                <template #info>
                  <div class="instance-info-panel">
                    <div class="info-button-group">
                      <button @click="showConfig(instanceId)">
                        Config
                      </button>
                      <button @click="showLogs(instanceId)">
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
          title="Instance Info"
          icon="📋"
          section-class="info-section"
        >
          <div class="info-content">
            <div v-if="instances.length === 0" class="empty-info">
              No instances running. Create a new instance to get started.
            </div>
            <div v-else class="instance-summary">
              <p><strong>Active Instances:</strong> {{ instances.length }}</p>
              <ul>
                <li v-for="instanceId in instances" :key="instanceId">
                  <strong>{{ instanceId }}</strong>:
                  {{ instanceConfigs[instanceId]?.websocket_port || 'Loading...' }}
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

const instances = ref<string[]>([]);
const selectedInstance = ref<string | null>(NEW_INSTANCE_ID);
const newInstanceConfig = ref<ZenohConfig>({ websocket_port: null });
const instanceConfigs = reactive<Record<string, ZenohConfig>>({});
const newInstanceEditsExpanded = ref(false);
const instanceEditsExpanded = reactive<Record<string, boolean>>({});

// Get all currently used ports
const getUsedPorts = async (): Promise<number[]> => {
  const ports: number[] = [];

  for (const instanceId of instances.value) {
    try {
      const config = await invoke<ZenohConfig>('zenoh_instance_config', { zid: instanceId });
      if (config.websocket_port) {
        // Extract port number from formats like "10000" or "127.0.0.1:10000"
        const portMatch = config.websocket_port.match(/:(\d+)$|^(\d+)$/);
        if (portMatch) {
          const port = parseInt(portMatch[1] || portMatch[2]);
          if (!isNaN(port)) {
            ports.push(port);
          }
        }
      }
    } catch (error) {
      console.error(`Failed to get config for instance ${instanceId}:`, error);
    }
  }

  return ports;
};

// Set default configuration for new instance
const setDefaultConfig = async () => {
  const defaultConfig = createDefaultZenohConfig();
  const usedPorts = await getUsedPorts();
  newInstanceConfig.value = nextZenohConfig(defaultConfig, usedPorts);
};

// Load instances on mount
const loadInstances = async () => {
  try {
    const list = await invoke<string[]>('zenoh_instance_list');
    instances.value = list;

    // Load configs for all instances
    for (const instanceId of list) {
      await loadConfig(instanceId);
    }
  } catch (error) {
    console.error('Failed to load instances:', error);
  }
};

// Load configuration for a specific instance
const loadConfig = async (instanceId: string) => {
  try {
    const config = await invoke<ZenohConfig>('zenoh_instance_config', { zid: instanceId });
    instanceConfigs[instanceId] = { ...config };
  } catch (error) {
    console.error('Failed to load config:', error);
  }
};

// Clone instance - copy config to new instance and open edits
const cloneInstance = (instanceId: string) => {
  newInstanceConfig.value = { ...instanceConfigs[instanceId] };
  newInstanceEditsExpanded.value = true;
};

// Show config for an instance (placeholder for future implementation)
const showConfig = (instanceId: string) => {
  console.log('Show config for instance:', instanceId);
  // TODO: Implementation will be added in next step
};

// Show logs for an instance (placeholder for future implementation)
const showLogs = (instanceId: string) => {
  console.log('Show logs for instance:', instanceId);
  // TODO: Implementation will be added in next step
};

// Create new instance with configured settings
const createNewInstance = async () => {
  try {
    // Use the exact config from the dialog (user's responsibility to enter correct values)
    const newInstanceId = await invoke<string>('zenoh_instance_start', { config: newInstanceConfig.value });
    await loadInstances();

    // After successful creation, prepare next free port for next invocation
    const usedPorts = await getUsedPorts();
    newInstanceConfig.value = nextZenohConfig(newInstanceConfig.value, usedPorts);

    // Optionally select the new instance
    selectedInstance.value = newInstanceId;
  } catch (error) {
    console.error('Failed to create instance:', error);
    alert(`Failed to create instance: ${error}`);
  }
};

// stop (delete) an instance
const stopInstance = async (instanceId: string) => {
  try {
    await invoke('zenoh_instance_stop', { zid: instanceId });
    if (selectedInstance.value === instanceId) {
      selectedInstance.value = NEW_INSTANCE_ID;
    }
    delete instanceConfigs[instanceId];
    delete instanceEditsExpanded[instanceId];
    await loadInstances();
  } catch (error) {
    console.error('Failed to stop instance:', error);
    alert(`Failed to stop instance: ${error}`);
  }
};

// Expose selectedInstance for parent component to persist
defineExpose({
  selectedInstance
});

// Initial load - load instances first, then set default config
(async () => {
  await loadInstances();
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

.instance-summary {
  font-size: var(--font-size-normal);
}

.instance-summary ul {
  margin-top: 1rem;
  margin-left: 1.5rem;
}

.instance-summary li {
  margin-bottom: 0.5rem;
  font-family: monospace;
}

.instance-info-panel {
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
