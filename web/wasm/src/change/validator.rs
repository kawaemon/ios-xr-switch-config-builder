//! Validation logic for change configurations.
//!
//! This module validates change specifications against base configuration,
//! ensuring that requested changes are valid (e.g., VLANs exist before removal).

use crate::ast::Span;
use crate::change::model::{BaseContext, ChangeSpec, InterfaceChange};
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

pub fn validate_vlan_addition(
    vlan: u32,
    change_spec: &ChangeSpec,
    base_ctx: &BaseContext,
    span: Span,
) -> Result<(), Diagnostic> {
    let vlan_defined_in_change = change_spec.vlans.contains_key(&vlan);
    let vlan_exists_in_base = base_ctx.domain_descriptions.contains_key(&vlan);

    if !vlan_defined_in_change && !vlan_exists_in_base {
        return Err(Diagnostic::with_span(
            ErrorKind::VlanNotDefinedInDatabase { vlan },
            span,
        ));
    }

    Ok(())
}

pub fn validate_not_bundled_interface(
    baseif: &str,
    change: &InterfaceChange,
    base_ctx: &BaseContext,
    change_spec: &ChangeSpec,
) -> Result<(), Diagnostic> {
    if let Some(&bundle_id) = base_ctx.bundled_interfaces.get(baseif) {
        // If interface is bundled, it should not have VLAN add/remove operations
        if !change.trunk_add.is_empty() || !change.trunk_remove.is_empty() {
            // Get the span from the first VLAN operation
            let span = change
                .trunk_add
                .values()
                .next()
                .or_else(|| change.trunk_remove.values().next())
                .copied()
                .or_else(|| change_spec.interface_span(baseif))
                .unwrap_or_else(|| Span::line_only(1));

            return Err(Diagnostic::with_span(
                ErrorKind::BundledInterfaceCannotConfigureVlans {
                    interface: baseif.to_string(),
                    bundle_id,
                },
                span,
            ));
        }
    }
    Ok(())
}
