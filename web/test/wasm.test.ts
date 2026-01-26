import { describe, it, expect } from "vitest";
import * as wasmModule from "../src/wasm/pkg/ncs_wasm";

type WasmModule = typeof wasmModule & {
  generate_change_config: (
    baseConfig: string,
    changeInput: string,
  ) => { changeOutput: string };
};

const wasm = wasmModule as WasmModule;

const iosxrConfig = `
interface FortyGigE0/0/0/46
  description To:eth1.server1
  mru 9216
interface FortyGigE0/0/0/46.300 l2transport
  description servers,To:eth1.server1
  encapsulation dot1q 300
  rewrite ingress tag pop 1 symmetric

interface BVI300
  description servers
  ipv4 address 192.168.1.1 255.255.255.0

l2vpn
  bridge group VLAN
    bridge-domain VLAN300
      description servers
      interface FortyGigE0/0/0/46.300
      routed interface BVI300`.trim();

function nodeToJson(node: unknown): unknown {
  const n = node as { type: string; asBlock: unknown; asStmt: unknown };
  if (n.type === "block") {
    const block = n.asBlock as
      | { name: string; stmts: unknown[] }
      | null
      | undefined;
    if (block) {
      return {
        type: "block",
        name: block.name,
        stmts: block.stmts.map((s) => nodeToJson(s)),
      };
    }
  } else if (n.type === "stmt") {
    const stmt = n.asStmt as { stmt: string } | null | undefined;
    if (stmt) {
      return {
        type: "stmt",
        stmt: stmt.stmt,
      };
    }
  }
  return n;
}

