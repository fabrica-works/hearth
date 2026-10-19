//! Relative forward jumps stay valid when a conditional rotates its branches (domain/render.md, 8.3).
use crate::{Op, Operation};
use skein_lib::List;
pub(super) fn patch(out: &mut List<Operation>, jump: u32) {
    let over = out.len().checked_sub(jump).expect("emitted jump").checked_sub(1).expect("jump occupies one slot");
    match &mut out.get_mut(jump).expect("emitted jump").op {
        Op::JumpIfFalseKeeping { over: value } | Op::JumpIfTrueKeeping { over: value } | Op::Jump { over: value } => {
            *value = over;
        }
        Op::JumpIfFalse { .. }
        | Op::Name { .. }
        | Op::Key { .. }
        | Op::Item { .. }
        | Op::String(_)
        | Op::Integer(_)
        | Op::Boolean(_)
        | Op::List { .. }
        | Op::Filter(_)
        | Op::Not
        | Op::Equal
        | Op::NotEqual
        | Op::Less
        | Op::LessOrEqual
        | Op::Greater
        | Op::GreaterOrEqual
        | Op::In
        | Op::NotIn
        | Op::Concat
        | Op::Add
        | Op::Subtract
        | Op::Defined
        | Op::NotDefined => unreachable!("patchable jump"),
    }
}
pub(super) fn rotate(out: &mut List<Operation>, start: u32, middle: u32) {
    let end = out.len();
    reverse(out, start, middle);
    reverse(out, middle, end);
    reverse(out, start, end);
}
fn reverse(out: &mut List<Operation>, start: u32, end: u32) {
    let length = end.checked_sub(start).expect("ordered rotation range");
    for offset in 0..length.checked_div(2).expect("nonzero divisor") {
        let left = start.saturating_add(offset);
        let right = end.saturating_sub(offset).saturating_sub(1);
        let value =
            core::mem::replace(out.get_mut(left).expect("rotation slot"), Operation { at: 0, op: Op::Boolean(false) });
        let other = core::mem::replace(out.get_mut(right).expect("rotation slot"), value);
        *out.get_mut(left).expect("rotation slot") = other;
    }
}
