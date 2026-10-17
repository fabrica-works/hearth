//! Build the packages record after expansion, preserving entry places.
use super::{errors, meta, parts, revision};
use alloc::boxed::Box;
pub(super) fn build(
    root: &crate::evaluate::Value,
    placement: &revision::Placement,
    expansion: &crate::entries::Expansion,
    set: Option<&errors::Place>,
    meta: &meta::Meta,
    errors: &mut errors::Collector,
) -> Option<(parts::packages::Packages, parts::boot::Boot)> {
    let packages = match parts::packages::build(root, placement, expansion, set) {
        Ok(packages) => packages,
        Err(faults) => {
            refuse(faults, meta, errors);
            return None;
        }
    };
    match parts::boot::build(expansion, placement) {
        Ok(boot) => Some((packages, boot)),
        Err(faults) => {
            refuse(faults, meta, errors);
            None
        }
    }
}

fn refuse(faults: Box<[errors::Error]>, meta: &meta::Meta, errors: &mut errors::Collector) {
    for mut fault in faults {
        fault.target = Some(meta.target.clone());
        errors.push(fault);
    }
}
