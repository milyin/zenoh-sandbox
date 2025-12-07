/**
 * ZenohConfig - Configuration for creating a Zenoh runtime
 *
 * Port assignment is handled automatically and guaranteed to be unique.
 * Use createZenohConfig() to create a new config with an auto-assigned free port.
 */

import { invoke } from '@tauri-apps/api/core';

export const DEFAULT_WEBSOCKET_PORT = 10000;

export type ZenohMode = "peer" | "router" | "client";

export interface ZenohConfig {
  mode: ZenohMode;
  websocket_port: string;
}

/**
 * Track all config entries that have been created but not yet used to start a runtime.
 * This prevents assigning the same port to multiple pending configs.
 */
const pendingConfigs = new Set<ZenohConfig>();

/**
 * Extracts port number from websocket_port string
 * Handles formats like "10000" or "127.0.0.1:10000"
 */
export function extractPort(websocketPort: string | null): number | null {
  if (!websocketPort) return null;
  const portMatch = websocketPort.match(/:(\d+)$|^(\d+)$/);
  if (portMatch) {
    const port = parseInt(portMatch[1] || portMatch[2]);
    return isNaN(port) ? null : port;
  }
  return null;
}

/**
 * Gets all ports currently in use (from running runtimes)
 */
async function getUsedPorts(): Promise<number[]> {
  const ports: number[] = [];

  try {
    const runtimes = await invoke<string[]>('zenoh_runtime_list');

    for (const runtimeId of runtimes) {
      try {
        const runtimeConfig = await invoke<ZenohConfig>('zenoh_runtime_config', { zid: runtimeId });
        const port = extractPort(runtimeConfig.websocket_port);
        if (port !== null) {
          ports.push(port);
        }
      } catch (error) {
        console.error(`Failed to get config for runtime ${runtimeId}:`, error);
      }
    }
  } catch (error) {
    console.error('Failed to get runtime list:', error);
  }

  return ports;
}

/**
 * Gets all ports from pending config entries
 */
function getPendingPorts(): number[] {
  const ports: number[] = [];
  for (const config of pendingConfigs) {
    const port = extractPort(config.websocket_port);
    if (port !== null) {
      ports.push(port);
    }
  }
  return ports;
}

/**
 * Finds the next available port, considering both running runtimes and pending configs
 */
function findFreePort(startPort: number, usedPorts: number[], pendingPorts: number[]): number {
  let port = startPort;
  const allUsed = new Set([...usedPorts, ...pendingPorts]);

  while (allUsed.has(port)) {
    port++;
  }

  return port;
}

/**
 * Creates a new ZenohConfig with an automatically assigned free port.
 * The port is guaranteed to be unique - no other ZenohConfig created with this
 * function will have the same port, even when called multiple times quickly.
 *
 * The config is automatically tracked until it's cleaned up with cleanupConfig().
 *
 * @param mode - Zenoh mode: "peer", "router", or "client" (default: "peer")
 * @returns A new ZenohConfig with a guaranteed unique port
 */
export async function createZenohConfig(mode: ZenohMode = "peer"): Promise<ZenohConfig> {
  // Clean up pending configs that are now running runtimes
  await cleanupRunningConfigs();

  // Get all currently used and pending ports
  const usedPorts = await getUsedPorts();
  const pendingPorts = getPendingPorts();

  // Find a free port
  const port = findFreePort(DEFAULT_WEBSOCKET_PORT, usedPorts, pendingPorts);

  // Create the config
  const config: ZenohConfig = {
    mode,
    websocket_port: port.toString()
  };

  // Track this config as pending (will be removed when runtime starts or config is replaced)
  pendingConfigs.add(config);

  return config;
}

/**
 * Cleans up a config that is no longer needed (runtime started or config replaced).
 * This allows the port to be reused by future configs.
 *
 * This is called automatically when a config is replaced in the UI.
 *
 * @param config - The config to clean up
 */
export function cleanupConfig(config: ZenohConfig): void {
  pendingConfigs.delete(config);
}

/**
 * Removes pending configs whose ports are now in use by running runtimes.
 * This prevents pendingConfigs from growing indefinitely.
 *
 * Called automatically during port assignment.
 */
async function cleanupRunningConfigs(): Promise<void> {
  const usedPorts = new Set(await getUsedPorts());

  for (const config of pendingConfigs) {
    const port = extractPort(config.websocket_port);
    if (port !== null && usedPorts.has(port)) {
      // This port is now in a running runtime, remove from pending
      pendingConfigs.delete(config);
    }
  }
}

/**
 * Creates a default ZenohConfig (for backward compatibility)
 * Note: This does not guarantee unique ports. Use createZenohConfig() instead.
 *
 * @deprecated Use createZenohConfig() for automatic port assignment
 */
export function createDefaultZenohConfig(): ZenohConfig {
  return {
    mode: "peer",
    websocket_port: DEFAULT_WEBSOCKET_PORT.toString()
  };
}

/**
 * Auto-assigns a free port for a config (for backward compatibility)
 * Note: This is async and may have race conditions. Use createZenohConfig() instead.
 *
 * @deprecated Use createZenohConfig() for guaranteed unique ports
 */
export async function assignFreePort(config: ZenohConfig): Promise<ZenohConfig> {
  const usedPorts = await getUsedPorts();
  const pendingPorts = getPendingPorts();
  const port = findFreePort(extractPort(config.websocket_port) ?? DEFAULT_WEBSOCKET_PORT, usedPorts, pendingPorts);

  return {
    ...config,
    websocket_port: port.toString()
  };
}
