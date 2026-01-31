import { it, expect } from "vitest";
import { wasm } from "./helpers";

it("passes through BVI interface configuration", () => {
  const baseConfig = `
interface BVI100
  description old-description

l2vpn
  bridge group VLAN
    bridge-domain VLAN100
      description test
      routed interface BVI100
`.trim();

  const changeInput = `
interface BVI100
  description hoge
  ipv4 address 192.168.1.1 255.255.255.0
`.trim();

  const result = wasm.generate_change_config(baseConfig, changeInput);

  expect(result.changeOutput).toContain("interface BVI100");
  expect(result.changeOutput).toContain("  description hoge");
  expect(result.changeOutput).toContain("  ipv4 address 192.168.1.1 255.255.255.0");
  expect(result.changeOutput).toContain("exit");
});

it("creates new BVI interface with passthrough config", () => {
  const baseConfig = `
l2vpn
  bridge group VLAN
    bridge-domain VLAN200
      description test
`.trim();

  const changeInput = `
interface BVI200
  description new-bvi
  no shutdown
`.trim();

  const result = wasm.generate_change_config(baseConfig, changeInput);

  // BVI interface config should be output
  expect(result.changeOutput).toContain("interface BVI200");
  expect(result.changeOutput).toContain("  description new-bvi");
  expect(result.changeOutput).toContain("  no shutdown");
  expect(result.changeOutput).toContain("exit");

  // BVI should also be added to bridge-domain
  expect(result.changeOutput).toContain("routed interface BVI200");
});

it("handles BVI interface with only description", () => {
  const baseConfig = `
l2vpn
  bridge group VLAN
    bridge-domain VLAN300
      description servers
      routed interface BVI300
`.trim();

  const changeInput = `
interface BVI300
  description updated-servers
`.trim();

  const result = wasm.generate_change_config(baseConfig, changeInput);

  expect(result.changeOutput).toContain("interface BVI300");
  expect(result.changeOutput).toContain("  description updated-servers");
  expect(result.changeOutput).toContain("exit");
});
