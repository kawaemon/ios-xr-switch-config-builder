use crate::parse::{Node, NodeBlock, NodeStmt};
use crate::regex;
use std::collections::HashMap;

fn split_subinterface_id(name: &str) -> Result<(String, Option<u32>), String> {
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
            ret.push("sub-interface number と encapsulation tag が一致していません".to_string());
        }

        if !self.has_rewrite {
            ret.push("rewrite ingress tag pop 1 symmetric が存在しない".to_string());
        }

        ret
    }
}

#[derive(Debug, Clone)]
pub struct BridgeDomain {
    pub vlan_tag: u32,
    pub interfaces: Vec<String>,
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

        Some(BridgeDomain {
            vlan_tag,
            interfaces,
        })
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

#[derive(Debug)]
pub struct Config {
    pub l2transport: HashMap<String, Vec<L2TransportConfig>>,
    pub domains: Vec<BridgeDomain>,
}

impl Config {
    pub fn lint(&self) -> String {
        let mut msg = String::new();

        for trans in self.l2transport.values().flat_map(|v| v.iter()) {
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

        for domain in &self.domains {
            let res = domain.lint();
            if !res.is_empty() {
                msg.push_str(&format!("--- bridge-domain VLAN{} ---\n", domain.vlan_tag));
                msg.push_str(&res.join("\n"));
                msg.push('\n');
            }
        }

        msg
    }
}

pub fn analyze(config: &[Node]) -> Config {
    let l2transport = get_l2_transports(config);
    let domains = get_bridge_domains(config).unwrap_or_default();
    Config {
        l2transport,
        domains,
    }
}
