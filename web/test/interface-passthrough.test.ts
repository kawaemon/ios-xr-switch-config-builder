import { it, expect } from "vitest";
import { wasm } from "./helpers";

it("passes through non-switchport statements in interface block", () => {
  const baseConfig = `
interface HundredGigE0/0/0/1
  description old-desc
  mru 9000
`.trim();

  const changeInput = `
interface HundredGigE0/0/0/1
  description new-desc
  mru 9216
  shutdown
`.trim();

  const result = wasm.generate_change_config(baseConfig, changeInput);

  expect(result.changeOutput).toContain("interface HundredGigE0/0/0/1");
  expect(result.changeOutput).toContain("  description new-desc");
  expect(result.changeOutput).toContain("  mru 9216");
  expect(result.changeOutput).toContain("  shutdown");
  expect(result.changeOutput).toContain("exit");
});

it("outputs interface config even without vlan changes", () => {
  const baseConfig = `
interface HundredGigE0/0/0/1
  description To:server1
`.trim();

  const changeInput = `
interface HundredGigE0/0/0/1
  description To:server2
  mru 9216
`.trim();

  const result = wasm.generate_change_config(baseConfig, changeInput);

  expect(result.changeOutput).toEqual(
    [
      "interface HundredGigE0/0/0/1",
      "  description To:server2",
      "  mru 9216",
      "exit",
    ].join("\n"),
  );
});

it("combines interface config with vlan changes", () => {
  const baseConfig = `
interface FortyGigE0/0/0/1
  description To:server1
interface FortyGigE0/0/0/1.100 l2transport
  description test,To:server1
  encapsulation dot1q 100
  rewrite ingress tag pop 1 symmetric

l2vpn
  bridge group VLAN
    bridge-domain VLAN100
      description test
      interface FortyGigE0/0/0/1.100
`.trim();

  const changeInput = `
vlan database
  vlan 200 name new-vlan

interface FortyGigE0/0/0/1
  description To:server2
  mru 9216
  switchport trunk allowed vlan add 200
`.trim();

  const result = wasm.generate_change_config(baseConfig, changeInput);

  expect(result.changeOutput).toContain("interface FortyGigE0/0/0/1");
  expect(result.changeOutput).toContain("  description To:server2");
  expect(result.changeOutput).toContain("  mru 9216");
  expect(result.changeOutput).toContain("interface FortyGigE0/0/0/1.200 l2transport");
});
