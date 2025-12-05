/**
 * ZenohConfig - Configuration for creating a Zenoh instance
 */

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
function extractPort(websocketPort: string | null): number | null {
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
 * Updates a ZenohConfig to values suitable for a new instance
 * Finds the next available port if the current port is in use
 *
 * @param config - The base configuration to update
 * @param usedPorts - Array of currently used port numbers
 * @returns A new ZenohConfig with updated values
 */
export function nextZenohConfig(
  config: ZenohConfig,
  usedPorts: number[]
): ZenohConfig {
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
