use crate::semantics::{split_subinterface_id, BridgeDomain};
use std::collections::{BTreeMap, BTreeSet};

fn format_vlan_ranges(tags: &BTreeSet<u32>) -> String {
    let mut iter = tags.iter();
    let Some(&first) = iter.next() else {
        return "-".to_string();
    };

    let mut segments: Vec<String> = Vec::new();
    let mut range_start = first;
    let mut prev = first;

    for &current in iter {
        if current != prev + 1 {
            if range_start == prev {
                segments.push(range_start.to_string());
            } else {
                segments.push(format!("{}-{}", range_start, prev));
            }
            range_start = current;
        }
        prev = current;
    }

    if range_start == prev {
        segments.push(range_start.to_string());
    } else {
        segments.push(format!("{}-{}", range_start, prev));
    }

    segments.join(" ")
}

pub struct SimplifiedConfigData {
    pub domains: Vec<BridgeDomain>,
    pub base_interfaces: BTreeMap<String, Vec<String>>,
    pub bvi_interfaces: BTreeMap<String, Option<String>>,
    pub bundle_members: BTreeMap<String, BTreeSet<String>>,
}

impl SimplifiedConfigData {
    pub fn new(
        domains: Vec<BridgeDomain>,
        base_interfaces: BTreeMap<String, Vec<String>>,
        bvi_interfaces: BTreeMap<String, Option<String>>,
        bundle_members: BTreeMap<String, BTreeSet<String>>,
    ) -> Self {
        Self {
            domains,
            base_interfaces,
            bvi_interfaces,
            bundle_members,
        }
    }
}

pub fn build_simplified_config(data: &SimplifiedConfigData) -> String {
    let mut vlan_map: BTreeMap<u32, Option<String>> = BTreeMap::new();
    let mut trunk_map: BTreeMap<String, BTreeSet<u32>> = BTreeMap::new();

    for domain in &data.domains {
        let entry = vlan_map.entry(domain.vlan_tag).or_insert(None);
        if entry.is_none() {
            *entry = domain.description().map(str::to_string);
        }

        for interface in &domain.interfaces {
            let Ok((base_interface, Some(_))) = split_subinterface_id(interface) else {
                continue;
            };

            trunk_map
                .entry(base_interface)
                .or_default()
                .insert(domain.vlan_tag);
        }
    }

    let mut lines: Vec<String> = Vec::new();

    if !trunk_map.is_empty() {
        for (base_interface, vlan_tags) in trunk_map {
            let vlan_list = format_vlan_ranges(&vlan_tags);
            lines.push(format!("interface {}", base_interface));
            if let Some(stmts) = data.base_interfaces.get(&base_interface) {
                for stmt in stmts {
                    lines.push(format!("  {}", stmt));
                }
            }
            lines.push("  switchport mode trunk".to_string());
            lines.push(format!("  switchport trunk allowed vlan add {}", vlan_list));
            lines.push(String::new());

            if let Some(members) = data.bundle_members.get(&base_interface) {
                for member in members {
                    lines.push(format!("interface {}", member));
                    if let Some(stmts) = data.base_interfaces.get(member) {
                        for stmt in stmts {
                            lines.push(format!("  {}", stmt));
                        }
                    }
                    lines.push(String::new());
                }
            }
        }
    }

    if !data.bvi_interfaces.is_empty() {
        if lines.last().map(|line| !line.is_empty()).unwrap_or(false) {
            lines.push(String::new());
        }
        for (name, description) in &data.bvi_interfaces {
            lines.push(format!("interface {}", name));
            if let Some(description) = description {
                lines.push(format!("  description {}", description));
            }
            lines.push("  ! -- L3 config reduced --".to_string());
            lines.push(String::new());
        }
    }

    if !vlan_map.is_empty() {
        if lines.last().map(|line| !line.is_empty()).unwrap_or(false) {
            lines.push(String::new());
        }
        lines.push("vlan database".to_string());
        for (vlan_tag, description) in vlan_map {
            if let Some(description) = description {
                lines.push(format!("  vlan {} name {}", vlan_tag, description));
            } else {
                lines.push(format!("  vlan {}", vlan_tag));
            }
        }
    }

    while lines.last().map(|line| line.is_empty()).unwrap_or(false) {
        lines.pop();
    }

    lines.join("\n")
}
