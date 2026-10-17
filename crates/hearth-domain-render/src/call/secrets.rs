//! Assemble names and places after target expansion (render.md, 3.3).
use crate::{entries::Expansion, errors, parts, parts::meta, vocabulary::Outcome};
use alloc::boxed::Box;
pub(crate) fn build(
    expansion: &Expansion,
    meta: &meta::Meta,
    errors: &mut errors::Collector,
    outcome: &mut Outcome,
) -> Option<(parts::secrets::Secrets, parts::live::Live)> {
    match built(expansion) {
        Ok(parts) => Some(parts),
        Err(faults) => {
            for mut fault in faults {
                fault.target = Some(meta.target.clone());
                errors.push(fault);
            }
            *outcome = outcome.worst(Outcome::Failed);
            None
        }
    }
}
fn built(expansion: &Expansion) -> Result<(parts::secrets::Secrets, parts::live::Live), Box<[errors::Error]>> {
    Ok((parts::secrets::build(expansion)?, parts::live::build(expansion)?))
}
