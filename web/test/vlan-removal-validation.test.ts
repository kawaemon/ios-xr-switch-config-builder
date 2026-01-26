import { it, expect } from "vitest";
import { wasm } from "./helpers";

it("rejects removal of non-existent VLAN", () => {
  const baseConfig = `
interface FortyGigE0/0/0/46
  description To:server1
interface FortyGigE0/0/0/46.300 l2transport
  description test,To:server1
  encapsulation dot1q 300
  rewrite ingress tag pop 1 symmetric

l2vpn
  bridge group VLAN
    bridge-domain VLAN300
      description test
      interface FortyGigE0/0/0/46.300
`.trim();

  const changeInput = `
interface FortyGigE0/0/0/46
  switchport trunk allowed vlan remove 100
`.trim();

  expect(() => wasm.generate_change_config(baseConfig, changeInput)).toThrow(
    /cannot remove VLAN 100 from interface FortyGigE0\/0\/0\/46: VLAN not present in base config \(line 2\)/,
  );
});