describe("WASM", () => {
  it("should parse and analyze IOS XR config", () => {
    const parsed = wasm.parse_config(iosxrConfig);
    const analyzed = wasm.analyze_config(iosxrConfig);

    expect({
      parsed: parsed.map((node) => nodeToJson(node)),
      analyzed: {
        domains: analyzed.domains.map((d) => ({
          vlanTag: d.vlanTag,
          interfaces: [...d.interfaces],
        })),
        lintOutput: analyzed.lintOutput,
        simplifiedConfig: analyzed.simplifiedConfig,
      },
    }).toEqual({
      parsed: [
        {
          type: "block",
          name: "interface FortyGigE0/0/0/46",
          stmts: [
            { type: "stmt", stmt: "description To:eth1.server1" },
            { type: "stmt", stmt: "mru 9216" },
          ],
        },
        {
          type: "block",
          name: "interface FortyGigE0/0/0/46.300 l2transport",
          stmts: [
            { type: "stmt", stmt: "description servers,To:eth1.server1" },
            { type: "stmt", stmt: "encapsulation dot1q 300" },
            { type: "stmt", stmt: "rewrite ingress tag pop 1 symmetric" },
          ],
        },
        {
          type: "block",
          name: "interface BVI300",
          stmts: [
            { type: "stmt", stmt: "description servers" },
            { type: "stmt", stmt: "ipv4 address 192.168.1.1 255.255.255.0" },
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
                    { type: "stmt", stmt: "routed interface BVI300" },
                  ],
                },
              ],
            },
          ],
        },
      ],
      analyzed: {
        domains: [
          {
            vlanTag: 300,
            interfaces: ["FortyGigE0/0/0/46.300"],
          },
        ],
        lintOutput: "",
        simplifiedConfig: [
          "interface FortyGigE0/0/0/46",
          "  description To:eth1.server1",
          "  mru 9216",
          "  switchport mode trunk",
          "  switchport trunk allowed vlan add 300",
          "",
          "interface BVI300",
          "  description servers",
          "  ! -- L3 config reduced --",
          "",
          "vlan database",
          "  vlan 300 name servers",
        ].join("\n"),
      },
    });
  });

  it("generates IOS XR change config from simplified diff", () => {
    const baseConfig = `
interface FortyGigE0/0/0/46
  description To:eth1.server1
  mru 9216
interface FortyGigE0/0/0/46.300 l2transport
  description servers,To:eth1.server1
  encapsulation dot1q 300
  rewrite ingress tag pop 1 symmetric

interface BVI300
  description servers

l2vpn
  bridge group VLAN
    bridge-domain VLAN300
      description servers
      interface FortyGigE0/0/0/46.300
      routed interface BVI300
`.trim();

    const changeInput = `
vlan database
  vlan 301 name servers-2
  vlan 500 name mgmt

interface FortyGigE0/0/0/46
  switchport trunk allowed vlan add 301 500
  switchport trunk allowed vlan remove 300
interface FortyGigE0/0/0/47
  description To:eth1.server2
  switchport mode trunk
  switchport trunk allowed vlan add 300 500
interface FortyGigE0/0/0/48
  description To:ge1/1.mgmt-sw
  switchport mode trunk
  switchport trunk allowed vlan add 500

interface BVI500
`.trim();

    const generated = wasm.generate_change_config(baseConfig, changeInput);

    expect(generated.changeOutput).toEqual(
      [
        "no interface FortyGigE0/0/0/46.300 l2transport",
        "",
        "interface FortyGigE0/0/0/46.301 l2transport",
        "  description servers-2,To:eth1.server1",
        "  encapsulation dot1q 301",
        "  rewrite ingress tag pop 1 symmetric",
        "exit",
        "",
        "interface FortyGigE0/0/0/46.500 l2transport",
        "  description mgmt,To:eth1.server1",
        "  encapsulation dot1q 500",
        "  rewrite ingress tag pop 1 symmetric",
        "exit",
        "",
        "interface FortyGigE0/0/0/47.300 l2transport",
        "  description servers,To:eth1.server2",
        "  encapsulation dot1q 300",
        "  rewrite ingress tag pop 1 symmetric",
        "exit",
        "",
        "interface FortyGigE0/0/0/47.500 l2transport",
        "  description mgmt,To:eth1.server2",
        "  encapsulation dot1q 500",
        "  rewrite ingress tag pop 1 symmetric",
        "exit",
        "",
        "interface FortyGigE0/0/0/48.500 l2transport",
        "  description mgmt,To:ge1/1.mgmt-sw",
        "  encapsulation dot1q 500",
        "  rewrite ingress tag pop 1 symmetric",
        "exit",
        "",
        "l2vpn",
        "  bridge group VLAN",
        "    bridge-domain VLAN300",
        "      description servers",
        "      no interface FortyGigE0/0/0/46.300",
        "      interface FortyGigE0/0/0/47.300",
        "      exit",
        "    exit",
        "    bridge-domain VLAN301",
        "      description servers-2",
        "      interface FortyGigE0/0/0/46.301",
        "      exit",
        "    exit",
        "    bridge-domain VLAN500",
        "      description mgmt",
        "      interface FortyGigE0/0/0/46.500",
        "      exit",
        "      interface FortyGigE0/0/0/47.500",
        "      exit",
        "      interface FortyGigE0/0/0/48.500",
        "      exit",
        "      routed interface BVI500",
        "      exit",
        "    exit",
        "  exit",
        "exit",
      ].join("\n"),
    );
  });

  it("rejects change input missing vlan name", () => {
    const baseConfig = iosxrConfig;
    const changeInput = `
    vlan database
      vlan 300

    interface FortyGigE0/0/0/46
      switchport trunk allowed vlan add 300
    `;

    expect(() => wasm.generate_change_config(baseConfig, changeInput)).toThrow(
      "vlan name is required",
    );
  });

  it("rejects interface block without supported statements", () => {
    const baseConfig = iosxrConfig;
    const changeInput = `
    vlan database
      vlan 300 name demo

    interface FortyGigE0/0/0/46
    `;

    expect(() => wasm.generate_change_config(baseConfig, changeInput)).toThrow(
      /interface block must contain supported statements/,
    );
  });

  it("rejects access switchport mode", () => {
    const baseConfig = iosxrConfig;
    const changeInput = `
    vlan database
      vlan 300 name demo

    interface FortyGigE0/0/0/46
      description To:eth1.server1
      switchport mode access
      switchport access vlan 300
    `;

    expect(() => wasm.generate_change_config(baseConfig, changeInput)).toThrow(
      /switchport mode access is not supported/,
    );
  });

  it("requires description for non-BVI interfaces", () => {
    const baseConfig = iosxrConfig;
    const changeInput = `
    vlan database
      vlan 300 name demo

    interface HundredGigE0/0/0/10
      switchport mode trunk
      switchport trunk allowed vlan add 300
    `;

    expect(() => wasm.generate_change_config(baseConfig, changeInput)).toThrow(
      /interface requires description/,
    );
  });

  it("rejects missing description even if interface already exists", () => {
    const baseConfig = `
interface HundredGigE0/0/0/11
  mru 9000
interface HundredGigE0/0/0/11.300 l2transport
  description servers
  encapsulation dot1q 300
  rewrite ingress tag pop 1 symmetric

l2vpn
  bridge group VLAN
    bridge-domain VLAN300
      description servers
      interface HundredGigE0/0/0/11.300
      exit
    exit
  exit
exit
`.trim();

    const changeInput = `
vlan database
  vlan 400 name new

interface HundredGigE0/0/0/11
  switchport trunk allowed vlan add 400
`.trim();

    expect(() => wasm.generate_change_config(baseConfig, changeInput)).toThrow(
      /interface requires description: HundredGigE0\/0\/0\/11 \(line 4\)/,
    );
  });

  it("rejects removal of VLANs that don't exist in base config", () => {
    const baseConfig = `
interface FortyGigE0/0/0/46
  description To:eth1.server1
  mru 9216
interface FortyGigE0/0/0/46.300 l2transport
  description servers,To:eth1.server1
  encapsulation dot1q 300
  rewrite ingress tag pop 1 symmetric

interface BVI300
  description servers

l2vpn
  bridge group VLAN
    bridge-domain VLAN300
      description servers
      interface FortyGigE0/0/0/46.300
      routed interface BVI300
`.trim();

    const changeInput = `
interface FortyGigE0/0/0/46
  switchport trunk allowed vlan remove 100
`.trim();

    // VLAN100 doesn't exist in base config, so it should throw an error with correct line number
    expect(() => wasm.generate_change_config(baseConfig, changeInput)).toThrow(
      /cannot remove VLAN 100 from interface FortyGigE0\/0\/0\/46: VLAN not present in base config \(line 2\)/,
    );
  });

  it("shows correct line number for VLAN removal error in multi-line config", () => {
    const baseConfig = `
interface FortyGigE0/0/0/46
  description To:eth1.server1
  mru 9216
interface FortyGigE0/0/0/46.300 l2transport
  description servers,To:eth1.server1
  encapsulation dot1q 300
  rewrite ingress tag pop 1 symmetric

l2vpn
  bridge group VLAN
    bridge-domain VLAN300
      description servers
      interface FortyGigE0/0/0/46.300
`.trim();

    const changeInput = `
vlan database
  vlan 400 name test

interface FortyGigE0/0/0/46
  switchport trunk allowed vlan add 400
  switchport trunk allowed vlan remove 100
  switchport trunk allowed vlan remove 200
`.trim();

    // Line 6 has the remove 100 statement
    expect(() => wasm.generate_change_config(baseConfig, changeInput)).toThrow(
      /cannot remove VLAN 100 from interface FortyGigE0\/0\/0\/46: VLAN not present in base config \(line 6\)/,
    );
  });
});
