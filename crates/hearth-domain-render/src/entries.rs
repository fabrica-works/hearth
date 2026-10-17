//! Conditions before concrete entries (format.md, 5.9; render.md, 3.5).
use crate::{
    errors::{Guard, Kind},
    evaluate::{self, Bindings, Budget, Context, Evaluator},
    limits::Limits,
    revision::{Position, declaration::Declaration},
};
use alloc::boxed::Box;
use skein_lib::List;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Fault {
    pub kind: Kind,
    pub at: Position,
    pub also: Option<Position>,
}
/// Every false or unevaluable guard, in name order.
#[must_use]
pub fn guards(declaration: &Declaration, context: &Context<'_>, limits: &Limits) -> Box<[Fault]> {
    let mut faults = List::with_capacity(u32::try_from(declaration.requires.len()).expect("bounded guards"));
    let mut evaluator = Evaluator::new(limits);
    let bindings = Bindings::with_capacity(0);
    for guard in &declaration.requires {
        let fault = match evaluate::expression(
            &mut evaluator,
            &guard.when.expression,
            context,
            &bindings,
            &mut Budget::new(limits),
        ) {
            Ok(value) => {
                if evaluate::truthy(&value) {
                    continue;
                }
                Fault {
                    kind: Kind::Guard(Guard { name: guard.name.text.clone(), message: guard.message.text.clone() }),
                    at: guard.name.at,
                    also: None,
                }
            }
            Err(fault) => Fault { kind: Kind::Evaluation(fault.kind), at: guard.when.at, also: None },
        };
        faults.push(fault).expect("guard count");
    }
    faults.into_boxed()
}
