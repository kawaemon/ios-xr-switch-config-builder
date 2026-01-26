use crate::parse::{tokenize, NodeBlock};
use crate::regex;
use crate::semantics::{analyze, split_subinterface_id, BridgeDomain};
use std::collections::{BTreeMap, BTreeSet, HashMap};

#[derive(Default)]
struct InterfaceChange {
    description: Option<String>,
    trunk_add: BTreeMap<u32, usize>,      // vlan -> line number
    trunk_remove: BTreeMap<u32, usize>,   // vlan -> line number
    mode: InterfaceMode,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum InterfaceMode {
    Unknown,
    Trunk,
}

impl Default for InterfaceMode {
    fn default() -> Self {
        InterfaceMode::Unknown
    }
}

#[derive(Default)]
struct ChangeSpec {
    vlans: BTreeMap<u32, Option<String>>,
    interface_changes: BTreeMap<String, InterfaceChange>,
    bvi_additions: BTreeSet<u32>,
    interface_lines: HashMap<String, usize>,
    stmt_lines: HashMap<String, Vec<usize>>,  // stmt text -> line numbers (can have duplicates)
}

struct BaseContext {
    base_descriptions: HashMap<String, String>,
    domain_descriptions: HashMap<u32, Option<String>>,
    existing_membership: HashMap<String, BTreeSet<u32>>, // baseif -> vlans
}

struct InterfaceCreation {
    baseif: String,
    vlan: u32,
    description: String,
}

struct InterfaceMembership {
    baseif: String,
    vlan: u32,
}

struct VlanChange {
    vlan: u32,
    description: Option<String>,
    removals: BTreeSet<String>,
    additions: Vec<InterfaceMembership>,
    add_bvi: bool,
}

pub fn generate_change(base_config: &str, change_input: &str) -> Result<String, String> {
    let base_nodes = tokenize(base_config);
    let analysis = analyze(&base_nodes);

    let base_ctx = build_base_context(&analysis.domains, &base_nodes);
    let change_spec = parse_change_input(change_input)?;

    let mut removal_cmds: Vec<String> = Vec::new();
    let mut additions: Vec<InterfaceCreation> = Vec::new();
    let mut vlan_changes: BTreeMap<u32, VlanChange> = BTreeMap::new();

    for (baseif, change) in &change_spec.interface_changes {
        let existing = base_ctx
            .existing_membership
            .get(baseif)
            .cloned()
            .unwrap_or_default();

        // Validate that VLANs to be removed actually exist
        for (vlan, line_no) in &change.trunk_remove {
            if !existing.contains(vlan) {
                return Err(format!(
                    "cannot remove VLAN {} from interface {}: VLAN not present in base config{}",
                    vlan,
                    baseif,
                    format_line_suffix(Some(*line_no))
                ));
            }
        }

        let desired = desired_vlans(change, &existing)?;

        let base_desc = change
            .description
            .as_ref()
            .or_else(|| base_ctx.base_descriptions.get(baseif))
            .cloned()
            .ok_or_else(|| {
                let line_no = change_spec.interface_lines.get(baseif).copied();
                format!(
                    "interface requires description: {}{}",
                    baseif,
                    format_line_suffix(line_no)
                )
            })?;

        for vlan in existing.difference(&desired) {
            let iface = format!("{}.{}", baseif, vlan);
            removal_cmds.push(format!("no interface {} l2transport", iface));

            vlan_changes
                .entry(*vlan)
                .or_insert_with(|| VlanChange::new(*vlan, &change_spec, &base_ctx))
                .removals
                .insert(iface);
        }

        for vlan in desired.difference(&existing) {
            additions.push(InterfaceCreation {
                baseif: baseif.clone(),
                vlan: *vlan,
                description: build_subinterface_description(
                    *vlan,
                    &base_desc,
                    &change_spec,
                    &base_ctx,
                ),
            });

            vlan_changes
                .entry(*vlan)
                .or_insert_with(|| VlanChange::new(*vlan, &change_spec, &base_ctx))
                .additions
                .push(InterfaceMembership {
                    baseif: baseif.clone(),
                    vlan: *vlan,
                });
        }
    }

    for vlan in &change_spec.bvi_additions {
        vlan_changes
            .entry(*vlan)
            .or_insert_with(|| VlanChange::new(*vlan, &change_spec, &base_ctx))
            .add_bvi = true;
    }

    let mut lines: Vec<String> = Vec::new();

    if !removal_cmds.is_empty() {
        lines.extend(removal_cmds);
        lines.push(String::new());
    }

    for addition in &additions {
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

    if !vlan_changes.is_empty() {
        lines.push("l2vpn".to_string());
        lines.push("  bridge group VLAN".to_string());
        for (_, change) in &vlan_changes {
            lines.push(format!("    bridge-domain VLAN{}", change.vlan));
            if let Some(desc) = &change.description {
                if !desc.is_empty() {
                    lines.push(format!("      description {}", desc));
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

    Ok(lines.join("\n"))
}

fn build_base_context(domains: &[BridgeDomain], nodes: &[crate::parse::Node]) -> BaseContext {
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

fn desired_vlans(
    change: &InterfaceChange,
    existing: &BTreeSet<u32>,
) -> Result<BTreeSet<u32>, String> {
    let mut result = existing.clone();
    for vlan in change.trunk_add.keys() {
        result.insert(*vlan);
    }
    for vlan in change.trunk_remove.keys() {
        result.remove(vlan);
    }
    Ok(result)
}

fn parse_change_input(input: &str) -> Result<ChangeSpec, String> {
    let mut spec = ChangeSpec::default();
    let normalized_input = normalize_indent(input);

    prevalidate_lines(&normalized_input, &mut spec)?;

    let nodes = tokenize(&normalized_input);

    for node in nodes {
        if let Some(block) = node.as_block() {
            if block.name == "vlan database" {
                parse_vlan_block(block, &mut spec)?;
                continue;
            }

            if let Some(ifname) = block.name.strip_prefix("interface ") {
                let line_no = spec.interface_lines.get(ifname).copied();
                parse_interface_block(ifname, block, line_no, &mut spec)?;
            }
            continue;
        }

        if let Some(stmt) = node.as_stmt() {
            let text = stmt.stmt();
            if let Some(ifname) = text.strip_prefix("interface ") {
                if ifname.starts_with("BVI") {
                    parse_interface_stmt(ifname, &mut spec)?;
                } else {
                    let line_no = spec.interface_lines.get(ifname).copied();
                    return Err(format!(
                        "interface block must contain supported statements{}",
                        format_line_suffix(line_no)
                    ));
                }
            }
        }
    }

    Ok(spec)
}

fn prevalidate_lines(input: &str, spec: &mut ChangeSpec) -> Result<(), String> {
    for (index, line) in input.lines().enumerate() {
        let line_no = index + 1;
        let trimmed = line.trim_end();
        if trimmed.trim().is_empty() {
            continue;
        }

        if trimmed.starts_with("vlan ") {
            parse_vlan_line(trimmed, Some(line_no))?;
            continue;
        }

        if let Some(ifname) = trimmed.strip_prefix("interface ") {
            spec.interface_lines
                .entry(ifname.to_string())
                .or_insert(line_no);
        }

        // Track line numbers for all statements (indented lines)
        let trimmed_stmt = trimmed.trim();
        if !trimmed_stmt.is_empty() && trimmed != trimmed_stmt {
            // This is an indented statement
            spec.stmt_lines
                .entry(trimmed_stmt.to_string())
                .or_insert_with(Vec::new)
                .push(line_no);
        }
    }

    Ok(())
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

fn format_line_suffix(line_no: Option<usize>) -> String {
    match line_no {
        Some(no) => format!(" (line {})", no),
        None => String::new(),
    }
}

fn parse_vlan_block(block: &NodeBlock, spec: &mut ChangeSpec) -> Result<(), String> {
    for stmt in block.stmts().filter_map(|s| s.as_stmt()) {
        if let Some((vlan, name)) = parse_vlan_line(stmt.stmt(), None)? {
            spec.vlans.insert(vlan, name);
        }
    }
    Ok(())
}

fn parse_interface_block(
    ifname: &str,
    block: &NodeBlock,
    line_no: Option<usize>,
    spec: &mut ChangeSpec,
) -> Result<(), String> {
    parse_interface_stmt(ifname, spec)?;

    if ifname.starts_with("BVI") {
        return Ok(());
    }

    let mut interface_change = spec.interface_changes.remove(ifname).unwrap_or_default();

    let mut has_supported_stmt = false;
    let desc_re = regex!(r"^description\s+(.+)$");
    let mode_re = regex!(r"^switchport mode\s+(trunk)$");
    let mode_any_re = regex!(r"^switchport mode\s+(.+)$");
    let trunk_re = regex!(r"^switchport trunk allowed vlan (add|remove)\s+(.+)$");

    for stmt in block.stmts().filter_map(|s| s.as_stmt()) {
        let stmt_text = stmt.stmt();

        if let Some(caps) = desc_re.captures(stmt_text) {
            let desc = caps
                .get(1)
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default();
            if !desc.is_empty() {
                interface_change.description = Some(desc);
            }
            has_supported_stmt = true;
            continue;
        }

        if mode_re.captures(stmt_text).is_some() {
            update_mode(&mut interface_change, InterfaceMode::Trunk)?;
            has_supported_stmt = true;
            continue;
        }

        if let Some(caps) = mode_any_re.captures(stmt_text) {
            let mode = caps
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            return Err(format!(
                "switchport mode {} is not supported{}",
                mode,
                format_line_suffix(line_no)
            ));
        }

        if stmt_text.starts_with("switchport access vlan") {
            return Err(format!(
                "switchport access is not supported{}",
                format_line_suffix(line_no)
            ));
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
            update_mode(&mut interface_change, InterfaceMode::Trunk)?;

            // Get line number for this statement
            let stmt_line_no = spec.stmt_lines
                .get(stmt_text)
                .and_then(|lines| lines.first().copied())
                .or(line_no)
                .unwrap_or(1);

            apply_trunk_action(&mut interface_change, &action, &list, stmt_line_no)?;
            has_supported_stmt = true;
            continue;
        }
    }

    if !has_supported_stmt {
        return Err(format!(
            "interface block must contain supported statements{}",
            format_line_suffix(line_no)
        ));
    }

    spec.interface_changes
        .insert(ifname.to_string(), interface_change);

    Ok(())
}

fn parse_interface_stmt(ifname: &str, spec: &mut ChangeSpec) -> Result<(), String> {
    if let Some(vlan) = ifname.strip_prefix("BVI") {
        let vlan_id = vlan
            .parse::<u32>()
            .map_err(|_| "invalid BVI number".to_string())?;
        spec.bvi_additions.insert(vlan_id);
    }

    Ok(())
}

fn parse_vlan_line(
    line: &str,
    line_no: Option<usize>,
) -> Result<Option<(u32, Option<String>)>, String> {
    if !line.starts_with("vlan ") {
        return Ok(None);
    }

    if let Some(caps) = regex!(r"^vlan\s+(\d+)\s+name\s+(.+)$").captures(line) {
        let vlan = caps
            .get(1)
            .and_then(|m| m.as_str().parse::<u32>().ok())
            .ok_or_else(|| "invalid vlan id".to_string())?;
        let name = caps
            .get(2)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();
        if name.is_empty() {
            return Err(format!(
                "vlan name is required{}",
                format_line_suffix(line_no)
            ));
        }
        return Ok(Some((vlan, Some(name))));
    }

    if regex!(r"^vlan\s+\d+\s*$").captures(line).is_some() {
        return Err(format!(
            "vlan name is required{}",
            format_line_suffix(line_no)
        ));
    }

    Ok(None)
}

fn apply_trunk_action(
    change: &mut InterfaceChange,
    action: &str,
    list: &str,
    line_no: usize,
) -> Result<(), String> {
    let vlans = parse_vlan_list(list)?;

    match action {
        "add" => {
            for vlan in vlans {
                change.trunk_add.insert(vlan, line_no);
            }
        }
        "remove" => {
            for vlan in vlans {
                change.trunk_remove.insert(vlan, line_no);
            }
        }
        _ => return Err("invalid trunk action".to_string()),
    }

    Ok(())
}

fn parse_vlan_list(list: &str) -> Result<Vec<u32>, String> {
    if list.trim().is_empty() {
        return Err("vlan list is empty".to_string());
    }

    list.split_whitespace()
        .map(|v| {
            v.parse::<u32>()
                .map_err(|_| "invalid vlan number".to_string())
        })
        .collect()
}

fn update_mode(change: &mut InterfaceChange, next: InterfaceMode) -> Result<(), String> {
    if change.mode != InterfaceMode::Unknown && change.mode != next {
        return Err("interface mode is conflicting".to_string());
    }

    change.mode = next;
    Ok(())
}

fn build_subinterface_description(
    vlan: u32,
    base_desc: &str,
    change_spec: &ChangeSpec,
    base_ctx: &BaseContext,
) -> String {
    let vlan_desc = change_spec
        .vlans
        .get(&vlan)
        .and_then(|v| v.as_deref())
        .or_else(|| {
            base_ctx
                .domain_descriptions
                .get(&vlan)
                .and_then(|v| v.as_deref())
        });

    match vlan_desc {
        Some(v) => format!("{},{}", v, base_desc),
        None => base_desc.to_string(),
    }
}

fn trim_trailing_empty_lines(lines: &mut Vec<String>) {
    while matches!(lines.last(), Some(last) if last.is_empty()) {
        lines.pop();
    }
}

impl VlanChange {
    fn new(vlan: u32, change_spec: &ChangeSpec, base_ctx: &BaseContext) -> Self {
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
}
