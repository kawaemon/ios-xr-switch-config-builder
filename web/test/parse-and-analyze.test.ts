import { it, expect } from "vitest";
import { wasm, nodeToJson } from "./helpers";

it("parses and analyzes IOS XR config", () => {
  const config = `
interface FortyGigE0/0/0/46
  description To:server1
  bundle id 1 mode active
interface FortyGigE0/0/0/46.300 l2transport
  description servers,To:server1
  encapsulation dot1q 300
  rewrite ingress tag pop 1 symmetric

l2vpn
  bridge group VLAN
    bridge-domain VLAN300
      description servers
      interface FortyGigE0/0/0/46.300
`.trim();

  const parsed = wasm.parse_config(config);
  const analyzed = wasm.analyze_config(config);

  expect(parsed.map((node) => nodeToJson(node))).toEqual([
    {
      type: "block",
      name: "interface FortyGigE0/0/0/46",
      stmts: [
        { type: "stmt", stmt: "description To:server1" },
        { type: "stmt", stmt: "bundle id 1 mode active" },
      ],
    },
    {
      type: "block",
      name: "interface FortyGigE0/0/0/46.300 l2transport",
      stmts: [
        { type: "stmt", stmt: "description servers,To:server1" },
        { type: "stmt", stmt: "encapsulation dot1q 300" },
        { type: "stmt", stmt: "rewrite ingress tag pop 1 symmetric" },
      ],
    },
    {
      type: "block",
      name: "l2vpn",
      stmts: [
        {
          type: "block",
          name: "bridge group VLAN",
          stmts: [
            {
              type: "block",
              name: "bridge-domain VLAN300",
              stmts: [
                { type: "stmt", stmt: "description servers" },
                { type: "stmt", stmt: "interface FortyGigE0/0/0/46.300" },
              ],
            },
          ],
        },
      ],
    },
  ]);

  expect(analyzed.simplifiedConfig).toEqual(
    [
      "interface FortyGigE0/0/0/46",
      "  description To:server1",
      "  bundle id 1 mode active",
      "  switchport mode trunk",
      "  switchport trunk allowed vlan add 300",
      "",
      "vlan database",
      "  vlan 300 name servers",
    ].join("\n"),
  );
});

it("includes bundle member interfaces in simplified config", () => {
  const config = `
interface FortyGigE0/0/0/46
  description member-a
  bundle id 100 mode active
interface FortyGigE0/0/0/47
  description member-b
  bundle id 100 mode active
interface Bundle-Ether100
  description uplink-bundle
interface Bundle-Ether100.300 l2transport
  encapsulation dot1q 300
  rewrite ingress tag pop 1 symmetric

l2vpn
  bridge group VLAN
    bridge-domain VLAN300
      description vlan-300
      interface Bundle-Ether100.300
`.trim();

  const simplified = wasm.analyze_config(config).simplifiedConfig;

  expect(simplified).toEqual(
    [
      "interface Bundle-Ether100",
      "  description uplink-bundle",
      "  switchport mode trunk",
      "  switchport trunk allowed vlan add 300",
      "",
      "interface FortyGigE0/0/0/46",
      "  description member-a",
      "  bundle id 100 mode active",
      "",
      "interface FortyGigE0/0/0/47",
      "  description member-b",
      "  bundle id 100 mode active",
      "",
      "vlan database",
      "  vlan 300 name vlan-300",
    ].join("\n"),
  );
});
