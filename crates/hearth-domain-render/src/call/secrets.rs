//! Assemble names and places after target expansion (render.md, 3.3).
use crate::{errors, parts, parts::meta, vocabulary::Outcome};
pub(crate) fn build(
    expansion: &crate::entries::Expansion,
    meta: &meta::Meta,
    errors: &mut errors::Collector,
    outcome: &mut Outcome,
) -> Option<parts::secrets::Secrets> {
    match parts::secrets::build(expansion) {
        Ok(secrets) => Some(secrets),
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
