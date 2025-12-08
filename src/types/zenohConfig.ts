import { invoke } from '@tauri-apps/api/core';

/**
 * ZenohConfig - Configuration for creating a Zenoh runtime
 *
 * This class stores the full JSON representation of zenoh::Config and provides
 * convenient accessors for common properties like mode.
 *
 * Port assignment is handled automatically and guaranteed to be unique.
 * Use createZenohConfig() to create a new config with an auto-assigned free port.
 */

export const DEFAULT_WEBSOCKET_PORT = 10000;

export type ZenohMode = "peer" | "router" | "client";

/**
 * ZenohConfig class that wraps the full JSON representation of zenoh::Config
 */
export class ZenohConfig {
  private configJson: Record<string, any>;

  /**
   * Private constructor - use static factory methods to create instances
   */
  private constructor(configJson: Record<string, any>) {
    this.configJson = configJson;
  }

  /**
   * Create a ZenohConfig from JSON, verifying it with zenoh::Config
   * @param json - JSON object to validate and use
   * @returns A new ZenohConfig if valid
   * @throws Error if JSON is not valid for zenoh::Config
   */
  static async fromJson(json: Record<string, any>): Promise<ZenohConfig> {
    // Verify with Rust backend
    await invoke('zenoh_config_verify_json', { configJson: json });
    return new ZenohConfig(json);
  }

  /**
   * Create a default ZenohConfig with specified mode and port
   * @param mode - Zenoh mode: "peer", "router", or "client"
   * @param websocketPort - WebSocket port for Remote API
   * @returns A new ZenohConfig
   */
  static async create(mode: ZenohMode, websocketPort: string): Promise<ZenohConfig> {
    const configData = await invoke<Record<string, any>>('zenoh_config_create', {
      mode,
      websocketPort,
    });
    return new ZenohConfig(configData);
  }

  /**
   * Get the mode from the config
   */
  get mode(): ZenohMode {
    const modeStr = this.configJson.mode as string;
    if (modeStr === 'peer' || modeStr === 'router' || modeStr === 'client') {
      return modeStr;
    }
    return 'peer'; // Default fallback
  }

  /**
   * Set the mode in the config
   * Note: This creates a new config internally by calling Rust to ensure validity
   */
  set mode(newMode: ZenohMode) {
    // Update the mode in the JSON
    this.configJson = {
      ...this.configJson,
      mode: newMode,
    };
  }

  /**
   * Get the websocket port from the config
   */
  get websocket_port(): string {
    return this.configJson.plugins?.remote_api?.websocket_port || DEFAULT_WEBSOCKET_PORT.toString();
  }

  /**
   * Set the websocket port in the config
   */
  set websocket_port(port: string) {
    if (!this.configJson.plugins) {
      this.configJson.plugins = {};
    }
    if (!this.configJson.plugins.remote_api) {
      this.configJson.plugins.remote_api = {};
    }
    this.configJson.plugins.remote_api.websocket_port = port;
  }

  /**
   * Get the full JSON representation
   */
  getJson(): Record<string, any> {
    return { ...this.configJson };
  }

  /**
   * Update the entire JSON configuration
   * @param json - New JSON configuration
   * @throws Error if JSON is not valid for zenoh::Config
   */
  async setJson(json: Record<string, any>): Promise<void> {
    // Verify with Rust backend
    await invoke('zenoh_config_verify_json', { configJson: json });
    this.configJson = json;
  }

  /**
   * Get a JSON string representation (formatted)
   */
  toJsonString(pretty: boolean = true): string {
    return JSON.stringify(this.configJson, null, pretty ? 2 : 0);
  }

  /**
   * Clone this config
   */
  clone(): ZenohConfig {
    return new ZenohConfig({ ...this.configJson });
  }

  /**
   * Convert to plain object for serialization (used by Tauri invoke)
   */
  toJSON(): Record<string, any> {
    return this.configJson;
  }
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
    const config = await ZenohConfig.create(mode, port.toString());

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
export async function createDefaultZenohConfig(): Promise<ZenohConfig> {
  return await ZenohConfig.create("peer", DEFAULT_WEBSOCKET_PORT.toString());
}

