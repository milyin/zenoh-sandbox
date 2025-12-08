import { invoke } from '@tauri-apps/api/core';

/**
 * Zenoh mode type
 */
export type ZenohMode = "peer" | "router" | "client";

/**
 * Editable fields for Zenoh configuration
 * This represents only the fields that can be modified through the UI
 */
export interface ZenohConfigEdit {
  mode: ZenohMode;
}

/**
 * Validated Zenoh configuration JSON
 * This class is OPAQUE - it can only be created through Rust validation
 *
 * DO NOT attempt to construct this directly or copy its internal state.
 * All instances must come from Rust Tauri commands.
 */
export class ZenohConfigJson {
  // Private field that cannot be accessed or modified from outside
  // The actual structure is opaque - we don't know or care what it contains
  private readonly _opaque: unknown;

  /**
   * Private constructor - cannot be called from outside this class
   * Instances are created by deserializing Tauri command results
   */
  private constructor(opaque: unknown) {
    this._opaque = opaque;
  }

  /**
   * This method is used internally by Tauri's invoke serialization
   * It allows the object to be sent back to Rust
   */
  toJSON(): unknown {
    return this._opaque;
  }

  /**
   * Internal factory method for creating instances from Tauri responses
   * @internal
   */
  static _fromTauri(opaque: unknown): ZenohConfigJson {
    const instance = Object.create(ZenohConfigJson.prototype);
    instance._opaque = opaque;
    return instance;
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
  const result = await invoke<[ZenohConfigEdit, unknown]>(
    'verify_zenoh_config_json',
    { json }
  );

  return [result[0], ZenohConfigJson._fromTauri(result[1])];
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
  const result = await invoke<unknown>('apply_zenoh_config_edit', {
    configJson: configJson.toJSON(),
    edit,
  });

  return ZenohConfigJson._fromTauri(result);
}

/**
 * Get the JSON representation of a validated config for display/editing
 * @param configJson - Validated config
 * @returns JSON object
 */
export async function getZenohConfigJson(
  configJson: ZenohConfigJson
): Promise<Record<string, any>> {
  const result = await invoke<Record<string, any>>('zenoh_config_get_json', {
    config: configJson.toJSON(),
  });

  return result;
}

/**
 * Create a new validated config with auto-assigned port
 * @param mode - Zenoh mode
 * @returns Tuple of [edit fields, validated config, assigned port]
 */
export async function createZenohConfigWithAutoPort(
  mode: ZenohMode
): Promise<[ZenohConfigEdit, ZenohConfigJson, number]> {
  const result = await invoke<[ZenohConfigEdit, unknown, number]>(
    'zenoh_config_create_with_auto_port',
    { mode }
  );

  return [result[0], ZenohConfigJson._fromTauri(result[1]), result[2]];
}
