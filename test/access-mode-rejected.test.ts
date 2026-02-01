import { it, expect } from "vitest";
import { wasm } from "./helpers";

it("rejects access switchport mode", () => {
  const baseConfig = `
interface FortyGigE0/0/0/46
  description To:server1
`.trim();

  const changeInput = `
vlan database
  vlan 300 name test

interface FortyGigE0/0/0/46
  description To:server1
  switchport mode access
  switchport access vlan 300
`.trim();

  expect(() => wasm.generate_change_config(baseConfig, changeInput)).toThrow(
    /switchport mode access はサポートされていません/,
  );
});
