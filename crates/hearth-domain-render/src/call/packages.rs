//! Build the packages record after expansion, preserving entry places.
use super::{errors, meta, parts, revision};
pub(super) fn build(
    root: &crate::evaluate::Value,
    placement: &revision::Placement,
    expansion: &crate::entries::Expansion,
    set: Option<&errors::Place>,
    meta: &meta::Meta,
    errors: &mut errors::Collector,
) -> Option<parts::packages::Packages> {
    match parts::packages::build(root, placement, expansion, set) {
        Ok(packages) => Some(packages),
        Err(faults) => {
            for mut fault in faults {
                fault.target = Some(meta.target.clone());
                errors.push(fault);
            }
            None
        }
    }
}
