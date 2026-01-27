use crate::ast::Span;
use crate::parse::Node;
use crate::semantics::{split_subinterface_id, BridgeDomain};
use std::collections::{BTreeMap, BTreeSet, HashMap};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InterfaceMode {
    Unknown,
    Trunk,
}

impl Default for InterfaceMode {
    fn default() -> Self {
        InterfaceMode::Unknown
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InterfaceChange {
    pub description: Option<String>,
    pub trunk_add: BTreeMap<u32, Span>,    // vlan -> span
    pub trunk_remove: BTreeMap<u32, Span>, // vlan -> span
    pub mode: InterfaceMode,
    pub other_statements: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct ChangeSpec {
    pub vlans: BTreeMap<u32, Option<String>>,
    pub interface_changes: BTreeMap<String, InterfaceChange>,
    pub bvi_additions: BTreeSet<u32>,
    pub interface_spans: HashMap<String, Span>,
}

impl ChangeSpec {
    pub fn interface_span(&self, name: &str) -> Option<Span> {
        self.interface_spans.get(name).copied()
    }
}

#[derive(Clone, Debug, Default)]
pub struct BaseContext {
    pub base_descriptions: HashMap<String, String>,
    pub domain_descriptions: HashMap<u32, Option<String>>,
    pub existing_membership: HashMap<String, BTreeSet<u32>>, // baseif -> vlans
}

impl BaseContext {
    pub fn from_analysis(domains: &[BridgeDomain], nodes: &[Node]) -> Self {
        let mut base_descriptions: HashMap<String, String> = HashMap::new();
        let mut domain_descriptions: HashMap<u32, Option<String>> = HashMap::new();
        let mut domain_interfaces: HashMap<u32, BTreeSet<String>> = HashMap::new();

        for domain in domains {
            domain_descriptions.insert(domain.vlan_tag, domain.description().map(str::to_string));
            for iface in &domain.interfaces {
                domain_interfaces
                    .entry(domain.vlan_tag)
                    .or_default()
                    .insert(iface.clone());
            }
        }

        for node in nodes.iter().filter_map(|n| n.as_block()) {
            if let Some(ifname) = node.name.strip_prefix("interface ") {
                if ifname.contains('.') || node.name.ends_with(" l2transport") {
                    continue;
                }

                let description = node
                    .stmts()
                    .filter_map(|x| x.as_stmt())
                    .find_map(|stmt| stmt.stmt().strip_prefix("description "))
                    .map(|s| s.trim().to_string());

                if let Some(desc) = description {
                    base_descriptions.insert(ifname.to_string(), desc);
                }
            }
        }

        let mut existing_membership: HashMap<String, BTreeSet<u32>> = HashMap::new();
        for (vlan, interfaces) in &domain_interfaces {
            for iface in interfaces {
                if let Ok((baseif, Some(_))) = split_subinterface_id(iface) {
                    existing_membership.entry(baseif).or_default().insert(*vlan);
                }
            }
        }

        BaseContext {
            base_descriptions,
            domain_descriptions,
            existing_membership,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InterfaceCreation {
    pub baseif: String,
    pub vlan: u32,
    pub description: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InterfaceMembership {
    pub baseif: String,
    pub vlan: u32,
}

#[derive(Clone, Debug, Default)]
pub struct VlanChange {
    pub vlan: u32,
    pub description: Option<String>,
    pub removals: BTreeSet<String>,
    pub additions: Vec<InterfaceMembership>,
    pub add_bvi: bool,
}

impl VlanChange {
    pub fn new(vlan: u32, change_spec: &ChangeSpec, base_ctx: &BaseContext) -> Self {
        let description = change_spec.vlans.get(&vlan).cloned().unwrap_or_else(|| {
            base_ctx
                .domain_descriptions
                .get(&vlan)
                .cloned()
                .unwrap_or(None)
        });

        VlanChange {
            vlan,
            description,
            removals: BTreeSet::new(),
            additions: Vec::new(),
            add_bvi: false,
        }
    }

    pub fn record_removal(&mut self, iface: String) {
        self.removals.insert(iface);
    }

    pub fn record_addition(&mut self, baseif: String, vlan: u32) {
        self.additions.push(InterfaceMembership { baseif, vlan });
    }
}

#[derive(Clone, Debug, Default)]
pub struct ChangePlan {
    pub removal_cmds: Vec<String>,
    pub additions: Vec<InterfaceCreation>,
    pub vlan_changes: BTreeMap<u32, VlanChange>,
}
