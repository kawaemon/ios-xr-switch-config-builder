import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import init, { parse_config, analyze_config } from "../src/wasm/pkg/ncs_wasm";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const wasmPath = join(
  __dirname,
  "..",
  "src",
  "wasm",
  "pkg",
  "ncs_wasm_bg.wasm"
);
const wasmBuffer = readFileSync(wasmPath);

await init(wasmBuffer);

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
    const parsed = parse_config(iosxrConfig);
    const analyzed = analyze_config(iosxrConfig);

    expect({
      parsed: parsed.map((node) => nodeToJson(node)),
      analyzed: {
        domains: analyzed.domains.map((d) => ({
          vlanTag: d.vlanTag,
          interfaces: [...d.interfaces],
        })),
        lintOutput: analyzed.lintOutput,
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
      },
    });
  });
});
