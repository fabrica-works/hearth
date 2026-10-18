//! Validate live consumers once plans expose every hearth-owned destination.
use super::{Side, scripts::Installation};
use crate::{
    entries::Expansion,
    kinds::users::Accounts,
    parts::{self, meta::Meta},
    reserved::{self, Installed, Shape},
    schema::Validated,
    vocabulary::Outcome,
};
use skein_lib::List;
impl Side {
    pub(super) fn reserved_installation(
        &mut self,
        meta: &Meta,
        checked: &Validated,
        expansion: &Expansion,
        accounts: &mut Accounts,
        packages: &mut parts::packages::Packages,
        boot: &mut parts::boot::Boot,
    ) -> Option<(Installation, parts::live::Live)> {
        let installation = self.installation(meta, checked, expansion, accounts, packages, boot)?;
        let mut installed = List::with_capacity(super::files::capacity(&installation.plans));
        for plan in &installation.plans {
            for install in &plan.installs {
                installed
                    .push(Installed { task: &plan.task, path: &install.path, at: install.at, shape: Shape::Directory })
                    .expect("plan bound");
            }
        }
        let faults = reserved::live(expansion, installed.as_slice());
        if !faults.is_empty() {
            for fault in faults {
                self.errors.push(reserved::error(fault, meta));
            }
            self.outcome = self.outcome.worst(Outcome::Failed);
            return None;
        }
        match parts::live::build(expansion) {
            Ok(live) => Some((installation, live)),
            Err(faults) => {
                for mut fault in faults {
                    fault.target = Some(meta.target.clone());
                    self.errors.push(fault);
                }
                self.outcome = self.outcome.worst(Outcome::Failed);
                None
            }
        }
    }
}
