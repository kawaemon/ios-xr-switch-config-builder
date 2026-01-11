import * as wasmModule from "./pkg/ncs_wasm.js";

export type NcsWasmModule = typeof wasmModule;

let initialized = false;

export async function initWasm(): Promise<NcsWasmModule> {
  if (!initialized) {
    initialized = true;
  }
  return wasmModule;
}
