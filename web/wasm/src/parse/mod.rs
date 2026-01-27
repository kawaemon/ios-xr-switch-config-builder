//! Configuration parsing utilities for IOS XR syntax.

use serde::{Deserialize, Serialize};

pub mod parser;

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

fn convert_spanned(node: &crate::ast::SpannedNode) -> Node {
    match node {
        crate::ast::SpannedNode::Block(b) => Node::Block(NodeBlock {
            name: b.name.clone(),
            stmts: b.stmts.iter().map(convert_spanned).collect(),
        }),
        crate::ast::SpannedNode::Stmt(s) => Node::Stmt(NodeStmt {
            stmt: s.stmt.clone(),
        }),
    }
}

/// Tokenize input using the spanned parser and drop span information.
pub fn tokenize(input: &str) -> Vec<Node> {
    parser::tokenize_spanned(input)
        .iter()
        .map(convert_spanned)
        .collect()
}
