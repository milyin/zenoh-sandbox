import { invoke } from '@tauri-apps/api/core';

// Import and re-export auto-generated types from Rust
import type { ZenohMode } from './generated/ZenohMode';
import type { ZenohConfigEdit } from './generated/ZenohConfigEdit';
import type { ZenohConfigJson } from './generated/ZenohConfigJson';

export type { ZenohMode, ZenohConfigEdit, ZenohConfigJson };

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
 * Get the JSON representation of a validated config for display/editing
 * @param configJson - Validated config
 * @returns JSON object
 */
export async function getZenohConfigJson(
  configJson: ZenohConfigJson
): Promise<Record<string, any>> {
  return await invoke<Record<string, any>>('zenoh_config_get_json', {
    config: configJson,
  });
}

/**
 * Create a new validated config with auto-assigned port
 * @param mode - Zenoh mode
 * @returns Tuple of [edit fields, validated config, assigned port]
 */
export async function createZenohConfigWithAutoPort(
  mode: ZenohMode
): Promise<[ZenohConfigEdit, ZenohConfigJson, number]> {
  return await invoke<[ZenohConfigEdit, ZenohConfigJson, number]>(
    'zenoh_config_create_with_auto_port',
    { mode }
  );
}
