use serde::{Deserialize, Serialize};

fn get_indent(line: &str) -> usize {
    line.chars().take_while(|&c| c == ' ').count()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeBlock {
    pub name: String,
    pub stmts: Vec<Node>,
}
impl NodeBlock {
    pub fn stmts(&self) -> impl Iterator<Item = &Node> {
        self.stmts.iter()
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStmt {
    pub stmt: String,
}

impl NodeStmt {
    pub fn stmt(&self) -> &str {
        &self.stmt
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Node {
    Block(NodeBlock),
    Stmt(NodeStmt),
}

impl Node {
    pub fn as_block(&self) -> Option<&NodeBlock> {
        match self {
            Node::Block(b) => Some(b),
            _ => None,
        }
    }

    pub fn as_stmt(&self) -> Option<&NodeStmt> {
        match self {
            Node::Stmt(s) => Some(s),
            _ => None,
        }
    }
}

struct Lines<'a> {
    lines: &'a [&'a str],
    index: usize,
}

impl<'a> Lines<'a> {
    fn peek(&self) -> Option<&'a str> {
        self.lines.get(self.index).copied()
    }
}

impl<'a> Iterator for Lines<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.lines.len() {
            let result = self.lines[self.index];
            self.index += 1;
            Some(result)
        } else {
            None
        }
    }
}

pub fn tokenize(s: &str) -> Vec<Node> {
    let lines_vec: Vec<&str> = s.lines().collect();
    let mut lines = Lines {
        lines: &lines_vec,
        index: 0,
    };
    let mut elements = Vec::new();
    tokenize_impl(&mut lines, &mut elements);
    elements
}

fn tokenize_impl(lines: &mut Lines, res: &mut Vec<Node>) {
    while let Some(l) = lines.next() {
        let this_indent = get_indent(l);
        let line = l.trim();

        let peek = lines.peek().unwrap_or("");
        let next_indent = get_indent(peek);
        let peek_trimmed = peek.trim();

        // this is beginning of block
        if next_indent > this_indent {
            let mut buf = Vec::new();
            tokenize_impl(lines, &mut buf);
            res.push(Node::Block(NodeBlock {
                name: line.to_string(),
                stmts: buf,
            }));
            continue;
        }

        if !line.starts_with('!') && !line.is_empty() {
            res.push(Node::Stmt(NodeStmt {
                stmt: line.to_string(),
            }));
        }

        // next line is other block
        if next_indent < this_indent {
            // wtf
            if peek_trimmed == "end-set" || peek_trimmed == "end-policy" {
                if let Some(next_line) = lines.next() {
                    res.push(Node::Stmt(NodeStmt {
                        stmt: next_line.trim().to_string(),
                    }));
                }
            }
            return;
        }
    }
}
