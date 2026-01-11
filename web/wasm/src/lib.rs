use wasm_bindgen::prelude::*;

mod parse;
mod regex;
mod semantics;

pub use parse::{tokenize, Node};
pub use semantics::{analyze, Config};

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct BridgeDomainJs {
    #[wasm_bindgen(js_name = vlanTag)]
    pub vlan_tag: u32,
    pub interfaces: Vec<String>,
}

#[wasm_bindgen]
impl BridgeDomainJs {
    #[wasm_bindgen(constructor)]
    pub fn new(vlan_tag: u32, interfaces: Vec<String>) -> Self {
        Self {
            vlan_tag,
            interfaces,
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct ConfigJs {
    pub domains: Vec<BridgeDomainJs>,
    #[wasm_bindgen(js_name = lintOutput)]
    pub lint_output: String,
}

#[wasm_bindgen]
impl ConfigJs {
    #[wasm_bindgen(constructor)]
    pub fn new(domains: Vec<BridgeDomainJs>, lint_output: String) -> Self {
        Self {
            domains,
            lint_output,
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct NodeStmtJs {
    pub stmt: String,
}

#[wasm_bindgen]
impl NodeStmtJs {
    #[wasm_bindgen(constructor)]
    pub fn new(stmt: String) -> Self {
        Self { stmt }
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct NodeBlockJs {
    pub name: String,
    pub stmts: Vec<NodeJs>,
}

#[wasm_bindgen]
impl NodeBlockJs {
    #[wasm_bindgen(constructor)]
    pub fn new(name: String, stmts: Vec<NodeJs>) -> Self {
        Self { name, stmts }
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct NodeJs {
    inner: NodeJsInner,
}

#[derive(Clone)]
enum NodeJsInner {
    Block(NodeBlockJs),
    Stmt(NodeStmtJs),
}

#[wasm_bindgen]
impl NodeJs {
    #[wasm_bindgen(getter, js_name = type)]
    pub fn node_type(&self) -> String {
        match &self.inner {
            NodeJsInner::Block(_) => "block".to_string(),
            NodeJsInner::Stmt(_) => "stmt".to_string(),
        }
    }

    #[wasm_bindgen(getter, js_name = asBlock)]
    pub fn as_block(&self) -> Option<NodeBlockJs> {
        match &self.inner {
            NodeJsInner::Block(b) => Some(NodeBlockJs {
                name: b.name.clone(),
                stmts: b.stmts.clone(),
            }),
            _ => None,
        }
    }

    #[wasm_bindgen(getter, js_name = asStmt)]
    pub fn as_stmt(&self) -> Option<NodeStmtJs> {
        match &self.inner {
            NodeJsInner::Stmt(s) => Some(NodeStmtJs {
                stmt: s.stmt.clone(),
            }),
            _ => None,
        }
    }
}

fn convert_node_to_js(node: &Node) -> NodeJs {
    match node {
        Node::Block(b) => NodeJs {
            inner: NodeJsInner::Block(NodeBlockJs {
                name: b.name.clone(),
                stmts: b.stmts.iter().map(convert_node_to_js).collect(),
            }),
        },
        Node::Stmt(s) => NodeJs {
            inner: NodeJsInner::Stmt(NodeStmtJs {
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
pub fn analyze_config(config_text: String) -> Result<ConfigJs, String> {
    let nodes = tokenize(&config_text);
    let config = analyze(&nodes);

    let domains_js = config
        .domains
        .iter()
        .map(|d| BridgeDomainJs {
            vlan_tag: d.vlan_tag,
            interfaces: d.interfaces.clone(),
        })
        .collect();

    Ok(ConfigJs {
        domains: domains_js,
        lint_output: config.lint(),
    })
}

#[wasm_bindgen]
pub fn lint_config(config_text: String) -> Result<String, String> {
    let nodes = tokenize(&config_text);
    let config = analyze(&nodes);
    Ok(config.lint())
}

#[wasm_bindgen]
pub fn parse_config(config_text: String) -> Result<Vec<NodeJs>, String> {
    let nodes = tokenize(&config_text);
    Ok(nodes.iter().map(convert_node_to_js).collect())
}
