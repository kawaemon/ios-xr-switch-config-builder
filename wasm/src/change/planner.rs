//! Change planning and diff calculation.
//!
//! This module builds the diff plan between the base configuration context and
//! the desired change specification.

use crate::ast::Span;
use crate::change::model::{
    BaseContext, ChangePlan, ChangeSpec, InterfaceChange, InterfaceCreation, InterfaceRemoval,
    VlanChange, VlanId,
};
use crate::change::validator::{
    validate_interface_description, validate_not_bundled_interface, validate_vlan_addition,
    validate_vlan_removals,
};
use std::cmp::Ordering;

/// Builds a concrete change plan from desired input and the existing base context.
pub struct ChangePlanner<'a> {
    /// Parsed change specification from user input.
    change_spec: &'a ChangeSpec,
    /// Context describing the current configuration state.
    base_ctx: &'a BaseContext,
}

impl<'a> ChangePlanner<'a> {
    /// Create a planner over the given change spec and base context.
    pub fn new(change_spec: &'a ChangeSpec, base_ctx: &'a BaseContext) -> Self {
        Self {
            change_spec,
            base_ctx,
        }
    }

    /// Produce a `ChangePlan`, validating inputs along the way.
    pub fn plan(&self) -> Result<ChangePlan, crate::error::Diagnostic> {
        let mut plan = ChangePlan::default();

        for (baseif, change) in &self.change_spec.interface_changes {
            let existing = self.base_ctx.vlans_for(baseif).cloned().unwrap_or_default();

            validate_not_bundled_interface(baseif, change, self.base_ctx, self.change_spec)?;
            validate_vlan_removals(baseif, change, &existing)?;

            let desired = desired_vlans(change, &existing);

            let base_desc = change
                .description
                .as_ref()
                .map(|d| d.value.clone())
                .or_else(|| {
                    self.base_ctx
                        .description_for(baseif)
                        .map(|desc| desc.to_string())
                });

            let has_description = base_desc.is_some();
            validate_interface_description(baseif, has_description, self.change_spec)?;
            let base_desc = base_desc.unwrap();

            for vlan in existing.difference(&desired) {
                let iface = format!("{}.{}", baseif, vlan);
                plan.removal_cmds.push(InterfaceRemoval {
                    baseif: baseif.clone(),
                    command: format!("no interface {} l2transport", iface),
                });

                plan.vlan_changes
                    .entry(*vlan)
                    .or_insert_with(|| VlanChange::new(*vlan, self.change_spec))
                    .record_removal(iface);
            }

            for vlan in desired.difference(&existing) {
                // Validate that the VLAN is defined in vlan database or exists in base config
                if let Some(span) = change
                    .addition_span_for(vlan)
                    .or_else(|| self.change_spec.interface_span(baseif))
                {
                    validate_vlan_addition(*vlan, self.change_spec, self.base_ctx, span)?;
                }

                plan.additions.push(InterfaceCreation {
                    baseif: baseif.clone(),
                    vlan: *vlan,
                    description: build_subinterface_description(
                        *vlan,
                        &base_desc,
                        self.change_spec,
                        self.base_ctx,
                    ),
                });

                plan.vlan_changes
                    .entry(*vlan)
                    .or_insert_with(|| VlanChange::new(*vlan, self.change_spec))
                    .record_addition(baseif.clone(), *vlan);
            }
        }

        for vlan in &self.change_spec.bvi_additions {
            plan.vlan_changes
                .entry(*vlan)
                .or_insert_with(|| VlanChange::new(*vlan, self.change_spec))
                .add_bvi = true;
        }

        Ok(plan)
    }
}

/// Build a subinterface description by combining VLAN and base interface descriptions.
pub fn build_subinterface_description(
    vlan: VlanId,
    base_desc: &str,
    change_spec: &ChangeSpec,
    base_ctx: &BaseContext,
) -> String {
    let vlan_desc = change_spec
        .vlan_name(&vlan)
        .map(|v| v.value.as_str())
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

/// Compute the desired VLAN set for an interface after applying trunk operations.
fn desired_vlans(
    change: &InterfaceChange,
    existing: &std::collections::BTreeSet<VlanId>,
) -> std::collections::BTreeSet<VlanId> {
    #[derive(Clone)]
    enum TrunkOp {
        Clear(Span),
        Set(Span, std::collections::BTreeSet<VlanId>),
        Add(Span, VlanId),
        Remove(Span, VlanId),
    }

    impl TrunkOp {
        fn span(&self) -> Span {
            match self {
                TrunkOp::Clear(span) => *span,
                TrunkOp::Set(span, _) => *span,
                TrunkOp::Add(span, _) => *span,
                TrunkOp::Remove(span, _) => *span,
            }
        }
    }

    let mut ops: Vec<TrunkOp> = Vec::new();

    if let Some(clear_span) = change.trunk_clear {
        ops.push(TrunkOp::Clear(clear_span));
    }

    if let Some(set) = &change.trunk_set {
        ops.push(TrunkOp::Set(set.span, set.value.clone()));
    }

    for (vlan, span) in &change.trunk_add {
        ops.push(TrunkOp::Add(*span, *vlan));
    }

    for (vlan, span) in &change.trunk_remove {
        ops.push(TrunkOp::Remove(*span, *vlan));
    }

    ops.sort_by(|a, b| {
        let sa = a.span();
        let sb = b.span();

        match sa.line.cmp(&sb.line) {
            Ordering::Equal => match sa.col_start.cmp(&sb.col_start) {
                Ordering::Equal => match sa.col_end.cmp(&sb.col_end) {
                    Ordering::Equal => op_order(a).cmp(&op_order(b)),
                    other => other,
                },
                other => other,
            },
            other => other,
        }
    });

    fn op_order(op: &TrunkOp) -> u8 {
        match op {
            TrunkOp::Clear(_) => 0,
            TrunkOp::Set(_, _) => 1,
            TrunkOp::Remove(_, _) => 2,
            TrunkOp::Add(_, _) => 3,
        }
    }

    let mut result = existing.clone();

    for op in ops {
        match op {
            TrunkOp::Clear(_) => result.clear(),
            TrunkOp::Set(_, vlans) => result = vlans,
            TrunkOp::Add(_, vlan) => {
                result.insert(vlan);
            }
            TrunkOp::Remove(_, vlan) => {
                result.remove(&vlan);
            }
        }
    }

    result
}
