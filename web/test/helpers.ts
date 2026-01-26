import * as wasmModule from "../src/wasm/pkg/ncs_wasm";

export type WasmModule = typeof wasmModule & {
  generate_change_config: (
    baseConfig: string,
    changeInput: string,
  ) => { changeOutput: string };
};

export const wasm = wasmModule as WasmModule;

export function nodeToJson(node: unknown): unknown {
  const n = node as { type: string; asBlock: unknown; asStmt: unknown };
  if (n.type === "block") {
    const block = n.asBlock as
      | { name: string; stmts: unknown[] }
      | null
      | undefined;
    if (block) {
      return {
        type: "block",
        name: block.name,
        stmts: block.stmts.map((s) => nodeToJson(s)),
      };
    }
  } else if (n.type === "stmt") {
    const stmt = n.asStmt as { stmt: string } | null | undefined;
    if (stmt) {
      return {
        type: "stmt",
        stmt: stmt.stmt,
      };
    }
  }
  return n;
}
