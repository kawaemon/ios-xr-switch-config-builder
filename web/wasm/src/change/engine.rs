use crate::change::codegen;
use crate::change::input_parser::parse_change_input;
use crate::change::model::BaseContext;
use crate::change::planner::ChangePlanner;
use crate::error::Diagnostic;
use crate::parse::tokenize;
use crate::semantics::analyze;

/// High-level entry point for generating IOS XR change commands.
pub struct ChangeEngine;

impl ChangeEngine {
    /// Generate CLI commands from base configuration text and simplified change input.
    pub fn generate(base_config: &str, change_input: &str) -> Result<String, Diagnostic> {
        let base_nodes = tokenize(base_config);
        let analysis = analyze(&base_nodes);
        let base_ctx = BaseContext::from_analysis(&analysis.domains, &base_nodes);

        let change_spec = parse_change_input(change_input)?;
        let planner = ChangePlanner::new(&change_spec, &base_ctx);
        let plan = planner.plan()?;
        let rendered_change = codegen::generate_commands(&plan, &change_spec);

        Ok(rendered_change)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn change_engine_generates_commands() {
        let base_config = r#"
interface FortyGigE0/0/0/46
  description To:demo-port

l2vpn
  bridge group VLAN
    bridge-domain VLAN300
      interface FortyGigE0/0/0/46.300
    exit
  exit
exit
"#;

        let change_input = [
            "interface FortyGigE0/0/0/46",
            "  description To:demo-port",
            "  switchport mode trunk",
            "  switchport trunk allowed vlan add 350",
            "  switchport trunk allowed vlan remove 300",
            "",
            "interface BVI500",
            "",
            "vlan database",
            "  vlan 350 name demo-servers",
            "  vlan 500 name demo-mgmt",
        ]
        .join("\n");

        let rendered =
            ChangeEngine::generate(base_config, &change_input).expect("generation succeeds");

        assert!(
            rendered.contains("no interface FortyGigE0/0/0/46.300 l2transport"),
            "removal command present"
        );
        assert!(
            rendered.contains("interface FortyGigE0/0/0/46.350 l2transport"),
            "addition command present"
        );
        assert!(
            rendered.contains("description demo-servers,To:demo-port"),
            "subinterface description derived from vlan + base description"
        );
        assert!(
            rendered.contains("bridge-domain VLAN350"),
            "bridge domain for new vlan emitted"
        );
    }
}
