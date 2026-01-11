import wasmInit, * as wasmModule from "./pkg/ncs_wasm.js";

export type NcsWasmModule = typeof wasmModule;

let initialized = false;

export async function initWasm(): Promise<NcsWasmModule> {
  if (!initialized) {
    await wasmInit();
    initialized = true;
  }
  return wasmModule;
}
