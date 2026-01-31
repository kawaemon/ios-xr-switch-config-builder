use crate::ast::{Span, SpannedNode, SpannedNodeBlock, SpannedNodeStmt};

fn get_indent(line: &str) -> usize {
    line.chars().take_while(|&c| c == ' ').count()
}

struct LinesWithLineNumbers<'a> {
    lines: &'a [&'a str],
    index: usize,
    line_number: u32, // 1-indexed line number
}

impl<'a> LinesWithLineNumbers<'a> {
    fn new(lines: &'a [&'a str]) -> Self {
        LinesWithLineNumbers {
            lines,
            index: 0,
            line_number: 1,
        }
    }

    fn peek(&self) -> Option<&'a str> {
        self.lines.get(self.index).copied()
    }
}

impl<'a> Iterator for LinesWithLineNumbers<'a> {
    type Item = (&'a str, u32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.lines.len() {
            let result = (self.lines[self.index], self.line_number);
            self.index += 1;
            self.line_number += 1;
            Some(result)
        } else {
            None
        }
    }
}

/// Tokenizes the input string into a tree of SpannedNodes with span information
pub fn tokenize_spanned(s: &str) -> Vec<SpannedNode> {
    let lines_vec: Vec<&str> = s.lines().collect();
    let mut lines = LinesWithLineNumbers::new(&lines_vec);
    let mut elements = Vec::new();
    tokenize_impl(&mut lines, &mut elements);
    elements
}

fn tokenize_impl(lines: &mut LinesWithLineNumbers, res: &mut Vec<SpannedNode>) {
    while let Some((l, line_no)) = lines.next() {
        let this_indent = get_indent(l);
        let line = l.trim();

        let peek = lines.peek().unwrap_or("");
        let next_indent = get_indent(peek);
        let peek_trimmed = peek.trim();

        // this is beginning of block
        if next_indent > this_indent {
            let mut buf = Vec::new();
            tokenize_impl(lines, &mut buf);
            res.push(SpannedNode::Block(SpannedNodeBlock {
                name: line.to_string(),
                span: Span::line_only(line_no),
                stmts: buf,
            }));
            continue;
        }

        if !line.starts_with('!') && !line.is_empty() {
            res.push(SpannedNode::Stmt(SpannedNodeStmt {
                stmt: line.to_string(),
                span: Span::line_only(line_no),
            }));
        }

        // next line is other block
        if next_indent < this_indent {
            // wtf
            if peek_trimmed == "end-set" || peek_trimmed == "end-policy" {
                if let Some((next_line, next_line_no)) = lines.next() {
                    res.push(SpannedNode::Stmt(SpannedNodeStmt {
                        stmt: next_line.trim().to_string(),
                        span: Span::line_only(next_line_no),
                    }));
                }
            }
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_spanned_basic() {
        let input = r#"interface Foo
  description test
  mtu 9000
"#;
        let nodes = tokenize_spanned(input);
        assert_eq!(nodes.len(), 1);

        if let Some(block) = nodes[0].as_block() {
            assert_eq!(block.name, "interface Foo");
            assert_eq!(block.span.line.get(), 1);
            assert_eq!(block.stmts.len(), 2);

            if let Some(stmt) = block.stmts[0].as_stmt() {
                assert_eq!(stmt.stmt, "description test");
                assert_eq!(stmt.span.line.get(), 2);
            }

            if let Some(stmt) = block.stmts[1].as_stmt() {
                assert_eq!(stmt.stmt, "mtu 9000");
                assert_eq!(stmt.span.line.get(), 3);
            }
        } else {
            panic!("Expected block");
        }
    }

    #[test]
    fn test_tokenize_spanned_multiple_blocks() {
        let input = r#"vlan database
  vlan 100 name test

interface Foo
  description bar
"#;
        let nodes = tokenize_spanned(input);
        assert_eq!(nodes.len(), 2);

        if let Some(block) = nodes[0].as_block() {
            assert_eq!(block.name, "vlan database");
            assert_eq!(block.span.line.get(), 1);
        }

        if let Some(block) = nodes[1].as_block() {
            assert_eq!(block.name, "interface Foo");
            assert_eq!(block.span.line.get(), 4);
        }
    }
}
