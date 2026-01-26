import { it, expect } from "vitest";
import { wasm } from "./helpers";

it("allows interface with different speed type than base config", () => {
  const baseConfig = `
interface FortyGigE0/0/0/1
  description To:server1
  mru 9216
interface FortyGigE0/0/0/1.300 l2transport
  description test,To:server1
  encapsulation dot1q 300
  rewrite ingress tag pop 1 symmetric

l2vpn
  bridge group VLAN
    bridge-domain VLAN300
      description test
      interface FortyGigE0/0/0/1.300
`.trim();

  const changeInput = `
vlan database
  vlan 301 name test2

interface HundredGigE0/0/0/1
  description To:server1
  switchport mode trunk
  switchport trunk allowed vlan add 301
`.trim();

  // Should not throw - different speed types are now allowed
  expect(() => wasm.generate_change_config(baseConfig, changeInput)).not.toThrow();

  const result = wasm.generate_change_config(baseConfig, changeInput);
  expect(result.changeOutput).toContain("interface HundredGigE0/0/0/1.301 l2transport");
});

it("allows interface with same speed type as base config", () => {
  const baseConfig = `
interface FortyGigE0/0/0/1
  description To:server1
  mru 9216
interface FortyGigE0/0/0/1.300 l2transport
  description test,To:server1
  encapsulation dot1q 300
  rewrite ingress tag pop 1 symmetric

l2vpn
  bridge group VLAN
    bridge-domain VLAN300
      description test
      interface FortyGigE0/0/0/1.300
`.trim();

  const changeInput = `
vlan database
  vlan 301 name test2

interface FortyGigE0/0/0/1
  switchport mode trunk
  switchport trunk allowed vlan add 301
`.trim();

  // Should not throw - interface already exists with same speed
  expect(() => wasm.generate_change_config(baseConfig, changeInput)).not.toThrow();

  const result = wasm.generate_change_config(baseConfig, changeInput);
  expect(result.changeOutput).toContain("interface FortyGigE0/0/0/1.301 l2transport");
});

it("allows new interface not in base config", () => {
  const baseConfig = `
interface FortyGigE0/0/0/1
  description To:server1
  mru 9216
interface FortyGigE0/0/0/1.300 l2transport
  description test,To:server1
  encapsulation dot1q 300
  rewrite ingress tag pop 1 symmetric

l2vpn
  bridge group VLAN
    bridge-domain VLAN300
      description test
      interface FortyGigE0/0/0/1.300
`.trim();

  const changeInput = `
vlan database
  vlan 301 name test2

interface HundredGigE0/0/0/2
  description To:server2
  switchport mode trunk
  switchport trunk allowed vlan add 301
`.trim();

  const result = wasm.generate_change_config(baseConfig, changeInput);
  expect(result.changeOutput).toContain("interface HundredGigE0/0/0/2.301 l2transport");
});
