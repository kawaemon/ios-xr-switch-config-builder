import { it, expect } from "vitest";
import { wasm } from "./helpers";

it("allows interface blocks without statements", () => {
  const baseConfig = `
interface FortyGigE0/0/0/46
  description To:server1
`.trim();

  const changeInput = `
vlan database
  vlan 300 name test

interface FortyGigE0/0/0/46
`.trim();

  const result = wasm.generate_change_config(baseConfig, changeInput);

  expect(result.changeOutput).toBe("");
});
