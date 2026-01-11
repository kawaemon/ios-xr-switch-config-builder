use crate::parse::{Node, NodeBlock, NodeStmt};
use crate::regex;
use crate::simplified_config::{build_simplified_config, SimplifiedConfigData};
use std::collections::{BTreeMap, HashMap};
use wasm_bindgen::prelude::*;

pub(crate) fn split_subinterface_id(name: &str) -> Result<(String, Option<u32>), String> {
    let caps = regex!(r"^([^.]+)\.(\d+)?$")
        .captures(name)
        .ok_or_else(|| "invalid ifname".to_string())?;

    let baseif = caps.get(1).unwrap().as_str().to_string();
    let subif = caps.get(2).and_then(|m| m.as_str().parse::<u32>().ok());

    Ok((baseif, subif))
}

#[derive(Debug, Clone)]
pub struct L2TransportConfig {
    pub baseif: String,
    pub sub_if_num: u32,
    pub encap: Option<u32>,
    pub has_rewrite: bool,
}

impl L2TransportConfig {
    pub fn try_new(node: &Node) -> Option<Self> {
        let node_block = node.as_block()?;

        let caps = regex!(r"^interface ([^.]+)\.(\d+) l2transport$").captures(&node_block.name)?;

        let baseif = caps.get(1)?.as_str().to_string();
        let sub_if_num = caps.get(2)?.as_str().parse::<u32>().ok()?;

        let has_rewrite = node_block
            .stmts()
            .filter_map(|x| x.as_stmt())
            .any(|stmt: &NodeStmt| stmt.stmt() == "rewrite ingress tag pop 1 symmetric");

        Some(L2TransportConfig {
            baseif,
            sub_if_num,
            encap: Self::find_encap(node_block),
            has_rewrite,
        })
    }

    fn find_encap(n: &NodeBlock) -> Option<u32> {
        n.stmts().filter_map(|x| x.as_stmt()).find_map(|stmt| {
            let caps = regex!(r"^encapsulation dot1q (\d+)$").captures(stmt.stmt())?;
            let m = caps.get(1)?;
            m.as_str().parse::<u32>().ok()
        })
    }

    pub fn lint(&self) -> Vec<String> {
        let mut ret = Vec::new();

        if Some(self.sub_if_num) != self.encap {
            ret.push("sub-interface number と encapsulation tag が一致していない".to_string());
        }

        if !self.has_rewrite {
            ret.push("rewrite ingress tag pop 1 symmetric が存在しない".to_string());
        }

        ret
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone)]
pub struct BridgeDomain {
    #[wasm_bindgen(js_name = vlanTag)]
    pub vlan_tag: u32,
    pub interfaces: Vec<String>,
    description: Option<String>,
}

impl BridgeDomain {
    pub fn try_new(block: &Node) -> Option<Self> {
        let node_block = block.as_block()?;

        let vlan_tag = regex!(r"^bridge-domain VLAN(\d+)$")
            .captures(&node_block.name)?
            .get(1)?
            .as_str()
            .parse::<u32>()
            .ok()?;

        let interfaces = node_block
            .stmts()
            .filter_map(|x| x.as_stmt())
            .filter_map(|stmt: &NodeStmt| {
                let caps = regex!(r"^interface (\S+)$").captures(stmt.stmt())?;
                Some(caps.get(1)?.as_str().to_string())
            })
            .collect();
        let description = Self::find_description(node_block);

        Some(BridgeDomain {
            vlan_tag,
            interfaces,
            description,
        })
    }

