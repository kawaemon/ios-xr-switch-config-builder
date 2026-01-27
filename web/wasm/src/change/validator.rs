//! Validation logic for change configurations.
//!
//! This module validates change specifications against base configuration,
//! ensuring that requested changes are valid (e.g., VLANs exist before removal).

use crate::change::model::{ChangeSpec, InterfaceChange};
use crate::error::{Diagnostic, ErrorKind};
use std::collections::BTreeSet;

pub fn desired_vlans(
    change: &InterfaceChange,
    existing: &BTreeSet<u32>,
) -> Result<BTreeSet<u32>, Diagnostic> {
    let mut result = existing.clone();
    for vlan in change.trunk_add.keys() {
        result.insert(*vlan);
    }
    for vlan in change.trunk_remove.keys() {
        result.remove(vlan);
    }
    Ok(result)
}

pub fn validate_vlan_removals(
    baseif: &str,
    change: &InterfaceChange,
    existing: &BTreeSet<u32>,
) -> Result<(), Diagnostic> {
    for (vlan, span) in &change.trunk_remove {
        if !existing.contains(vlan) {
            return Err(Diagnostic::with_span(
                ErrorKind::VlanNotPresent {
                    vlan: *vlan,
                    interface: baseif.to_string(),
                },
                *span,
            ));
        }
    }
    Ok(())
}

pub fn validate_interface_description(
    baseif: &str,
    has_description: bool,
    change_spec: &ChangeSpec,
) -> Result<(), Diagnostic> {
    if !has_description {
        let mut diag = Diagnostic::new(ErrorKind::MissingDescription {
            interface: baseif.to_string(),
        });
        if let Some(span) = change_spec.interface_span(baseif) {
            diag.span = Some(span);
        }
        return Err(diag);
    }
    Ok(())
}
