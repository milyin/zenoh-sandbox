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
 * Track ports that are currently reserved (either running or pending)
 * This prevents race conditions when creating multiple configs quickly
 */
const reservedPorts = new Set<number>();

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
 * Finds the next available port, considering both running runtimes and reserved ports
 */
function findFreePort(startPort: number, usedPorts: number[]): number {
  let port = startPort;
  const allUsed = new Set([...usedPorts, ...reservedPorts]);

  while (allUsed.has(port)) {
    port++;
  }

  return port;
}

/**
 * Creates a new ZenohConfig with an automatically assigned free port.
 * The port is guaranteed to be unique - no other ZenohConfig created with this
 * function will have the same port.
 *
 * @param mode - Zenoh mode: "peer", "router", or "client" (default: "peer")
 * @returns A new ZenohConfig with a guaranteed unique port
 */
export async function createZenohConfig(mode: ZenohMode = "peer"): Promise<ZenohConfig> {
  // Get all currently used ports
  const usedPorts = await getUsedPorts();

  // Find a free port
  const port = findFreePort(DEFAULT_WEBSOCKET_PORT, usedPorts);

  // Reserve this port immediately to prevent race conditions
  reservedPorts.add(port);

  return {
    mode,
    websocket_port: port.toString()
  };
}

/**
 * Releases a port reservation when a runtime is successfully started or fails to start.
 * This allows the port to be reused if needed.
 *
 * @param config - The config whose port should be released
 */
export function releasePort(config: ZenohConfig): void {
  const port = extractPort(config.websocket_port);
  if (port !== null) {
    reservedPorts.delete(port);
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
  const port = findFreePort(extractPort(config.websocket_port) ?? DEFAULT_WEBSOCKET_PORT, usedPorts);

  return {
    ...config,
    websocket_port: port.toString()
  };
}
