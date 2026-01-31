//! Input parser for change configuration.
//!
//! This module parses simplified change input syntax and converts it into
//! a structured ChangeSpec intermediate representation.

use crate::ast::{Span, Spanned, SpannedNode, SpannedNodeBlock, SpannedNodeStmt};
use crate::change::model::{BaseIf, ChangeSpec, InterfaceChange, VlanId};
use crate::error::{Diagnostic, ErrorKind};
use crate::parse::parser::tokenize_spanned;
use crate::regex;

/// Parse simplified change input text into a `ChangeSpec` structure.
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

/// Process a parsed block node and update the change spec.
fn handle_block(block: SpannedNodeBlock, spec: &mut ChangeSpec) -> Result<(), Diagnostic> {
    if block.name == "vlan database" {
        parse_vlan_block(&block, spec)?;
        return Ok(());
    }

    if let Some(ifname) = block.name.strip_prefix("interface ") {
        let baseif = BaseIf::from(ifname);
        spec.interface_spans.insert(baseif.clone(), block.span);
        parse_interface_block(&baseif, &block, spec)?;
    }

    Ok(())
}

/// Process a standalone statement node and update the change spec.
fn handle_stmt(stmt: crate::ast::SpannedNodeStmt, spec: &mut ChangeSpec) -> Result<(), Diagnostic> {
    let text = stmt.stmt.trim();
    if text.is_empty() {
        return Ok(());
    }

    if text.starts_with("vlan ") {
        parse_vlan_line(text, stmt.span)?.map(|(vlan, name)| {
            spec.vlans.insert(vlan, name);
        });
        return Ok(());
    }

    if let Some(ifname) = text.strip_prefix("interface ") {
        let baseif = BaseIf::from(ifname);
        spec.interface_spans.insert(baseif.clone(), stmt.span);
        if baseif.as_str().starts_with("BVI") {
            parse_interface_stmt(&baseif, spec)?;
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

/// Parse a `vlan database` block, capturing VLAN names when present.
fn parse_vlan_block(block: &SpannedNodeBlock, spec: &mut ChangeSpec) -> Result<(), Diagnostic> {
    for stmt in block.stmts().filter_map(|s| s.as_stmt()) {
        if let Some((vlan, name)) = parse_vlan_line(&stmt.stmt, stmt.span)? {
            spec.vlans.insert(vlan, name);
        }
    }
    Ok(())
}

/// Parse an interface block and populate interface-specific changes.
fn parse_interface_block(
    ifname: &BaseIf,
    block: &SpannedNodeBlock,
    spec: &mut ChangeSpec,
) -> Result<(), Diagnostic> {
    parse_interface_stmt(ifname, spec)?;

    if ifname.as_str().starts_with("BVI") {
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
                interface_change.description = Some(Spanned::new(desc.clone(), stmt.span));
                interface_change.other_statements.push(SpannedNodeStmt {
                    stmt: format!("description {}", desc),
                    span: stmt.span,
                });
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
            interface_change.other_statements.push(SpannedNodeStmt {
                stmt: stmt_text.to_string(),
                span: stmt.span,
            });
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
        .insert(ifname.clone(), interface_change);

    Ok(())
}

/// Parse statements under a BVI interface block.
fn parse_bvi_block(
    ifname: &BaseIf,
    block: &SpannedNodeBlock,
    spec: &mut ChangeSpec,
) -> Result<(), Diagnostic> {
    if let Some(vlan) = ifname.as_str().strip_prefix("BVI") {
        let vlan_id = vlan.parse::<u32>().map_err(|_| {
            Diagnostic::new(ErrorKind::InvalidBviNumber {
                text: vlan.to_string(),
            })
        })?;

        let mut statements = Vec::new();
        for stmt in block.stmts().filter_map(|s| s.as_stmt()) {
            let stmt_text = stmt.stmt.trim_end();
            if !stmt_text.is_empty() {
                statements.push(Spanned::new(stmt_text.to_string(), stmt.span));
            }
        }

        spec.bvi_statements.insert(VlanId::new(vlan_id), statements);
    }

    Ok(())
}

/// Handle an interface declaration that is not part of a block (e.g., BVI lines).
fn parse_interface_stmt(ifname: &BaseIf, spec: &mut ChangeSpec) -> Result<(), Diagnostic> {
    if let Some(vlan) = ifname.as_str().strip_prefix("BVI") {
        let vlan_id = vlan.parse::<u32>().map_err(|_| {
            Diagnostic::new(ErrorKind::InvalidBviNumber {
                text: vlan.to_string(),
            })
        })?;
        spec.bvi_additions.insert(VlanId::new(vlan_id));
    }

    Ok(())
}

/// Parse a single `vlan` statement, returning the VLAN ID and optional name.
fn parse_vlan_line(
    line: &str,
    span: Span,
) -> Result<Option<(VlanId, Option<Spanned<String>>)>, Diagnostic> {
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
            diag.span = Some(span);
            return Err(diag);
        }
        return Ok(Some((VlanId::new(vlan), Some(Spanned::new(name, span)))));
    }

    if regex!(r"^vlan\s+\d+\s*$").captures(line).is_some() {
        let mut diag = Diagnostic::new(ErrorKind::VlanNameRequired { vlan: None });
        diag.span = Some(span);
        return Err(diag);
    }

    Ok(None)
}

/// Apply trunk add/remove actions to an interface change.
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

/// Parse a whitespace-separated VLAN list (supports ranges like `300-305`).
fn parse_vlan_list(list: &str) -> Result<Vec<VlanId>, Diagnostic> {
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
                vlans.push(VlanId::new(vlan));
            }
        } else {
            // 単一のVLAN番号
            let vlan = token.parse::<u32>().map_err(|_| {
                Diagnostic::new(ErrorKind::InvalidVlanNumber {
                    text: token.to_string(),
                })
            })?;
            vlans.push(VlanId::new(vlan));
        }
    }

    Ok(vlans)
}

/// Normalize indentation so parsing is independent of leading spaces.
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

/// Count leading space characters on a line.
fn count_leading_spaces(text: &str) -> usize {
    text.chars().take_while(|&c| c == ' ').count()
}
