//! Input parser for change configuration.
//!
//! This module parses simplified change input syntax and converts it into
//! a structured ChangeSpec intermediate representation.

use crate::ast::{Span, SpannedNode, SpannedNodeBlock};
use crate::change::model::{ChangeSpec, InterfaceChange};
use crate::error::{Diagnostic, ErrorKind};
use crate::parse::parser::tokenize_spanned;
use crate::regex;

pub fn parse_change_input(input: &str) -> Result<ChangeSpec, Diagnostic> {
    let normalized_input = normalize_indent(input);
    let nodes = tokenize_spanned(&normalized_input);

    let mut spec = ChangeSpec::default();

    for node in nodes {
        match node {
            SpannedNode::Block(block) => handle_block(block, &mut spec)?,
            SpannedNode::Stmt(stmt) => handle_stmt(stmt, &mut spec)?,
        }
    }

    Ok(spec)
}

fn handle_block(block: SpannedNodeBlock, spec: &mut ChangeSpec) -> Result<(), Diagnostic> {
    if block.name == "vlan database" {
        parse_vlan_block(&block, spec)?;
        return Ok(());
    }

    if let Some(ifname) = block.name.strip_prefix("interface ") {
        spec.interface_spans.insert(ifname.to_string(), block.span);
        parse_interface_block(ifname, &block, spec)?;
    }

    Ok(())
}

fn handle_stmt(stmt: crate::ast::SpannedNodeStmt, spec: &mut ChangeSpec) -> Result<(), Diagnostic> {
    let text = stmt.stmt.trim();
    if text.is_empty() {
        return Ok(());
    }

    if text.starts_with("vlan ") {
        parse_vlan_line(text, Some(stmt.span))?.map(|(vlan, name)| {
            spec.vlans.insert(vlan, name);
        });
        return Ok(());
    }

    if let Some(ifname) = text.strip_prefix("interface ") {
        spec.interface_spans.insert(ifname.to_string(), stmt.span);
        if ifname.starts_with("BVI") {
            parse_interface_stmt(ifname, spec)?;
        } else {
            let mut diag = Diagnostic::new(ErrorKind::InterfaceRequiresStatements {
                interface: ifname.to_string(),
            });
            diag.span = Some(stmt.span);
            return Err(diag);
        }
    }

    Ok(())
}

fn parse_vlan_block(block: &SpannedNodeBlock, spec: &mut ChangeSpec) -> Result<(), Diagnostic> {
    for stmt in block.stmts().filter_map(|s| s.as_stmt()) {
        if let Some((vlan, name)) = parse_vlan_line(&stmt.stmt, Some(stmt.span))? {
            spec.vlans.insert(vlan, name);
        }
    }
    Ok(())
}

fn parse_interface_block(
    ifname: &str,
    block: &SpannedNodeBlock,
    spec: &mut ChangeSpec,
) -> Result<(), Diagnostic> {
    parse_interface_stmt(ifname, spec)?;

    if ifname.starts_with("BVI") {
        parse_bvi_block(ifname, block, spec)?;
        return Ok(());
    }

    let mut interface_change = spec.interface_changes.remove(ifname).unwrap_or_default();
    let mut has_supported_stmt = false;

    let desc_re = regex!(r"^description\s+(.+)$");
    let mode_re = regex!(r"^switchport mode\s+(trunk)$");
    let mode_any_re = regex!(r"^switchport mode\s+(.+)$");
    let trunk_re = regex!(r"^switchport trunk allowed vlan (add|remove)\s+(.+)$");

    for stmt in block.stmts().filter_map(|s| s.as_stmt()) {
        let stmt_text = stmt.stmt.trim_end();

        if let Some(caps) = desc_re.captures(stmt_text) {
            let desc = caps
                .get(1)
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default();
            if !desc.is_empty() {
                interface_change.description = Some(desc.clone());
                interface_change
                    .other_statements
                    .push(format!("description {}", desc));
            }
            has_supported_stmt = true;
            continue;
        }

        if mode_re.captures(stmt_text).is_some() {
            // Ignore "switchport mode trunk" - all ports are trunk by default
            has_supported_stmt = true;
            continue;
        }

        if let Some(caps) = mode_any_re.captures(stmt_text) {
            let mode = caps
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let mut diag = Diagnostic::new(ErrorKind::UnsupportedSwitchportMode { mode });
            diag.span = Some(stmt.span);
            return Err(diag);
        }

        if stmt_text.starts_with("switchport access vlan") {
            let mut diag = Diagnostic::new(ErrorKind::AccessModeNotSupported);
            diag.span = Some(stmt.span);
            return Err(diag);
        }

        if let Some(caps) = trunk_re.captures(stmt_text) {
            let action = caps
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let list = caps
                .get(2)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();

            apply_trunk_action(&mut interface_change, &action, &list, stmt.span)?;
            has_supported_stmt = true;
            continue;
        }

        if !stmt_text.starts_with("switchport") {
            interface_change
                .other_statements
                .push(stmt_text.to_string());
            has_supported_stmt = true;
        }
    }

    if !has_supported_stmt {
        let mut diag = Diagnostic::new(ErrorKind::InterfaceRequiresStatements {
            interface: ifname.to_string(),
        });
        diag.span = Some(block.span);
        return Err(diag);
    }

    spec.interface_changes
        .insert(ifname.to_string(), interface_change);

    Ok(())
}