    fn find_description(node_block: &NodeBlock) -> Option<String> {
        node_block
            .stmts()
            .filter_map(|x| x.as_stmt())
            .find_map(|stmt: &NodeStmt| stmt.stmt().strip_prefix("description "))
            .map(|desc| desc.trim().to_string())
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn lint(&self) -> Vec<String> {
        self.interfaces
            .iter()
            .filter_map(|int| {
                if let Some(bvi_suffix) = int.strip_prefix("BVI") {
                    let Ok(bvi_num) = bvi_suffix.parse::<u32>() else {
                        return None;
                    };

                    if self.vlan_tag != bvi_num {
                        return Some(format!("BVI number がブリッジ名と異なる: {}", int));
                    }
                } else if let Ok((_, sub)) = split_subinterface_id(int) {
                    if Some(self.vlan_tag) != sub {
                        return Some(format!(
                            "sub-interface number がブリッジ名と異なる: {}",
                            int
                        ));
                    }
                }

                None
            })
            .collect()
    }
}

fn get_bridge_domains(config: &[Node]) -> Option<Vec<BridgeDomain>> {
    let res = config
        .iter()
        .find_map(|x| x.as_block().filter(|x| x.name == "l2vpn"))?
        .stmts()
        .find_map(|x| x.as_block().filter(|x| x.name == "bridge group VLAN"))?
        .stmts()
        .filter_map(BridgeDomain::try_new)
        .collect();

    Some(res)
}

fn get_l2_transports(config: &[Node]) -> HashMap<String, Vec<L2TransportConfig>> {
    let mut grouped: HashMap<String, Vec<L2TransportConfig>> = HashMap::new();

    for interface in config.iter().filter_map(L2TransportConfig::try_new) {
        grouped
            .entry(interface.baseif.clone())
            .or_default()
            .push(interface);
    }

    grouped
}

fn collect_simplified_data(config: &[Node], domains: Vec<BridgeDomain>) -> SimplifiedConfigData {
    let mut base_interfaces: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut bvi_interfaces: BTreeMap<String, Option<String>> = BTreeMap::new();

    for node_block in config.iter().filter_map(|node| node.as_block()) {
        let Some(interface_name) = node_block.name.strip_prefix("interface ") else {
            continue;
        };

        if interface_name.contains('.') || node_block.name.ends_with(" l2transport") {
            continue;
        }

        if interface_name.starts_with("BVI") {
            let description = node_block
                .stmts()
                .filter_map(|node| node.as_stmt())
                .find_map(|stmt: &NodeStmt| stmt.stmt().strip_prefix("description "))
                .map(|desc| desc.trim().to_string());
            bvi_interfaces.entry(interface_name.to_string()).or_insert(description);
            continue;
        }

        let stmts = node_block
            .stmts()
            .filter_map(|node| node.as_stmt())
            .map(|stmt: &NodeStmt| stmt.stmt().to_string())
            .collect::<Vec<String>>();

        if !stmts.is_empty() {
            base_interfaces.insert(interface_name.to_string(), stmts);
        }
    }

    SimplifiedConfigData::new(domains, base_interfaces, bvi_interfaces)
}

fn build_lint_output(
    l2transport: &HashMap<String, Vec<L2TransportConfig>>,
    domains: &[BridgeDomain],
) -> String {
    let mut msg = String::new();

    for trans in l2transport.values().flat_map(|v| v.iter()) {
        let res = trans.lint();
        if !res.is_empty() {
            msg.push_str(&format!(
                "--- interface {}.{} l2transport ---\n",
                trans.baseif, trans.sub_if_num
            ));
            msg.push_str(&res.join("\n"));
            msg.push('\n');
        }
    }

    for domain in domains {
        let res = domain.lint();
        if !res.is_empty() {
            msg.push_str(&format!("--- bridge-domain VLAN{} ---\n", domain.vlan_tag));
            msg.push_str(&res.join("\n"));
            msg.push('\n');
        }
    }

    msg
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone)]
pub struct Config {
    pub domains: Vec<BridgeDomain>,
    #[wasm_bindgen(js_name = lintOutput)]
    pub lint_output: String,
    #[wasm_bindgen(js_name = simplifiedConfig)]
    pub simplified_config: String,
}

impl Config {
    pub fn lint(&self) -> String {
        self.lint_output.clone()
    }
}

pub fn analyze(config: &[Node]) -> Config {
    let l2transport = get_l2_transports(config);
    let domains = get_bridge_domains(config).unwrap_or_default();
    let lint_output = build_lint_output(&l2transport, &domains);
    let simplified_data = collect_simplified_data(config, domains.clone());
    let simplified_config = build_simplified_config(&simplified_data);

    Config {
        domains,
        lint_output,
        simplified_config,
    }
}
