function getIndent(line: string): number {
  for (let i = 0; ; i++) {
    if (!line.startsWith(" ")) {
      return i;
    }
    line = line.substring(1);
  }
}
function last<T>(x: Array<T>): T | null {
  return x[x.length - 1] ?? null;
}
function assert(b: boolean, msg: unknown): asserts b is true {
  if (!b) {
    console.error(`assertion failed:`, msg);
    throw new Error();
  }
}

function splitSubinterfaceID(name: string): [string, number | null] {
  const regexp = /(?<baseif>[^.]+).(?<subifnum>\d+)?/gm;

  const mat = regexp.exec(name);
  if (mat == null) {
    throw new Error("invalid ifname");
  }

  const baseif = mat.groups!["baseif"];
  const subif = mat.groups!["subifnum"];

  return [baseif, subif == null ? null : parseInt(subif)];
}

/*

{
  type: "stmt",
  stmt: "logging trap warning"
}
{
  type: "block",
  name: "username kawak",
  stmts: [
    "address-family ipv4 "
  ]
}

*/

class NodeBlock {
  public readonly type: "block" = "block";
  constructor(
    public readonly name: string,
    public readonly stmts: ReadonlyArray<Node>
  ) {}
}
class NodeStmt {
  public readonly type: "stmt" = "stmt";
  constructor(public readonly stmt: string) {}
}

type Node = NodeBlock | NodeStmt;

class Lines {
  private i: number = 0;
  constructor(public readonly s: string[]) {}

  next(): string | null {
    return this.s[this.i++];
  }
  peek(): string | null {
    return this.s[this.i];
  }
}

function tokenize(lines: Lines, res: Array<Node>) {
  let l = null;
  while ((l = lines.next()) != null) {
    const thisIndent = getIndent(l);
    const line = l.trim();

    const p = lines.peek() ?? "";
    const nextIndent = getIndent(p);
    const peek = p.trim();

    // this is beginning of block
    if (nextIndent > thisIndent) {
      const buf: Array<Node> = [];
      tokenize(lines, buf);
      res.push(new NodeBlock(line, buf));
      continue;
    }

    if (!line.startsWith("!") && line.trim().length > 0) {
      res.push(new NodeStmt(line));
    }

    // next line is other block
    if (nextIndent < thisIndent) {
      // wtf
      if (["end-set", "end-policy"].includes(peek)) {
        res.push(new NodeStmt(lines.next()!));
      }
      return;
    }
  }
}

// interface Bundle-Ether100.300 l2transport
//  encapsulation dot1q 300
//  rewrite ingress tag pop 1 symmetric
class L2TransportConfig {
  private constructor(
    public readonly baseif: string,
    public readonly subIfNum: number,
    public readonly encap: number | null,
    public readonly hasRewrite: boolean
  ) {}

  public static tryNew(block: Node): L2TransportConfig | null {
    if (block.type !== "block") {
      return null;
    }

    const regexp = /interface (?<baseif>[^.]+)\.(?<subifnum>\d+) l2transport/gm;
    const mat = regexp.exec(block.name);
    if (mat == null) {
      return null;
    }

    const encap = block.stmts.flatMap((x) => {
      if (x.type !== "stmt") {
        return [];
      }
      const mat = /encapsulation dot1q (?<tag>\d+)/.exec(x.stmt);
      if (mat == null) {
        return [];
      }
      return parseInt(mat.groups!["tag"]!);
    });

    return new L2TransportConfig(
      mat.groups!["baseif"]!,
      parseInt(mat.groups!["subifnum"]!),
      encap[0],
      block.stmts.some(
        (x) =>
          x.type === "stmt" && x.stmt === "rewrite ingress tag pop 1 symmetric"
      )
    );
  }

  public lint(): Array<string> {
    const ret = [];
    if (this.subIfNum !== this.encap) {
      ret.push("sub-interface number と encapsulation tag が一致していません");
    }
    if (!this.hasRewrite) {
      ret.push("rewrite ingress tag pop 1 symmetric が存在しない");
    }
    return ret;
  }
}

// l2vpn
//  bridge group VLAN
//   bridge-domain VLAN300
//    interface HundredGigE0/0/0/0.300
//    interface HundredGigE0/0/0/1.300
class BridgeDomain {
  private constructor(
    public readonly vlanTag: number,
    public readonly interfaces: ReadonlyArray<string>
  ) {}

  public static tryNew(block: Node): BridgeDomain | null {
    if (block.type !== "block") {
      return null;
    }

    const mat = /bridge-domain VLAN(?<tag>\d+)/gm.exec(block.name);
    if (mat == null) {
      return null;
    }

    const interfaces = block.stmts.flatMap((x) => {
      if (x.type !== "stmt") {
        return [];
      }
      const mat = /interface (?<ifname>\S+)/.exec(x.stmt);
      if (mat == null) {
        return [];
      }
      return mat.groups!["ifname"];
    });

    return new BridgeDomain(parseInt(mat.groups!["tag"]), interfaces);
  }

  public lint(): Array<string> {
    const res = [];

    for (const int of this.interfaces) {
      const [_main, sub] = splitSubinterfaceID(int);
      if (this.vlanTag !== sub) {
        res.push(`sub-interface number がブリッジ名と異なる: ${int}`);
      }
    }

    return res;
  }
}

function getBridgeDomains(config: ReadonlyArray<Node>): Array<BridgeDomain> {
  const l2vpn = config.find(
    (x): x is NodeBlock => x.type === "block" && x.name === "l2vpn"
  );
  if (l2vpn == null) {
    return [];
  }

  const group = l2vpn.stmts.find(
    (x): x is NodeBlock => x.type === "block" && x.name === "bridge group VLAN"
  );
  if (group == null) {
    return [];
  }

  return group.stmts
    .map((x) => BridgeDomain.tryNew(x))
    .filter((x) => x != null);
}

function getL2Transports(
  config: ReadonlyArray<Node>
): Record<string, Array<L2TransportConfig>> {
  const interfaces = config
    .map((x) => L2TransportConfig.tryNew(x))
    .filter((x) => x != null);

  const grouped: Record<string, Array<L2TransportConfig>> = {};
  for (const k of interfaces) {
    if (!(k.baseif in grouped)) {
      grouped[k.baseif] = [];
    }
    grouped[k.baseif].push(k);
  }

  return grouped;
}

interface Config {
  l2transport: Record<string, Array<L2TransportConfig>>;
  domains: Array<BridgeDomain>;
}

function analyze(config: ReadonlyArray<Node>): Config {
  const l2transport = getL2Transports(config);
  const domains = getBridgeDomains(config);
  return { l2transport, domains };
}

export function main(current: string): Config {
  const elements: Array<Node> = [];
  tokenize(new Lines(current.split("\n")), elements);
  console.log(elements);

  const currentConfig = analyze(elements);
  console.log(currentConfig);

  return currentConfig;
}
