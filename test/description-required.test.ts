import { it, expect } from "vitest";
import { wasm } from "./helpers";

it("requires description for new interfaces", () => {
  const baseConfig = "".trim();

  const changeInput = `
vlan database
  vlan 300 name test

interface FortyGigE0/0/0/46
  switchport mode trunk
  switchport trunk allowed vlan add 300
`.trim();

  expect(() => wasm.generate_change_config(baseConfig, changeInput)).toThrow(
    /インターフェイス.*にはdescriptionが必要です/,
  );
});
