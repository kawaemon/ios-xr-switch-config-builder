import { it, expect } from "vitest";
import { wasm } from "./helpers";

it("requires description even for existing interfaces", () => {
  const baseConfig = `
interface FortyGigE0/0/0/46
  mru 9000
`.trim();

  const changeInput = `
vlan database
  vlan 400 name new

interface FortyGigE0/0/0/46
  switchport trunk allowed vlan add 400
`.trim();

  expect(() => wasm.generate_change_config(baseConfig, changeInput)).toThrow(
    /interface requires description: FortyGigE0\/0\/0\/46 \(line 4\)/,
  );
});
