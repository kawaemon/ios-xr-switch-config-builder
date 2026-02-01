import { it, expect } from "vitest";
import { wasm } from "./helpers";

it("rejects VLAN configuration on bundled interfaces", () => {
  const baseConfig = `
interface HundredGigE0/0/0/10
  description uplink
  bundle id 100 mode active
`.trim();

  const changeInput = `
vlan database
  vlan 300 name test

interface HundredGigE0/0/0/10
  switchport trunk allowed vlan add 300
`.trim();

  try {
    wasm.generate_change_config(baseConfig, changeInput);
    expect.fail("Expected an error to be thrown");
  } catch (e: any) {
    const errorStr = String(e);
    expect(errorStr).toContain("HundredGigE0/0/0/10");
    expect(errorStr).toContain("Bundle 100");
    expect(errorStr).toContain("Bundle-Ether100");
  }
});

it("allows VLAN configuration on non-bundled interfaces", () => {
  const baseConfig = `
interface HundredGigE0/0/0/10
  description uplink
  mtu 9000
`.trim();

  const changeInput = `
vlan database
  vlan 300 name test

interface HundredGigE0/0/0/10
  switchport trunk allowed vlan add 300
`.trim();

  const result = wasm.generate_change_config(baseConfig, changeInput);
  expect(result.changeOutput).toContain("interface HundredGigE0/0/0/10.300 l2transport");
});

it("allows passthrough config on bundled interfaces without VLAN changes", () => {
  const baseConfig = `
interface HundredGigE0/0/0/10
  description old-uplink
  bundle id 100 mode active
`.trim();

  const changeInput = `
interface HundredGigE0/0/0/10
  description new-uplink
  no shutdown
`.trim();

  const result = wasm.generate_change_config(baseConfig, changeInput);
  expect(result.changeOutput).toContain("interface HundredGigE0/0/0/10");
  expect(result.changeOutput).toContain("  description new-uplink");
  expect(result.changeOutput).toContain("  no shutdown");
});

it("shows correct bundle ID in error message for different bundle numbers", () => {
  const baseConfig = `
interface FortyGigE0/0/0/1
  description member-of-bundle-5
  bundle id 5 mode active

interface FortyGigE0/0/0/2
  description member-of-bundle-200
  bundle id 200 mode active
`.trim();

  const changeInput1 = `
vlan database
  vlan 100 name test

interface FortyGigE0/0/0/1
  switchport trunk allowed vlan add 100
`.trim();

  const changeInput2 = `
vlan database
  vlan 100 name test

interface FortyGigE0/0/0/2
  switchport trunk allowed vlan add 100
`.trim();

  // Test first interface (bundle 5)
  try {
    wasm.generate_change_config(baseConfig, changeInput1);
    expect.fail("Expected an error for bundle 5");
  } catch (e: any) {
    const errorStr = String(e);
    expect(errorStr).toContain("FortyGigE0/0/0/1");
    expect(errorStr).toContain("Bundle 5");
    expect(errorStr).toContain("Bundle-Ether5");
  }

  // Test second interface (bundle 200)
  try {
    wasm.generate_change_config(baseConfig, changeInput2);
    expect.fail("Expected an error for bundle 200");
  } catch (e: any) {
    const errorStr = String(e);
    expect(errorStr).toContain("FortyGigE0/0/0/2");
    expect(errorStr).toContain("Bundle 200");
    expect(errorStr).toContain("Bundle-Ether200");
  }
});
