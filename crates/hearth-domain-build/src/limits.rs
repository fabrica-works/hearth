//! The plan's bound (programming-model.md, 6.3).
use crate::plan::{Planned, Step};
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    /// The most layers a plan builds: the recipe's three.
    pub layers: u32,
    /// The most steps a layer runs.
    pub steps: u32,
}
pub const SHIPPED: Limits = Limits { layers: 3, steps: crate::plan::MAX_STEPS };
/// One plan, held while it runs, and the output queue holding its terminal.
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    use skein_lib::{List, Queue};
    let plan = List::<Planned>::worst_case(limits.layers)?
        .checked_add(List::<Step>::worst_case(limits.steps)?.checked_mul(u64::from(limits.layers))?)?;
    plan.checked_add(Queue::<crate::vocabulary::Output>::worst_case(crate::MAX_OUT)?)
}
