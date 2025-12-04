<template>
  <div class="instances-container">
    <!-- Left Panel: Instance List -->
    <div class="instances-list-panel">
      <div class="instances-list">
        <!-- New Instance Row -->
        <div
          class="instance-item new-instance"
          :class="{ selected: selectedInstance === NEW_INSTANCE_ID }"
          @click="selectNewInstance"
        >
          <div class="instance-info">
            <span class="instance-label">New Instance</span>
          </div>
          <button @click.stop="createNewInstance" class="action-button invoke-button">
            Invoke
          </button>
        </div>

        <!-- Existing Instances -->
        <div
          v-for="instanceId in instances"
          :key="instanceId"
          class="instance-item"
          :class="{ selected: selectedInstance === instanceId }"
          @click="selectInstance(instanceId)"
        >
          <div class="instance-info">
            <span class="instance-id">{{ instanceId }}</span>
          </div>
          <button
            @click.stop="dismissInstance(instanceId)"
            class="action-button dismiss-button"
          >
            Dismiss
          </button>
        </div>

        <div v-if="instances.length === 0" class="empty-list">
          No instances created yet. Click "Invoke" to create one.
        </div>
      </div>
    </div>

    <!-- Right Panel: Configuration Editor -->
    <div class="config-panel">
      <!-- Config for New Instance -->
      <div v-if="selectedInstance === NEW_INSTANCE_ID" class="config-editor">
        <h3>Configuration for New Instance</h3>

        <div class="config-form">
          <div class="form-group">
            <label for="websocket-port">WebSocket Port:</label>
            <input
              id="websocket-port"
              v-model="newInstanceConfig.websocket_port"
              type="text"
              placeholder="e.g., 10000 or 127.0.0.1:10000"
              class="config-input"
            />
            <small class="help-text">
              Port number or &lt;local_ip&gt;:&lt;port_number&gt; format
            </small>
          </div>

          <div class="form-actions">
            <button @click="setDefaultConfig" class="action-button default-button">
              Default
            </button>
          </div>
        </div>
      </div>

      <!-- Config for Running Instance -->
      <div v-else-if="selectedInstance" class="config-editor">
        <h3>Configuration for {{ selectedInstance }}</h3>

        <div class="config-form">
          <div class="form-group">
            <label for="websocket-port-readonly">WebSocket Port:</label>
            <input
              id="websocket-port-readonly"
              v-model="currentConfig.websocket_port"
              type="text"
              placeholder="e.g., 10000 or 127.0.0.1:10000"
              class="config-input"
              readonly
            />
            <small class="help-text">
              Port number or &lt;local_ip&gt;:&lt;port_number&gt; format (read-only)
            </small>
          </div>

          <div class="form-actions">
            <button @click="copyToNewInstance" class="action-button copy-button">
              Copy
            </button>
          </div>
        </div>
      </div>

      <!-- No Selection -->
      <div v-else class="no-selection">
        <p>Select an instance to view its configuration</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, defineExpose } from 'vue';
import { invoke } from '@tauri-apps/api/core';

// Constants
const DEFAULT_WEBSOCKET_PORT = 10000;
const NEW_INSTANCE_ID = '__NEW_INSTANCE__';

interface ZenohConfig {
  websocket_port: string | null;
}

const instances = ref<string[]>([]);
const selectedInstance = ref<string | null>(NEW_INSTANCE_ID);
const currentConfig = ref<ZenohConfig>({ websocket_port: null });
const newInstanceConfig = ref<ZenohConfig>({ websocket_port: null });

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

// Find first unused port starting from DEFAULT_WEBSOCKET_PORT
const findFirstUnusedPort = async (): Promise<number> => {
  const usedPorts = await getUsedPorts();
  let port = DEFAULT_WEBSOCKET_PORT;

  while (usedPorts.includes(port)) {
    port++;
  }

  return port;
};

// Set default configuration
const setDefaultConfig = async () => {
  const port = await findFirstUnusedPort();
  newInstanceConfig.value = {
    websocket_port: port.toString()
  };
};

// Load instances on mount
const loadInstances = async () => {
  try {
    const list = await invoke<string[]>('zenoh_instance_list');
    instances.value = list;
  } catch (error) {
    console.error('Failed to load instances:', error);
  }
};

// Load configuration for selected instance
const loadConfig = async (instanceId: string) => {
  try {
    const config = await invoke<ZenohConfig>('zenoh_instance_config', { zid: instanceId });
    currentConfig.value = { ...config };
  } catch (error) {
    console.error('Failed to load config:', error);
  }
};

