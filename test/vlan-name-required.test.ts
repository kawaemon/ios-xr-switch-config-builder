import { it, expect } from "vitest";
import { wasm } from "./helpers";

it("rejects vlan without name", () => {
  const baseConfig = `
interface FortyGigE0/0/0/46
  description To:server1
`.trim();

  const changeInput = `
vlan database
  vlan 300

interface FortyGigE0/0/0/46
  switchport trunk allowed vlan add 300
`.trim();

  expect(() => wasm.generate_change_config(baseConfig, changeInput)).toThrow(
    /VLAN名は必須です|vlan name is required/,
  );
});
