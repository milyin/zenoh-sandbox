import { invoke } from '@tauri-apps/api/core';

// Import and re-export auto-generated types from Rust
import type { ZenohMode } from './generated/ZenohMode';
import type { ZenohConfigEdit } from './generated/ZenohConfigEdit';
import type { ZenohConfigJson } from './generated/ZenohConfigJson';

export type { ZenohMode, ZenohConfigEdit, ZenohConfigJson };

/**
 * Configuration entry combines editable fields with validated config
 */
export class ZenohConfig {
  edit: ZenohConfigEdit;
  configJson: ZenohConfigJson;

  constructor(edit: ZenohConfigEdit, configJson: ZenohConfigJson) {
    this.edit = edit;
    this.configJson = configJson;
  }

  /**
   * Get the websocket port from the config JSON
   * Extracts from plugins.remote_api.websocket_port path
   */
  get websocket_port(): number | undefined {
    const portStr = this.configJson?.plugins?.remote_api?.websocket_port;
    if (typeof portStr === 'string') {
      const port = parseInt(portStr, 10);
      return isNaN(port) ? undefined : port;
    }
    return undefined;
  }

  /**
   * Get the mode from the config JSON
   */
  get mode(): string {
    return this.configJson?.mode || 'peer';
  }
}

/**
 * Validate JSON5 string and get validated config
 * @param content - JSON5 string to validate
 * @returns Validated config JSON
 * @throws Error if JSON5 is invalid
 */
export async function validateConfigJson5(
  content: string
): Promise<ZenohConfigJson> {
  return await invoke<ZenohConfigJson>('validate_config_json5', { content });
}

/**
 * Get the default app configuration with plugins as JSON string
 * Used as baseline for difference highlighting
 * @returns JSON string of default config
 */
export async function getDefaultConfigJson(): Promise<string> {
  return await invoke<string>('get_default_config_json');
}

/**
 * Compute the difference between two JSON configurations
 * @param base - Base configuration to compare against
 * @param modified - Modified configuration
 * @returns JSON object containing only fields that differ from base
 */
export async function computeConfigDiff(
  base: ZenohConfigJson,
  modified: ZenohConfigJson
): Promise<Record<string, any>> {
  return await invoke<Record<string, any>>('compute_config_diff', {
    base,
    modified,
  });
}

/**
 * Create a new validated config from edit content with auto-assigned port
 * @param edit - Edit object with JSON5 content
 * @returns Tuple of [updated edit with port, validated config]
 */
export async function createZenohConfig(
  edit: ZenohConfigEdit
): Promise<[ZenohConfigEdit, ZenohConfigJson]> {
  return await invoke<[ZenohConfigEdit, ZenohConfigJson]>(
    'create_zenoh_config',
    { edit }
  );
}