fn parse_bvi_block(
    ifname: &str,
    block: &SpannedNodeBlock,
    spec: &mut ChangeSpec,
) -> Result<(), Diagnostic> {
    if let Some(vlan) = ifname.strip_prefix("BVI") {
        let vlan_id = vlan.parse::<u32>().map_err(|_| {
            Diagnostic::new(ErrorKind::InvalidBviNumber {
                text: vlan.to_string(),
            })
        })?;

        let mut statements = Vec::new();
        for stmt in block.stmts().filter_map(|s| s.as_stmt()) {
            let stmt_text = stmt.stmt.trim_end();
            if !stmt_text.is_empty() {
                statements.push(stmt_text.to_string());
            }
        }

        spec.bvi_statements.insert(vlan_id, statements);
    }

    Ok(())
}

fn parse_interface_stmt(ifname: &str, spec: &mut ChangeSpec) -> Result<(), Diagnostic> {
    if let Some(vlan) = ifname.strip_prefix("BVI") {
        let vlan_id = vlan.parse::<u32>().map_err(|_| {
            Diagnostic::new(ErrorKind::InvalidBviNumber {
                text: vlan.to_string(),
            })
        })?;
        spec.bvi_additions.insert(vlan_id);
    }

    Ok(())
}

fn parse_vlan_line(
    line: &str,
    span: Option<Span>,
) -> Result<Option<(u32, Option<String>)>, Diagnostic> {
    if !line.starts_with("vlan ") {
        return Ok(None);
    }

    if let Some(caps) = regex!(r"^vlan\s+(\d+)\s+name\s+(.+)$").captures(line) {
        let vlan = caps
            .get(1)
            .and_then(|m| m.as_str().parse::<u32>().ok())
            .ok_or_else(|| {
                Diagnostic::new(ErrorKind::InvalidVlanId {
                    text: line.to_string(),
                })
            })?;
        let name = caps
            .get(2)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();
        if name.is_empty() {
            let mut diag = Diagnostic::new(ErrorKind::VlanNameRequired { vlan: Some(vlan) });
            if let Some(span) = span {
                diag.span = Some(span);
            }
            return Err(diag);
        }
        return Ok(Some((vlan, Some(name))));
    }

    if regex!(r"^vlan\s+\d+\s*$").captures(line).is_some() {
        let mut diag = Diagnostic::new(ErrorKind::VlanNameRequired { vlan: None });
        if let Some(span) = span {
            diag.span = Some(span);
        }
        return Err(diag);
    }

    Ok(None)
}

fn apply_trunk_action(
    change: &mut InterfaceChange,
    action: &str,
    list: &str,
    span: Span,
) -> Result<(), Diagnostic> {
    let vlans = parse_vlan_list(list)?;

    match action {
        "add" => {
            for vlan in vlans {
                change.trunk_add.insert(vlan, span);
            }
        }
        "remove" => {
            for vlan in vlans {
                change.trunk_remove.insert(vlan, span);
            }
        }
        _ => {
            return Err(Diagnostic::new(ErrorKind::InvalidTrunkAction {
                action: action.to_string(),
            }))
        }
    }

    Ok(())
}

fn parse_vlan_list(list: &str) -> Result<Vec<u32>, Diagnostic> {
    if list.trim().is_empty() {
        return Err(Diagnostic::new(ErrorKind::VlanListEmpty));
    }

    let mut vlans = Vec::new();

    for token in list.split_whitespace() {
        if token.contains('-') {
            // 範囲指定: 302-308
            let parts: Vec<&str> = token.split('-').collect();
            if parts.len() != 2 {
                return Err(Diagnostic::new(ErrorKind::InvalidVlanNumber {
                    text: token.to_string(),
                }));
            }

            let start = parts[0].parse::<u32>().map_err(|_| {
                Diagnostic::new(ErrorKind::InvalidVlanNumber {
                    text: token.to_string(),
                })
            })?;

            let end = parts[1].parse::<u32>().map_err(|_| {
                Diagnostic::new(ErrorKind::InvalidVlanNumber {
                    text: token.to_string(),
                })
            })?;

            if start > end {
                return Err(Diagnostic::new(ErrorKind::InvalidVlanRange {
                    text: token.to_string(),
                }));
            }

            for vlan in start..=end {
                vlans.push(vlan);
            }
        } else {
            // 単一のVLAN番号
            let vlan = token.parse::<u32>().map_err(|_| {
                Diagnostic::new(ErrorKind::InvalidVlanNumber {
                    text: token.to_string(),
                })
            })?;
            vlans.push(vlan);
        }
    }

    Ok(vlans)
}


fn normalize_indent(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let min_indent = lines
        .iter()
        .filter_map(|line| {
            let trimmed_end = line.trim_end();
            if trimmed_end.trim().is_empty() {
                None
            } else {
                Some(count_leading_spaces(trimmed_end))
            }
        })
        .min()
        .unwrap_or(0);

    lines
        .iter()
        .map(|line| {
            let trimmed_end = line.trim_end();
            let drop = min_indent.min(count_leading_spaces(trimmed_end));
            trimmed_end.chars().skip(drop).collect::<String>()
        })
        .collect::<Vec<String>>()
        .join("\n")
}

fn count_leading_spaces(text: &str) -> usize {
    text.chars().take_while(|&c| c == ' ').count()
}
