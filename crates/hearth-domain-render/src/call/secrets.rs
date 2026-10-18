//! Assemble names and places after target expansion (render.md, 3.3).
use crate::{entries::Expansion, errors, parts, parts::meta, vocabulary::Outcome};
use alloc::boxed::Box;
pub(crate) fn build(
    expansion: &Expansion,
    meta: &meta::Meta,
    errors: &mut errors::Collector,
    outcome: &mut Outcome,
) -> Option<(parts::secrets::Secrets, parts::live::Live, parts::software::Software, crate::kinds::users::Accounts)> {
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
fn built(
    expansion: &Expansion,
) -> Result<
    (parts::secrets::Secrets, parts::live::Live, parts::software::Software, crate::kinds::users::Accounts),
    Box<[errors::Error]>,
> {
    let accounts = parts::config::accounts(expansion)?;
    Ok((
        parts::secrets::build(expansion)?,
        parts::live::build(expansion)?,
        parts::software::build(expansion, Box::new([]))?,
        accounts,
    ))
}
