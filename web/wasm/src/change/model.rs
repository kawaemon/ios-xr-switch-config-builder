use crate::ast::{Span, Spanned, SpannedNodeStmt};
use crate::parse::Node;
use crate::semantics::{split_subinterface_id, BridgeDomain};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt;

/// Base (non-subinterface) identifier such as `FortyGigE0/0/0/1`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BaseIf(String);

impl BaseIf {
    /// Create a new base interface wrapper from any string-like value.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Return the underlying interface name as a `&str`.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for BaseIf {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for BaseIf {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for BaseIf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// VLAN identifier wrapper used for type safety and ordering.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct VlanId(u32);

impl VlanId {
    /// Construct a new VLAN ID.
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    /// Read the raw VLAN number.
    pub fn get(self) -> u32 {
        self.0
    }
}

impl From<u32> for VlanId {
    fn from(value: u32) -> Self {
        VlanId::new(value)
    }
}

impl From<VlanId> for u32 {
    fn from(value: VlanId) -> Self {
        value.0
    }
}

impl fmt::Display for VlanId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Desired changes for a single interface gathered from change input.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InterfaceChange {
    /// Optional description override for the interface.
    pub description: Option<Spanned<String>>,
    /// VLANs to add to the trunk along with their source span.
    pub trunk_add: BTreeMap<VlanId, Span>, // vlan -> span
    /// VLANs to remove from the trunk along with their source span.
    pub trunk_remove: BTreeMap<VlanId, Span>, // vlan -> span
    /// Additional statements to apply under the interface.
    pub other_statements: Vec<SpannedNodeStmt>,
}

#[derive(Clone, Debug, Default)]
pub struct ChangeSpec {
    /// VLAN definitions and optional names from the change input.
    pub vlans: BTreeMap<VlanId, Option<Spanned<String>>>,
    /// Per-interface requested trunk changes.
    pub interface_changes: BTreeMap<BaseIf, InterfaceChange>,
    /// BVI interfaces to create for given VLANs.
    pub bvi_additions: BTreeSet<VlanId>,
    /// Statements to emit under each BVI interface.
    pub bvi_statements: BTreeMap<VlanId, Vec<Spanned<String>>>, // vlan -> statements
    /// Source span of each interface declaration (for error reporting).
    pub interface_spans: HashMap<BaseIf, Span>,
}

impl ChangeSpec {
    /// Return the span for a given interface block, if present.
    pub fn interface_span(&self, name: &BaseIf) -> Option<Span> {
        self.interface_spans.get(name).copied()
    }

    /// Retrieve the VLAN name (if any) from the change spec.
    pub fn vlan_name(&self, vlan: &VlanId) -> Option<&Spanned<String>> {
        self.vlans.get(vlan).and_then(|v| v.as_ref())
    }
}

/// Context about a base interface found in the existing configuration.
#[derive(Clone, Debug, Default)]
pub struct InterfaceContext {
    /// Description configured on the base interface.
    pub description: Option<String>,
    /// Bundle-Ether ID when the interface is part of a bundle.
    pub bundle_id: Option<u32>,
    /// VLANs currently present on subinterfaces for this base interface.
    pub vlans: BTreeSet<VlanId>,
}

/// Snapshot of the base configuration used to validate and plan changes.
#[derive(Clone, Debug, Default)]
pub struct BaseContext {
    /// VLAN descriptions keyed by VLAN ID from existing bridge-domains.
    pub domain_descriptions: HashMap<VlanId, Option<String>>,
    /// Base interface contexts keyed by interface name.
    pub interfaces: HashMap<BaseIf, InterfaceContext>,
}

impl BaseContext {
    /// Build base context from analyzed bridge-domains and parsed nodes.
    pub fn from_analysis(domains: &[BridgeDomain], nodes: &[Node]) -> Self {
        let mut domain_descriptions: HashMap<VlanId, Option<String>> = HashMap::new();
        let mut domain_interfaces: HashMap<VlanId, BTreeSet<String>> = HashMap::new();
        let mut interfaces: HashMap<BaseIf, InterfaceContext> = HashMap::new();

        for domain in domains {
            domain_descriptions.insert(
                VlanId::from(domain.vlan_tag),
                domain.description().map(str::to_string),
            );
            for iface in &domain.interfaces {
                domain_interfaces
                    .entry(VlanId::from(domain.vlan_tag))
                    .or_default()
                    .insert(iface.clone());
            }
        }

        for node in nodes.iter().filter_map(|n| n.as_block()) {
            if let Some(ifname) = node.name.strip_prefix("interface ") {
                if ifname.contains('.') || node.name.ends_with(" l2transport") {
                    continue;
                }

                let interface = interfaces.entry(BaseIf::from(ifname)).or_default();

                if let Some(desc) = node
                    .stmts()
                    .filter_map(|x| x.as_stmt())
                    .find_map(|stmt| stmt.stmt().strip_prefix("description "))
                    .map(|s| s.trim().to_string())
                {
                    interface.description = Some(desc);
                }

                if interface.bundle_id.is_none() {
                    interface.bundle_id =
                        node.stmts().filter_map(|x| x.as_stmt()).find_map(|stmt| {
                            let trimmed = stmt.stmt().trim();
                            trimmed
                                .strip_prefix("bundle id ")?
                                .split_whitespace()
                                .next()?
                                .parse::<u32>()
                                .ok()
                        });
                }
            }
        }

        for (vlan, interfaces_in_domain) in &domain_interfaces {
            for iface in interfaces_in_domain {
                if let Ok((baseif, Some(_))) = split_subinterface_id(iface) {
                    interfaces
                        .entry(BaseIf::from(baseif))
                        .or_default()
                        .vlans
                        .insert(*vlan);
                }
            }
        }

        BaseContext {
            domain_descriptions,
            interfaces,
        }
    }

