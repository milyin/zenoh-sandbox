/**
 * ZenohConfig - Configuration for creating a Zenoh instance
 */

import { invoke } from '@tauri-apps/api/core';

export const DEFAULT_WEBSOCKET_PORT = 10000;

export interface ZenohConfig {
  websocket_port: string | null;
}

/**
 * Creates a default ZenohConfig with initial values
 */
export function createDefaultZenohConfig(): ZenohConfig {
  return {
    websocket_port: DEFAULT_WEBSOCKET_PORT.toString()
  };
}

/**
 * Extracts port number from websocket_port string
 * Handles formats like "10000" or "127.0.0.1:10000"
 */
export function extractPort(websocketPort: string | null): number | null {
  if (!websocketPort) {
    return null;
  }

  const portMatch = websocketPort.match(/:(\d+)$|^(\d+)$/);
  if (portMatch) {
    const port = parseInt(portMatch[1] || portMatch[2]);
    return isNaN(port) ? null : port;
  }

  return null;
}

/**
 * Gets all used ports from an array of ZenohConfigs
 *
 * @param configs - Array of ZenohConfig objects
 * @returns Array of port numbers that are currently in use
 */
export function getUsedPorts(configs: ZenohConfig[]): number[] {
  const ports: number[] = [];

  for (const config of configs) {
    if (config.websocket_port) {
      const port = extractPort(config.websocket_port);
      if (port !== null) {
        ports.push(port);
      }
    }
  }

  return ports;
}

/**
 * Updates a ZenohConfig to values suitable for a new instance
 * Finds the next available port if the current port is in use
 * Fetches current instance configs from Rust directly
 *
 * @param config - The base configuration to update
 * @returns A new ZenohConfig with updated values
 */
export async function nextZenohConfig(
  config: ZenohConfig
): Promise<ZenohConfig> {
  // Get all current instances from Rust
  let usedPorts: number[] = [];

  try {
    const instances = await invoke<string[]>('zenoh_instance_list');

    // Get config for each instance and extract ports
    const configs: ZenohConfig[] = [];
    for (const instanceId of instances) {
      try {
        const instanceConfig = await invoke<ZenohConfig>('zenoh_instance_config', { zid: instanceId });
        configs.push(instanceConfig);
      } catch (error) {
        console.error(`Failed to get config for instance ${instanceId}:`, error);
      }
    }

    usedPorts = getUsedPorts(configs);
  } catch (error) {
    console.error('Failed to get instance list:', error);
  }

  // Start with the port from config, or default
  let port = extractPort(config.websocket_port) ?? DEFAULT_WEBSOCKET_PORT;

  // Find next unused port
  while (usedPorts.includes(port)) {
    port++;
  }

  // Preserve the format if it was IP:PORT
  const originalPort = config.websocket_port;
  let newPortString: string;

  if (originalPort && originalPort.includes(':')) {
    // Format: IP:PORT
    const ipMatch = originalPort.match(/^(.+):(\d+)$/);
    if (ipMatch) {
      newPortString = `${ipMatch[1]}:${port}`;
    } else {
      newPortString = port.toString();
    }
  } else {
    // Format: PORT only
    newPortString = port.toString();
  }

  return {
    ...config,
    websocket_port: newPortString
  };
}
