import { it, expect } from "vitest";
import { wasm } from "./helpers";

it("passes through all interface config transparently", () => {
  const config = `
interface HundredGigE0/0/0/10
  description uplink
  mtu 9000
  bundle id 100 mode active
  no shutdown
interface HundredGigE0/0/0/10.500 l2transport
  description mgmt
  encapsulation dot1q 500
  rewrite ingress tag pop 1 symmetric

l2vpn
  bridge group VLAN
    bridge-domain VLAN500
      description mgmt
      interface HundredGigE0/0/0/10.500
`.trim();

  const analyzed = wasm.analyze_config(config);

  expect(analyzed.simplifiedConfig).toEqual(
    [
      "interface HundredGigE0/0/0/10",
      "  description uplink",
      "  mtu 9000",
      "  bundle id 100 mode active",
      "  no shutdown",
      "  switchport mode trunk",
      "  switchport trunk allowed vlan add 500",
      "",
      "vlan database",
      "  vlan 500 name mgmt",
    ].join("\n"),
  );
});
