//! Change planning and diff calculation.
//!
//! This module builds the diff plan between the base configuration context and
//! the desired change specification.

use crate::change::model::{
    BaseContext, ChangePlan, ChangeSpec, InterfaceCreation, VlanChange,
};
use crate::change::validator::{
    desired_vlans, validate_interface_description, validate_not_bundled_interface,
    validate_vlan_addition, validate_vlan_removals,
};

pub struct ChangePlanner<'a> {
    change_spec: &'a ChangeSpec,
    base_ctx: &'a BaseContext,
}

impl<'a> ChangePlanner<'a> {
    pub fn new(change_spec: &'a ChangeSpec, base_ctx: &'a BaseContext) -> Self {
        Self {
            change_spec,
            base_ctx,
        }
    }

    pub fn plan(&self) -> Result<ChangePlan, crate::error::Diagnostic> {
        let mut plan = ChangePlan::default();

        for (baseif, change) in &self.change_spec.interface_changes {
            let existing = self
                .base_ctx
                .existing_membership
                .get(baseif)
                .cloned()
                .unwrap_or_default();

            validate_not_bundled_interface(baseif, change, self.base_ctx, self.change_spec)?;
            validate_vlan_removals(baseif, change, &existing)?;

            let desired = desired_vlans(change, &existing)?;

            let base_desc = change
                .description
                .as_ref()
                .or_else(|| self.base_ctx.base_descriptions.get(baseif))
                .cloned();

            let has_description = base_desc.is_some();
            validate_interface_description(baseif, has_description, self.change_spec)?;
            let base_desc = base_desc.unwrap();

            for vlan in existing.difference(&desired) {
                let iface = format!("{}.{}", baseif, vlan);
                plan.removal_cmds
                    .push(format!("no interface {} l2transport", iface));

                plan.vlan_changes
                    .entry(*vlan)
                    .or_insert_with(|| VlanChange::new(*vlan, self.change_spec, self.base_ctx))
                    .record_removal(iface);
            }

            for vlan in desired.difference(&existing) {
                // Validate that the VLAN is defined in vlan database or exists in base config
                if let Some(&span) = change.trunk_add.get(vlan) {
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
                    .or_insert_with(|| VlanChange::new(*vlan, self.change_spec, self.base_ctx))
                    .record_addition(baseif.clone(), *vlan);
            }
        }

        for vlan in &self.change_spec.bvi_additions {
            plan.vlan_changes
                .entry(*vlan)
                .or_insert_with(|| VlanChange::new(*vlan, self.change_spec, self.base_ctx))
                .add_bvi = true;
        }

        Ok(plan)
    }
}

pub fn build_subinterface_description(
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
