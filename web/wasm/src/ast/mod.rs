//! AST types and utilities for representing parsed configuration with position information.

pub mod node;
pub mod span;

pub use node::{SpannedNode, SpannedNodeBlock, SpannedNodeStmt};
pub use span::Span;
