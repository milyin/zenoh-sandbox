/**
 * ZenohConfig - Configuration for creating a Zenoh runtime
 *
 * Port assignment is handled automatically and guaranteed to be unique.
 * Use createZenohConfig() to create a new config with an auto-assigned free port.
 */

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
 * Mutex to ensure only one config is created at a time.
 * This prevents race conditions when creating multiple configs quickly.
 */
let configCreationLock: Promise<void> | null = null;

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
 * Gets all ports currently in use (from a provided runtime configs map)
 * @param runtimeConfigs - Map of runtime IDs to their configs (from frontend state)
 */
function getUsedPortsFromConfigs(runtimeConfigs: Record<string, ZenohConfig>): number[] {
  const ports: number[] = [];

  for (const config of Object.values(runtimeConfigs)) {
    const port = extractPort(config.websocket_port);
    if (port !== null) {
      ports.push(port);
    }
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
 * @param runtimeConfigs - Current runtime configs from frontend state (to avoid backend queries)
 * @returns A new ZenohConfig with a guaranteed unique port
 */
export async function createZenohConfig(
  mode: ZenohMode = "peer",
  runtimeConfigs: Record<string, ZenohConfig> = {}
): Promise<ZenohConfig> {
  // Wait for any in-progress config creation to complete
  while (configCreationLock) {
    await configCreationLock;
  }

  // Create a new lock for this operation
  let resolveLock: () => void;
  configCreationLock = new Promise(resolve => {
    resolveLock = resolve;
  });

  try {
    // Get all currently used and pending ports
    // Use frontend state instead of querying backend to avoid circular dependency
    const usedPorts = getUsedPortsFromConfigs(runtimeConfigs);
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
  } finally {
    // Release the lock
    configCreationLock = null;
    resolveLock!();
  }
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

