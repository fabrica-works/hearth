//! Finish agreement and fixed ids before a target starts its templates.
use crate::{
    call::Side,
    collisions::{self, Declared, Survivors},
    entries::Expansion,
    kinds::{cmdline::Parameter, files::Plan, users::Accounts},
    parts::{self, meta::Meta, packages::Seeds},
    vocabulary::Outcome,
};
use alloc::boxed::Box;
use skein_lib::List;
impl Side {
    pub(super) fn agreement(
        &mut self,
        meta: &Meta,
        expansion: &Expansion,
        plans: &[Plan],
        accounts: &Accounts,
    ) -> Option<(Survivors, Seeds)> {
        let declared = Declared { target: &meta.target, expansion, plans, accounts };
        if !collisions::paths(&declared, &self.limits, &mut self.errors) {
            self.outcome = self.outcome.worst(Outcome::Failed);
            return None;
        }
        let Some(survivors) = collisions::declarations(&declared, &self.limits, &mut self.errors) else {
            self.outcome = self.outcome.worst(Outcome::Failed);
            return None;
        };
        if !collisions::ids(&survivors.accounts, &meta.target, &self.limits, &mut self.errors) {
            self.outcome = self.outcome.worst(Outcome::Failed);
            return None;
        }
        match crate::ids::expanded(&survivors.accounts, expansion) {
            Ok(seeds) => Some((survivors, seeds)),
            Err(faults) => {
                self.declaration_faults(meta, faults);
                None
            }
        }
    }
    pub(in crate::call) fn data(&mut self, expansion: &Expansion, meta: &Meta) -> Option<parts::data::Data> {
        match parts::data::build(expansion) {
            Ok(data) => Some(data),
            Err(faults) => {
                self.declaration_faults(meta, faults);
                None
            }
        }
    }
    fn declaration_faults(&mut self, meta: &Meta, faults: Box<[crate::errors::Error]>) {
        for mut fault in faults {
            fault.target = Some(meta.target.clone());
            self.errors.push(fault);
        }
        self.outcome = self.outcome.worst(Outcome::Failed);
    }
}
pub(super) fn parameters(parameters: Box<[Parameter]>) -> Box<[Box<[u8]>]> {
    let mut params = List::with_capacity(u32::try_from(parameters.len()).expect("entry bound"));
    for parameter in parameters {
        params.push(parameter.param).expect("entry bound");
    }
    params.into_boxed()
}
