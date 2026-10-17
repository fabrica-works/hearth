use super::{composition::at, declaration, expr, key, name, op};
use crate::{
    evaluate::{Missing, Op},
    revision::{
        Position,
        declaration::{Condition, Declaration, Guard, Text},
    },
};
use alloc::boxed::Box;
use skein_lib::bytes::copy_of;
pub fn guard(name: &[u8], at: Position, when: Condition, message: &[u8]) -> Guard {
    Guard { name: Text { at, text: copy_of(name) }, at, when, message: Text { at, text: copy_of(message) } }
}
pub fn requires<const N: usize>(guards: [Guard; N]) -> Declaration {
    let mut declared = declaration(&[], &[]);
    declared.requires = Box::new(guards);
    declared
}
pub fn port_is(value: i64, at: Position) -> Condition {
    Condition {
        at,
        expression: expr(
            2,
            [
                name(0, b"settings", Missing::Error),
                key(9, b"port", Missing::Error),
                op(17, Op::Integer(value)),
                op(14, Op::Equal),
            ],
        ),
    }
}
pub fn optional_guard() -> Declaration {
    requires([guard(
        b"wifi",
        at(3, 5),
        Condition {
            at: at(3, 20),
            expression: expr(1, [name(0, b"settings", Missing::Error), key(9, b"wifi", Missing::Error)]),
        },
        b"wifi needed",
    )])
}
pub fn guarded_tasks() -> (crate::call::Side, crate::schema::Validated) {
    let declared = requires([guard(b"port", at(3, 5), port_is(22, at(3, 20)), b"port needed")]);
    let mut side = super::declaration_call();
    side.documents = super::declarations(&[(b"core", Some(declared.clone())), (b"motd", Some(declared))]);
    let roots = super::Roots::x1_prod();
    let tasks = [b"core".as_slice(), b"motd".as_slice()].map(|name| crate::schema::Checked {
        name: copy_of(name),
        settings: roots.settings.clone(),
        tasks: roots.tasks.clone(),
    });
    (side, crate::schema::Validated { tasks: Box::new(tasks) })
}
pub fn only_mut<T>(items: &mut [T]) -> &mut T {
    let [item] = items else { panic!("one item") };
    item
}
