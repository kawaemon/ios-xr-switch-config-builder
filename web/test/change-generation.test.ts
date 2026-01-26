import { it, expect } from "vitest";
import { wasm } from "./helpers";

it("generates IOS XR change commands", () => {
  const baseConfig = `
interface FortyGigE0/0/0/46
  description To:server1
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
  switchport trunk allowed vlan remove 300
`.trim();

  const result = wasm.generate_change_config(baseConfig, changeInput);

  expect(result.changeOutput).toEqual(
    [
      "no interface FortyGigE0/0/0/46.300 l2transport",
      "",
      "interface FortyGigE0/0/0/46.400 l2transport",
      "  description new,To:server1",
      "  encapsulation dot1q 400",
      "  rewrite ingress tag pop 1 symmetric",
      "exit",
      "",
      "l2vpn",
      "  bridge group VLAN",
      "    bridge-domain VLAN300",
      "      description old",
      "      no interface FortyGigE0/0/0/46.300",
      "    exit",
      "    bridge-domain VLAN400",
      "      description new",
      "      interface FortyGigE0/0/0/46.400",
      "      exit",
      "    exit",
      "  exit",
      "exit",
    ].join("\n"),
  );
});
