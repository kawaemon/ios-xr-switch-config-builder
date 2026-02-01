import { it, expect } from "vitest";
import { wasm } from "./helpers";

it("generates commands in the correct order: base interface -> removals -> additions", () => {
  const baseConfig = `
interface FortyGigE0/0/0/10
  description old-desc
interface FortyGigE0/0/0/10.100 l2transport
  description vlan100
  encapsulation dot1q 100
  rewrite ingress tag pop 1 symmetric

l2vpn
  bridge group VLAN
    bridge-domain VLAN100
      interface FortyGigE0/0/0/10.100
`.trim();

  const changeInput = `
vlan database
  vlan 200 name vlan200

interface FortyGigE0/0/0/10
  description new-desc
  switchport trunk allowed vlan remove 100
  switchport trunk allowed vlan add 200

interface FortyGigE0/0/0/20
  description new-desc
  switchport trunk allowed vlan add 200
`.trim();

  const result = wasm.generate_change_config(baseConfig, changeInput);
  const output = result.changeOutput;

  expect(output).toBe(`interface FortyGigE0/0/0/10
  description new-desc
exit

no interface FortyGigE0/0/0/10.100 l2transport

interface FortyGigE0/0/0/10.200 l2transport
  description vlan200,new-desc
  encapsulation dot1q 200
  rewrite ingress tag pop 1 symmetric
exit

interface FortyGigE0/0/0/20
  description new-desc
exit

interface FortyGigE0/0/0/20.200 l2transport
  description vlan200,new-desc
  encapsulation dot1q 200
  rewrite ingress tag pop 1 symmetric
exit

l2vpn
  bridge group VLAN
    bridge-domain VLAN100
      no interface FortyGigE0/0/0/10.100
    exit
    bridge-domain VLAN200
      description vlan200
      interface FortyGigE0/0/0/10.200
      exit
      interface FortyGigE0/0/0/20.200
      exit
    exit
  exit
exit

`);
});
