use crate::ast::span::Span;

/// A statement node with span information
#[derive(Debug, Clone)]
pub struct SpannedNodeStmt {
    pub stmt: String,
    pub span: Span,
}

/// A block node with span information
#[derive(Debug, Clone)]
pub struct SpannedNodeBlock {
    pub name: String,
    pub span: Span,
    pub stmts: Vec<SpannedNode>,
}

impl SpannedNodeBlock {
    pub fn stmts(&self) -> impl Iterator<Item = &SpannedNode> {
        self.stmts.iter()
    }
}

/// A node in the AST (either a block or a statement) with span information
#[derive(Debug, Clone)]
pub enum SpannedNode {
    Block(SpannedNodeBlock),
    Stmt(SpannedNodeStmt),
}

impl SpannedNode {
    pub fn as_stmt(&self) -> Option<&SpannedNodeStmt> {
        match self {
            SpannedNode::Stmt(s) => Some(s),
            _ => None,
        }
    }
}