// Select an existing instance
const selectInstance = (instanceId: string) => {
  selectedInstance.value = instanceId;
  loadConfig(instanceId);
};

// Select the new instance placeholder
const selectNewInstance = () => {
  selectedInstance.value = NEW_INSTANCE_ID;
};

// Copy current instance config to new instance config
const copyToNewInstance = () => {
  newInstanceConfig.value = { ...currentConfig.value };
  selectedInstance.value = NEW_INSTANCE_ID;
};

// Create new instance with configured settings
const createNewInstance = async () => {
  try {
    const config: ZenohConfig = {
      websocket_port: newInstanceConfig.value.websocket_port || (await findFirstUnusedPort()).toString()
    };
    const newInstanceId = await invoke<string>('zenoh_instance_invoke', { config });
    await loadInstances();
    selectInstance(newInstanceId);
  } catch (error) {
    console.error('Failed to create instance:', error);
    alert(`Failed to create instance: ${error}`);
  }
};

// Dismiss (delete) an instance
const dismissInstance = async (instanceId: string) => {
  try {
    await invoke('zenoh_instance_dismiss', { zid: instanceId });
    if (selectedInstance.value === instanceId) {
      selectedInstance.value = NEW_INSTANCE_ID;
    }
    await loadInstances();
  } catch (error) {
    console.error('Failed to dismiss instance:', error);
    alert(`Failed to dismiss instance: ${error}`);
  }
};

// Expose selectedInstance for parent component to persist
defineExpose({
  selectedInstance
});

// Initial load
loadInstances();
setDefaultConfig();
</script>

<style scoped>
.instances-container {
  display: flex;
  height: 100%;
  gap: 1rem;
  padding: 1rem;
}

.instances-list-panel {
  width: 40%;
  display: flex;
  flex-direction: column;
}

.instances-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  overflow-y: auto;
}

.instance-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem;
  border: 1px solid var(--border-color, #ccc);
  border-radius: 4px;
  background: var(--bg-color, #fff);
  cursor: pointer;
  transition: all 0.2s;
}

.instance-item:hover {
  background: var(--hover-bg-color, #f5f5f5);
}

.instance-item.selected {
  border-color: var(--primary-color, #007bff);
  background: var(--selected-bg-color, #e7f3ff);
}

.instance-item.new-instance {
  border-style: dashed;
  cursor: default;
}

.instance-item.new-instance:hover {
  background: var(--bg-color, #fff);
}

.instance-info {
  flex: 1;
}

.instance-label {
  font-weight: bold;
  color: var(--text-color, #333);
}

.instance-id {
  font-family: monospace;
  color: var(--text-color, #333);
}

.action-button {
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-weight: 500;
  transition: all 0.2s;
}

.invoke-button {
  background: var(--success-color, #28a745);
  color: white;
}

.invoke-button:hover {
  background: var(--success-hover-color, #218838);
}

.dismiss-button {
  background: var(--danger-color, #dc3545);
  color: white;
}

.dismiss-button:hover {
  background: var(--danger-hover-color, #c82333);
}

.empty-list {
  padding: 2rem;
  text-align: center;
  color: var(--muted-color, #6c757d);
}

.config-panel {
  width: 60%;
  padding: 1rem;
  border: 1px solid var(--border-color, #ccc);
  border-radius: 4px;
  background: var(--bg-color, #fff);
}

.config-editor h3 {
  margin-top: 0;
  margin-bottom: 1.5rem;
  color: var(--text-color, #333);
}

.config-form {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.form-group label {
  font-weight: 500;
  color: var(--text-color, #333);
}

.config-input {
  padding: 0.5rem;
  border: 1px solid var(--border-color, #ccc);
  border-radius: 4px;
  font-size: 1rem;
  font-family: monospace;
}

.config-input:focus {
  outline: none;
  border-color: var(--primary-color, #007bff);
}

.help-text {
  color: var(--muted-color, #6c757d);
  font-size: 0.875rem;
}

.form-actions {
  display: flex;
  gap: 0.5rem;
  margin-top: 1rem;
}

.copy-button {
  background: var(--primary-color, #007bff);
  color: white;
}

.copy-button:hover {
  background: var(--primary-hover-color, #0056b3);
}

.default-button {
  background: var(--secondary-color, #6c757d);
  color: white;
}

.default-button:hover {
  background: var(--secondary-hover-color, #545b62);
}

.no-selection {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--muted-color, #6c757d);
}
</style>