    /// Return context for a given base interface, if any.
    pub fn interface(&self, name: &BaseIf) -> Option<&InterfaceContext> {
        self.interfaces.get(name)
    }

    /// Get the description of a base interface, if one exists.
    pub fn description_for(&self, name: &BaseIf) -> Option<&str> {
        self.interface(name)
            .and_then(|iface| iface.description.as_deref())
    }

    /// Get the bundle ID for a base interface when it is part of a bundle.
    pub fn bundle_id(&self, name: &BaseIf) -> Option<u32> {
        self.interface(name).and_then(|iface| iface.bundle_id)
    }

    /// Get the VLAN set already present on a base interface.
    pub fn vlans_for(&self, name: &BaseIf) -> Option<&BTreeSet<VlanId>> {
        self.interface(name).map(|iface| &iface.vlans)
    }
}

/// Planned creation of a new subinterface for a VLAN.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InterfaceCreation {
    /// Base interface on which to create the subinterface.
    pub baseif: BaseIf,
    /// VLAN identifier used for the subinterface tag.
    pub vlan: VlanId,
    /// Description to set on the subinterface.
    pub description: String,
}

/// Membership of a base interface in a VLAN.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InterfaceMembership {
    /// Base interface associated with the VLAN.
    pub baseif: BaseIf,
    /// VLAN identifier for membership.
    pub vlan: VlanId,
}

/// Accumulated per-VLAN changes (additions/removals and metadata).
#[derive(Clone, Debug, Default)]
pub struct VlanChange {
    /// VLAN being modified.
    pub vlan: VlanId,
    /// Optional name/description for the VLAN.
    pub description: Option<Spanned<String>>,
    /// Interfaces to remove from the VLAN.
    pub removals: BTreeSet<String>,
    /// Interface memberships to add to the VLAN.
    pub additions: Vec<InterfaceMembership>,
    /// Whether to create a BVI interface for this VLAN.
    pub add_bvi: bool,
}

impl VlanChange {
    /// Initialize a change record for the given VLAN using existing spec metadata.
    pub fn new(vlan: VlanId, change_spec: &ChangeSpec) -> Self {
        let description = change_spec.vlan_name(&vlan).cloned();

        VlanChange {
            vlan,
            description,
            removals: BTreeSet::new(),
            additions: Vec::new(),
            add_bvi: false,
        }
    }

    /// Track removal of an interface from the VLAN.
    pub fn record_removal(&mut self, iface: String) {
        self.removals.insert(iface);
    }

    /// Track addition of an interface to the VLAN.
    pub fn record_addition(&mut self, baseif: BaseIf, vlan: VlanId) {
        self.additions.push(InterfaceMembership { baseif, vlan });
    }
}

/// Finalized plan of configuration changes to render into CLI commands.
#[derive(Clone, Debug, Default)]
pub struct ChangePlan {
    /// Commands to remove outdated subinterfaces.
    pub removal_cmds: Vec<String>,
    /// New subinterfaces to create with associated descriptions.
    pub additions: Vec<InterfaceCreation>,
    /// Per-VLAN change details.
    pub vlan_changes: BTreeMap<VlanId, VlanChange>,
}
