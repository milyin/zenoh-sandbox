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
}

/**
 * Verify raw JSON and get both editable fields and validated config
 * @param json - Raw JSON object to validate
 * @returns Tuple of [edit fields, validated config]
 * @throws Error if JSON is invalid
 */
export async function verifyZenohConfigJson(
  json: Record<string, any>
): Promise<[ZenohConfigEdit, ZenohConfigJson]> {
  return await invoke<[ZenohConfigEdit, ZenohConfigJson]>(
    'verify_zenoh_config_json',
    { json }
  );
}

/**
 * Apply editable fields to a validated config
 * @param configJson - Validated config to modify
 * @param edit - Editable fields to apply
 * @returns New validated config with changes applied
 */
export async function applyZenohConfigEdit(
  configJson: ZenohConfigJson,
  edit: ZenohConfigEdit
): Promise<ZenohConfigJson> {
  return await invoke<ZenohConfigJson>('apply_zenoh_config_edit', {
    configJson,
    edit,
  });
}

/**
 * Create a new validated config from edit fields with auto-assigned port
 * @param edit - Editable fields to apply
 * @returns Tuple of [validated config, assigned port]
 */
export async function createZenohConfig(
  edit: ZenohConfigEdit
): Promise<ZenohConfigJson> {
  return await invoke<ZenohConfigJson>(
    'create_zenoh_config',
    { edit }
  );
}
