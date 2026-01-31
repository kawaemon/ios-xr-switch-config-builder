//! Validation logic for change configurations.
//!
//! This module validates change specifications against base configuration,
//! ensuring that requested changes are valid (e.g., VLANs exist before removal).

use crate::ast::Span;
use crate::change::model::{BaseContext, BaseIf, ChangeSpec, InterfaceChange, VlanId};
use crate::error::{Diagnostic, ErrorKind};
use std::collections::BTreeSet;

/// Ensure VLAN removals reference VLANs that exist on the base interface.
pub fn validate_vlan_removals(
    baseif: &BaseIf,
    change: &InterfaceChange,
    existing: &BTreeSet<VlanId>,
) -> Result<(), Diagnostic> {
    for (vlan, span) in &change.trunk_remove {
        if !existing.contains(vlan) {
            return Err(Diagnostic::with_span(
                ErrorKind::VlanNotPresent {
                    vlan: vlan.get(),
                    interface: baseif.to_string(),
                },
                *span,
            ));
        }
    }
    Ok(())
}

/// Validate that an interface has a description in either base or change input.
pub fn validate_interface_description(
    baseif: &BaseIf,
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

/// Check that a VLAN being added is defined in the change input or base config.
pub fn validate_vlan_addition(
    vlan: VlanId,
    change_spec: &ChangeSpec,
    base_ctx: &BaseContext,
    span: Span,
) -> Result<(), Diagnostic> {
    let vlan_defined_in_change = change_spec.vlans.contains_key(&vlan);
    let vlan_exists_in_base = base_ctx.domain_descriptions.contains_key(&vlan);

    if !vlan_defined_in_change && !vlan_exists_in_base {
        return Err(Diagnostic::with_span(
            ErrorKind::VlanNotDefinedInDatabase { vlan: vlan.get() },
            span,
        ));
    }

    Ok(())
}

/// Ensure VLAN changes are not attempted on member interfaces of a bundle.
pub fn validate_not_bundled_interface(
    baseif: &BaseIf,
    change: &InterfaceChange,
    base_ctx: &BaseContext,
    change_spec: &ChangeSpec,
) -> Result<(), Diagnostic> {
    if let Some(bundle_id) = base_ctx.bundle_id(baseif) {
        // If interface is bundled, it should not have VLAN add/remove operations
        if change.trunk_clear.is_some()
            || change.trunk_set.is_some()
            || !change.trunk_add.is_empty()
            || !change.trunk_remove.is_empty()
        {
            // Get the span from the first VLAN operation
            let span = change
                .trunk_clear
                .or_else(|| change.trunk_set.as_ref().map(|s| s.span))
                .or_else(|| change.trunk_add.values().copied().next())
                .or_else(|| change.trunk_remove.values().copied().next())
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
