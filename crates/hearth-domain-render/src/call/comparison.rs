//! Compare assembled sides, resolving changed keys before answering (render.md, 5.7).
use super::{Comparison, Side};
use crate::vocabulary::Output;
use skein_lib::Queue;
impl Comparison {
    pub(crate) fn resolve(&mut self, deadline: Option<skein_lib::Time>) {
        self.deadline = deadline;
    }
    fn swap_resolution_side(&mut self, side: &mut Side) {
        let held = side.held_specs();
        let from = self.from.as_mut().expect("from side");
        core::mem::swap(&mut from.made, &mut side.made);
        core::mem::swap(&mut from.finding, &mut side.finding);
        core::mem::swap(&mut from.sizes, &mut side.sizes);
        side.specs_before = held;
        self.from_active = !self.from_active;
        side.errors.side(if self.from_active { crate::diff::Side::From } else { crate::diff::Side::To });
    }
    pub(super) fn resolve_changed(&mut self, side: &mut Side, out: &mut Queue<Output>) -> bool {
        if self.from_active {
            self.swap_resolution_side(side);
        }
        let Some(deadline) = self.deadline else { return false };
        self.swap_resolution_side(side);
        side.inherit_finding(&self.from.as_ref().expect("other side").finding);
        self.swap_resolution_side(side);
        side.inherit_finding(&self.from.as_ref().expect("other side").finding);
        for index in self.resolving..u32::try_from(self.names.len()).expect("bounded names") {
            self.resolving = index;
            let target =
                self.names.get(usize::try_from(index).expect("bounded cursor")).expect("resolution cursor").clone();
            let from = self.from.as_mut().expect("from side");
            let (Some(before), Some(after)) = (key(from.made.get(&target)), key(side.made.get(&target))) else {
                continue;
            };
            if before == after {
                continue;
            }
            self.swap_resolution_side(side);
            if side.resolve_target(&target, deadline, out) {
                return true;
            }
            self.swap_resolution_side(side);
            if side.resolve_target(&target, deadline, out) {
                return true;
            }
        }
        self.resolving = u32::try_from(self.names.len()).expect("bounded names");
        false
    }
}

pub(super) fn key(made: Option<&crate::diff::Made>) -> Option<hearth_lib::encode::Digest> {
    let crate::diff::Made::Spec(spec) = made? else { return None };
    Some(spec.resolution.as_ref()?.key())
}
pub(super) fn closure(record: Option<&super::resolution::Known>) -> Option<&[crate::resolution::Package]> {
    record?.closure.as_deref()
}
