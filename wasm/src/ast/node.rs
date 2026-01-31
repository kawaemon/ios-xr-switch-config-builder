use crate::ast::span::Span;

/// A statement node with span information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpannedNodeStmt {
    /// Raw statement text.
    pub stmt: String,
    /// Source location for the statement.
    pub span: Span,
}

/// A block node with span information.
#[derive(Debug, Clone)]
pub struct SpannedNodeBlock {
    /// Block header text.
    pub name: String,
    /// Source span of the block header.
    pub span: Span,
    /// Child nodes contained in the block.
    pub stmts: Vec<SpannedNode>,
}

impl SpannedNodeBlock {
    /// Iterate over child nodes.
    pub fn stmts(&self) -> impl Iterator<Item = &SpannedNode> {
        self.stmts.iter()
    }
}

/// A node in the AST (either a block or a statement) with span information.
#[derive(Debug, Clone)]
pub enum SpannedNode {
    Block(SpannedNodeBlock),
    Stmt(SpannedNodeStmt),
}

impl SpannedNode {
    /// Treat the node as a block when applicable.
    pub fn as_block(&self) -> Option<&SpannedNodeBlock> {
        match self {
            SpannedNode::Block(b) => Some(b),
            _ => None,
        }
    }

    /// Treat the node as a statement when applicable.
    pub fn as_stmt(&self) -> Option<&SpannedNodeStmt> {
        match self {
            SpannedNode::Stmt(s) => Some(s),
            _ => None,
        }
    }
}
