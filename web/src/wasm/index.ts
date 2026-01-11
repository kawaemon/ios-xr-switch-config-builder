export type NcsWasmModule = typeof import("./pkg/ncs_wasm.js");

export async function initWasm(): Promise<NcsWasmModule> {
  const module = await import("./pkg/ncs_wasm.js");
  await module.default();
  return module;
}
