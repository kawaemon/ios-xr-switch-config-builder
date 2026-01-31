//! IOS XR command generation.
//!
//! This module generates the actual IOS XR CLI commands needed to apply
//! the planned changes to the switch configuration.

use crate::change::model::{ChangePlan, ChangeSpec};

pub fn generate_commands(plan: &ChangePlan, change_spec: &ChangeSpec) -> String {
    let mut lines: Vec<String> = Vec::new();

    // Output physical interface configuration changes
    for (baseif, change) in &change_spec.interface_changes {
        if !change.other_statements.is_empty() {
            lines.push(format!("interface {}", baseif));
            for stmt in &change.other_statements {
                lines.push(format!("  {}", stmt));
            }
            lines.push("exit".to_string());
            lines.push(String::new());
        }
    }

    // Output BVI interface configuration changes
    for (vlan, statements) in &change_spec.bvi_statements {
        if !statements.is_empty() {
            lines.push(format!("interface BVI{}", vlan));
            for stmt in statements {
                lines.push(format!("  {}", stmt));
            }
            lines.push("exit".to_string());
            lines.push(String::new());
        }
    }

    if !plan.removal_cmds.is_empty() {
        lines.extend(plan.removal_cmds.clone());
        lines.push(String::new());
    }

    for addition in &plan.additions {
        lines.push(format!(
            "interface {}.{} l2transport",
            addition.baseif, addition.vlan
        ));
        lines.push(format!("  description {}", addition.description));
        lines.push(format!("  encapsulation dot1q {}", addition.vlan));
        lines.push("  rewrite ingress tag pop 1 symmetric".to_string());
        lines.push("exit".to_string());
        lines.push(String::new());
    }

    if !plan.vlan_changes.is_empty() {
        lines.push("l2vpn".to_string());
        lines.push("  bridge group VLAN".to_string());
        for (_, change) in &plan.vlan_changes {
            lines.push(format!("    bridge-domain VLAN{}", change.vlan));
            // Only output description if explicitly specified in change_spec
            if change_spec.vlans.contains_key(&change.vlan) {
                if let Some(desc) = &change.description {
                    if !desc.is_empty() {
                        lines.push(format!("      description {}", desc));
                    }
                }
            }
            for removal in &change.removals {
                lines.push(format!("      no interface {}", removal));
            }
            for addition in &change.additions {
                lines.push(format!(
                    "      interface {}.{}",
                    addition.baseif, addition.vlan
                ));
                lines.push("      exit".to_string());
            }
            if change.add_bvi {
                lines.push(format!("      routed interface BVI{}", change.vlan));
                lines.push("      exit".to_string());
            }
            lines.push("    exit".to_string());
        }
        lines.push("  exit".to_string());
        lines.push("exit".to_string());
    }

    trim_trailing_empty_lines(&mut lines);

    lines.join("\n")
}

fn trim_trailing_empty_lines(lines: &mut Vec<String>) {
    while matches!(lines.last(), Some(last) if last.is_empty()) {
        lines.pop();
    }
}
