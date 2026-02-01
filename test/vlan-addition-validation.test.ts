import { it, expect } from "vitest";
import { wasm } from "./helpers";

it("rejects addition of VLAN not defined in vlan database", () => {
  const baseConfig = `
interface FortyGigE0/0/0/46
  description To:server1
`.trim();

  const changeInput = `
vlan database
  vlan 300 name test

interface FortyGigE0/0/0/46
  switchport trunk allowed vlan add 300 400
`.trim();

  expect(() => wasm.generate_change_config(baseConfig, changeInput)).toThrow(
    /VLAN 400.*vlan database.*定義されていません|VLAN 400.*not defined in vlan database/i,
  );
});

it("allows addition of VLAN defined in vlan database", () => {
  const baseConfig = `
interface FortyGigE0/0/0/46
  description To:server1
`.trim();

  const changeInput = `
vlan database
  vlan 300 name test
  vlan 400 name prod

interface FortyGigE0/0/0/46
  switchport trunk allowed vlan add 300 400
`.trim();

  const result = wasm.generate_change_config(baseConfig, changeInput);
  expect(result.changeOutput).toContain("interface FortyGigE0/0/0/46.300 l2transport");
  expect(result.changeOutput).toContain("interface FortyGigE0/0/0/46.400 l2transport");
});

it("allows addition of VLAN already in base config without vlan database entry", () => {
  const baseConfig = `
interface FortyGigE0/0/0/46
  description To:server1
interface FortyGigE0/0/0/46.100 l2transport
  description existing,To:server1
  encapsulation dot1q 100
  rewrite ingress tag pop 1 symmetric

l2vpn
  bridge group VLAN
    bridge-domain VLAN100
      description existing
      interface FortyGigE0/0/0/46.100
`.trim();

  const changeInput = `
interface FortyGigE0/0/0/47
  description To:server2
  switchport trunk allowed vlan add 100
`.trim();

  const result = wasm.generate_change_config(baseConfig, changeInput);
  expect(result.changeOutput).toContain("interface FortyGigE0/0/0/47.100 l2transport");
});

it("rejects addition when VLAN exists in base but not in vlan database for new interface", () => {
  const baseConfig = `
interface FortyGigE0/0/0/46
  description To:server1
interface FortyGigE0/0/0/46.100 l2transport
  description existing,To:server1
  encapsulation dot1q 100
  rewrite ingress tag pop 1 symmetric

l2vpn
  bridge group VLAN
    bridge-domain VLAN100
      description existing
      interface FortyGigE0/0/0/46.100
`.trim();

  const changeInput = `
vlan database
  vlan 200 name new-vlan

interface FortyGigE0/0/0/47
  description To:server2
  switchport trunk allowed vlan add 100 500
`.trim();

  expect(() => wasm.generate_change_config(baseConfig, changeInput)).toThrow(
    /VLAN 500.*vlan database.*定義されていません|VLAN 500.*not defined in vlan database/i,
  );
});
