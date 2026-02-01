import { it, expect } from "vitest";
import { wasm } from "./helpers";

it("supports VLAN range syntax for add (302-308)", () => {
  const baseConfig = `
interface FortyGigE0/0/0/46
  description To:server1
`.trim();

  const changeInput = `
vlan database
  vlan 302 name vlan302
  vlan 303 name vlan303
  vlan 304 name vlan304
  vlan 305 name vlan305
  vlan 306 name vlan306
  vlan 307 name vlan307
  vlan 308 name vlan308

interface FortyGigE0/0/0/46
  switchport trunk allowed vlan add 302-308
`.trim();

  const result = wasm.generate_change_config(baseConfig, changeInput);

  // すべてのVLANのサブインターフェースが作成されることを確認
  expect(result.changeOutput).toContain("interface FortyGigE0/0/0/46.302 l2transport");
  expect(result.changeOutput).toContain("interface FortyGigE0/0/0/46.303 l2transport");
  expect(result.changeOutput).toContain("interface FortyGigE0/0/0/46.304 l2transport");
  expect(result.changeOutput).toContain("interface FortyGigE0/0/0/46.305 l2transport");
  expect(result.changeOutput).toContain("interface FortyGigE0/0/0/46.306 l2transport");
  expect(result.changeOutput).toContain("interface FortyGigE0/0/0/46.307 l2transport");
  expect(result.changeOutput).toContain("interface FortyGigE0/0/0/46.308 l2transport");
});

it("supports VLAN range syntax for remove (302-308)", () => {
  const baseConfig = `
interface FortyGigE0/0/0/46
  description To:server1
interface FortyGigE0/0/0/46.302 l2transport
  description vlan302,To:server1
  encapsulation dot1q 302
  rewrite ingress tag pop 1 symmetric
interface FortyGigE0/0/0/46.303 l2transport
  description vlan303,To:server1
  encapsulation dot1q 303
  rewrite ingress tag pop 1 symmetric
interface FortyGigE0/0/0/46.304 l2transport
  description vlan304,To:server1
  encapsulation dot1q 304
  rewrite ingress tag pop 1 symmetric
interface FortyGigE0/0/0/46.305 l2transport
  description vlan305,To:server1
  encapsulation dot1q 305
  rewrite ingress tag pop 1 symmetric
interface FortyGigE0/0/0/46.306 l2transport
  description vlan306,To:server1
  encapsulation dot1q 306
  rewrite ingress tag pop 1 symmetric
interface FortyGigE0/0/0/46.307 l2transport
  description vlan307,To:server1
  encapsulation dot1q 307
  rewrite ingress tag pop 1 symmetric
interface FortyGigE0/0/0/46.308 l2transport
  description vlan308,To:server1
  encapsulation dot1q 308
  rewrite ingress tag pop 1 symmetric

l2vpn
  bridge group VLAN
    bridge-domain VLAN302
      description vlan302
      interface FortyGigE0/0/0/46.302
    bridge-domain VLAN303
      description vlan303
      interface FortyGigE0/0/0/46.303
    bridge-domain VLAN304
      description vlan304
      interface FortyGigE0/0/0/46.304
    bridge-domain VLAN305
      description vlan305
      interface FortyGigE0/0/0/46.305
    bridge-domain VLAN306
      description vlan306
      interface FortyGigE0/0/0/46.306
    bridge-domain VLAN307
      description vlan307
      interface FortyGigE0/0/0/46.307
    bridge-domain VLAN308
      description vlan308
      interface FortyGigE0/0/0/46.308
`.trim();

  const changeInput = `
interface FortyGigE0/0/0/46
  switchport trunk allowed vlan remove 302-308
`.trim();

  const result = wasm.generate_change_config(baseConfig, changeInput);

  // すべてのVLANのサブインターフェースが削除されることを確認
  expect(result.changeOutput).toContain("no interface FortyGigE0/0/0/46.302 l2transport");
  expect(result.changeOutput).toContain("no interface FortyGigE0/0/0/46.303 l2transport");
  expect(result.changeOutput).toContain("no interface FortyGigE0/0/0/46.304 l2transport");
  expect(result.changeOutput).toContain("no interface FortyGigE0/0/0/46.305 l2transport");
  expect(result.changeOutput).toContain("no interface FortyGigE0/0/0/46.306 l2transport");
  expect(result.changeOutput).toContain("no interface FortyGigE0/0/0/46.307 l2transport");
  expect(result.changeOutput).toContain("no interface FortyGigE0/0/0/46.308 l2transport");
});

it("supports mixed VLAN syntax (individual and range)", () => {
  const baseConfig = `
interface FortyGigE0/0/0/46
  description To:server1
`.trim();

  const changeInput = `
vlan database
  vlan 100 name vlan100
  vlan 302 name vlan302
  vlan 303 name vlan303
  vlan 304 name vlan304
  vlan 400 name vlan400

interface FortyGigE0/0/0/46
  switchport trunk allowed vlan add 100 302-304 400
`.trim();

  const result = wasm.generate_change_config(baseConfig, changeInput);

  // すべての指定されたVLANが作成されることを確認
  expect(result.changeOutput).toContain("interface FortyGigE0/0/0/46.100 l2transport");
  expect(result.changeOutput).toContain("interface FortyGigE0/0/0/46.302 l2transport");
  expect(result.changeOutput).toContain("interface FortyGigE0/0/0/46.303 l2transport");
  expect(result.changeOutput).toContain("interface FortyGigE0/0/0/46.304 l2transport");
  expect(result.changeOutput).toContain("interface FortyGigE0/0/0/46.400 l2transport");
});

it("rejects invalid range (start > end)", () => {
  const baseConfig = `
interface FortyGigE0/0/0/46
  description To:server1
`.trim();

  const changeInput = `
vlan database
  vlan 308 name vlan308
  vlan 302 name vlan302

interface FortyGigE0/0/0/46
  switchport trunk allowed vlan add 308-302
`.trim();

  expect(() => wasm.generate_change_config(baseConfig, changeInput)).toThrow(
    /無効なVLAN範囲.*308-302|invalid VLAN range.*308-302/i,
  );
});
