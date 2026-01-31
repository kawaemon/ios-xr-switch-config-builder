import { expect, it } from "vitest";
import { wasm } from "./helpers";

it("clears all trunk VLANs when configured with none", () => {
  const baseConfig = `
interface FortyGigE0/0/0/46
  description To:server
interface FortyGigE0/0/0/46.300 l2transport
  description vlan300,To:server
  encapsulation dot1q 300
  rewrite ingress tag pop 1 symmetric
interface FortyGigE0/0/0/46.400 l2transport
  description vlan400,To:server
  encapsulation dot1q 400
  rewrite ingress tag pop 1 symmetric

l2vpn
  bridge group VLAN
    bridge-domain VLAN300
      interface FortyGigE0/0/0/46.300
    bridge-domain VLAN400
      interface FortyGigE0/0/0/46.400
`.trim();

  const changeInput = `
interface FortyGigE0/0/0/46
  switchport trunk allowed vlan none
`.trim();

  const result = wasm.generate_change_config(baseConfig, changeInput);

  expect(result.changeOutput).toEqual(
    [
      "no interface FortyGigE0/0/0/46.300 l2transport",
      "no interface FortyGigE0/0/0/46.400 l2transport",
      "",
      "l2vpn",
      "  bridge group VLAN",
      "    bridge-domain VLAN300",
      "      no interface FortyGigE0/0/0/46.300",
      "    exit",
      "    bridge-domain VLAN400",
      "      no interface FortyGigE0/0/0/46.400",
      "    exit",
      "  exit",
      "exit",
    ].join("\n"),
  );
});

it("rebuilds allowed VLANs after clearing list", () => {
  const baseConfig = `
interface FortyGigE0/0/0/46
  description To:server
interface FortyGigE0/0/0/46.300 l2transport
  description vlan300,To:server
  encapsulation dot1q 300
  rewrite ingress tag pop 1 symmetric
interface FortyGigE0/0/0/46.400 l2transport
  description vlan400,To:server
  encapsulation dot1q 400
  rewrite ingress tag pop 1 symmetric

l2vpn
  bridge group VLAN
    bridge-domain VLAN300
      interface FortyGigE0/0/0/46.300
    bridge-domain VLAN400
      interface FortyGigE0/0/0/46.400
`.trim();

  const changeInput = `
vlan database
  vlan 500 name new

interface FortyGigE0/0/0/46
  switchport trunk allowed vlan none
  switchport trunk allowed vlan add 500
`.trim();

  const result = wasm.generate_change_config(baseConfig, changeInput);

  expect(result.changeOutput).toEqual(
    [
      "no interface FortyGigE0/0/0/46.300 l2transport",
      "no interface FortyGigE0/0/0/46.400 l2transport",
      "",
      "interface FortyGigE0/0/0/46.500 l2transport",
      "  description new,To:server",
      "  encapsulation dot1q 500",
      "  rewrite ingress tag pop 1 symmetric",
      "exit",
      "",
      "l2vpn",
      "  bridge group VLAN",
      "    bridge-domain VLAN300",
      "      no interface FortyGigE0/0/0/46.300",
      "    exit",
      "    bridge-domain VLAN400",
      "      no interface FortyGigE0/0/0/46.400",
      "    exit",
      "    bridge-domain VLAN500",
      "      description new",
      "      interface FortyGigE0/0/0/46.500",
      "      exit",
      "    exit",
      "  exit",
      "exit",
    ].join("\n"),
  );
});

it("rejects clearing VLANs on bundled interfaces", () => {
  const baseConfig = `
interface HundredGigE0/0/0/10
  description uplink
  bundle id 100 mode active
`.trim();

  const changeInput = `
interface HundredGigE0/0/0/10
  switchport trunk allowed vlan none
`.trim();

  expect(() => wasm.generate_change_config(baseConfig, changeInput)).toThrow(
    /bundle 100/,
  );
});

it("clear overrides earlier add on the same interface", () => {
  const baseConfig = `
interface FortyGigE0/0/0/46
  description To:server
interface FortyGigE0/0/0/46.300 l2transport
  description vlan300,To:server
  encapsulation dot1q 300
  rewrite ingress tag pop 1 symmetric

l2vpn
  bridge group VLAN
    bridge-domain VLAN300
      interface FortyGigE0/0/0/46.300
`.trim();

  const changeInput = `
interface FortyGigE0/0/0/46
  switchport trunk allowed vlan add 300
  switchport trunk allowed vlan none
`.trim();

  const result = wasm.generate_change_config(baseConfig, changeInput);

  expect(result.changeOutput).toContain("no interface FortyGigE0/0/0/46.300 l2transport");
  expect(result.changeOutput).not.toContain("interface FortyGigE0/0/0/46.300 l2transport\n  description");
});
