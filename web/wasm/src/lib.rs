use wasm_bindgen::prelude::*;

mod change;
mod parse;
mod regex;
mod semantics;
mod simplified_config;

use crate::parse::Node as ParsedNode;

pub use change::generate_change;
pub use parse::tokenize;
pub use semantics::{analyze, Config};

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct NodeStmt {
    pub stmt: String,
}

#[wasm_bindgen]
impl NodeStmt {
    #[wasm_bindgen(constructor)]
    pub fn new(stmt: String) -> Self {
        Self { stmt }
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct NodeBlock {
    pub name: String,
    pub stmts: Vec<Node>,
}

#[wasm_bindgen]
impl NodeBlock {
    #[wasm_bindgen(constructor)]
    pub fn new(name: String, stmts: Vec<Node>) -> Self {
        Self { name, stmts }
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct Node {
    inner: NodeInner,
}

#[derive(Clone)]
enum NodeInner {
    Block(NodeBlock),
    Stmt(NodeStmt),
}

#[wasm_bindgen]
impl Node {
    #[wasm_bindgen(getter, js_name = type)]
    pub fn node_type(&self) -> String {
        match &self.inner {
            NodeInner::Block(_) => "block".to_string(),
            NodeInner::Stmt(_) => "stmt".to_string(),
        }
    }

    #[wasm_bindgen(getter, js_name = asBlock)]
    pub fn as_block(&self) -> Option<NodeBlock> {
        match &self.inner {
            NodeInner::Block(b) => Some(NodeBlock {
                name: b.name.clone(),
                stmts: b.stmts.clone(),
            }),
            _ => None,
        }
    }

    #[wasm_bindgen(getter, js_name = asStmt)]
    pub fn as_stmt(&self) -> Option<NodeStmt> {
        match &self.inner {
            NodeInner::Stmt(s) => Some(NodeStmt {
                stmt: s.stmt.clone(),
            }),
            _ => None,
        }
    }
}

fn convert_node_to_wasm(node: &ParsedNode) -> Node {
    match node {
        ParsedNode::Block(b) => Node {
            inner: NodeInner::Block(NodeBlock {
                name: b.name.clone(),
                stmts: b.stmts.iter().map(convert_node_to_wasm).collect(),
            }),
        },
        ParsedNode::Stmt(s) => Node {
            inner: NodeInner::Stmt(NodeStmt {
                stmt: s.stmt.clone(),
            }),
        },
    }
}

#[wasm_bindgen]
pub fn wasm_version() -> String {
    "ncs-wasm 0.1.0".to_string()
}

#[wasm_bindgen]
pub fn analyze_config(config_text: String) -> Result<Config, String> {
    let nodes = tokenize(&config_text);
    Ok(analyze(&nodes))
}

#[wasm_bindgen]
pub fn lint_config(config_text: String) -> Result<String, String> {
    let nodes = tokenize(&config_text);
    let config = analyze(&nodes);
    Ok(config.lint())
}

#[wasm_bindgen]
pub fn parse_config(config_text: String) -> Result<Vec<Node>, String> {
    let nodes = tokenize(&config_text);
    Ok(nodes.iter().map(convert_node_to_wasm).collect())
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct GeneratedChange {
    #[wasm_bindgen(js_name = changeOutput)]
    pub change_output: String,
}

#[wasm_bindgen]
pub fn generate_change_config(
    base_config: String,
    change_input: String,
) -> Result<GeneratedChange, String> {
    let change_output = generate_change(&base_config, &change_input)?;
    Ok(GeneratedChange { change_output })
}
