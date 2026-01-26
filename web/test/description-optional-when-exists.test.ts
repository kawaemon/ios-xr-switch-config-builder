import { it, expect } from "vitest";
import { wasm } from "./helpers";

it("does not require description when interface already has one in base config", () => {
  const baseConfig = `
interface FortyGigE0/0/0/46
  description To:server1
  mru 9000
interface FortyGigE0/0/0/46.300 l2transport
  description old,To:server1
  encapsulation dot1q 300
  rewrite ingress tag pop 1 symmetric

l2vpn
  bridge group VLAN
    bridge-domain VLAN300
      description old
      interface FortyGigE0/0/0/46.300
`.trim();

  const changeInput = `
vlan database
  vlan 400 name new

interface FortyGigE0/0/0/46
  switchport trunk allowed vlan add 400
`.trim();

  // Should succeed without description in changeInput because base config has one
  const result = wasm.generate_change_config(baseConfig, changeInput);

  expect(result.changeOutput).toContain("interface FortyGigE0/0/0/46.400 l2transport");
  expect(result.changeOutput).toContain("description new,To:server1");
});
