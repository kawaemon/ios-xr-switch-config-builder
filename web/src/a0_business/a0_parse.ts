function getIndent(line: string): number {
  for (let i = 0; ; i++) {
    if (!line.startsWith(" ")) {
      return i;
    }
    line = line.substring(1);
  }
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
    "group root-lr",
    "group cisco-support"
  ]
}

*/

export class NodeBlock {
  public readonly type = "block" as const;
  constructor(
    public readonly name: string,
    public readonly stmts: ReadonlyArray<Node>
  ) {}
}
export class NodeStmt {
  public readonly type = "stmt" as const;
  constructor(public readonly stmt: string) {}
}

export type Node = NodeBlock | NodeStmt;

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

export function tokenize(s: string): Array<Node> {
  const lines = new Lines(s.split("\n"));
  const elements: Array<Node> = [];
  _tokenize(lines, elements);
  return elements;
}

function _tokenize(lines: Lines, res: Array<Node>) {
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
      _tokenize(lines, buf);
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
