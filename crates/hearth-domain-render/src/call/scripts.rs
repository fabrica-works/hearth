//! Validate declared scripts and gather the config lists before rendering.
mod collisions;
use super::{Side, files};
use crate::{
    entries::Expansion,
    errors::Error,
    kinds::{
        files::Plan,
        scripts::{self, Source},
    },
    parts::{
        config::{Preset, Tmpfile},
        meta::Meta,
    },
    schema::Validated,
    vocabulary::Outcome,
};
use alloc::boxed::Box;
use skein_lib::List;
pub(crate) struct Installation {
    pub plans: Box<[Plan]>,
    pub presets: Box<[Preset]>,
    pub tmpfiles: Box<[Tmpfile]>,
    pub scripts: Box<[Source]>,
}
impl Side {
    pub(super) fn installation(
        &mut self,
        meta: &Meta,
        checked: &Validated,
        expansion: &Expansion,
        accounts: &mut crate::kinds::users::Accounts,
        packages: &mut crate::parts::packages::Packages,
        boot: &mut crate::parts::boot::Boot,
    ) -> Option<Installation> {
        let mut capacity = files::fault_capacity(expansion, self.entries.len());
        for task in &checked.tasks {
            let (_, Some(declaration)) = self.documents.declaration(&task.name)? else { return None };
            capacity = capacity.saturating_add(u32::try_from(declaration.scripts.len()).expect("declaration bound"));
        }
        let mut faults: List<Error> = List::with_capacity(capacity);
        let plans = self.plans(meta, checked, expansion, &mut faults);
        let mut sources =
            List::with_capacity(u32::try_from(checked.tasks.len()).expect("task bound").saturating_mul(2));
        for task in &checked.tasks {
            let (_, Some(declaration)) = self.documents.declaration(&task.name)? else { return None };
            for source in scripts::plan(&task.name, declaration, &self.entries, &mut faults) {
                sources.push(source).expect("one customize script per task");
            }
        }
        if !faults.is_empty() {
            let mut faults = faults.into_boxed();
            for fault in &mut faults {
                fault.target = Some(meta.target.clone());
            }
            self.errors.extend(faults);
            self.outcome = self.outcome.worst(Outcome::Failed);
            return None;
        }
        let plans = plans?;
        let (survivors, seeds) = self.agreement(meta, expansion, &plans, accounts)?;
        *accounts = survivors.accounts;
        packages.preseeds = survivors.preseeds;
        packages.seeds = seeds;
        if meta.placement.kind != crate::revision::Runs::Container {
            boot.params = collisions::parameters(survivors.parameters);
        }
        Some(Installation {
            plans,
            presets: survivors.presets,
            tmpfiles: survivors.tmpfiles,
            scripts: sources.into_boxed(),
        })
    }
}
