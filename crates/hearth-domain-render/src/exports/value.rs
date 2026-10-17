use crate::{
    evaluate::{FaultKind, Operand, Value},
    limits::Limits,
};
use skein_lib::{List, Stack, bytes};
/// Only evaluator-created lists need reconstruction; context values are already typed trees.
pub(super) fn own(operand: &Operand<'_>, limits: &Limits) -> Result<Value, FaultKind> {
    let mut stack = Stack::with_capacity(limits.nesting);
    let mut next = operand;
    for _ in 0..limits.made.saturating_mul(2).saturating_add(u64::from(limits.nesting)).saturating_add(1) {
        let mut value = match next {
            Operand::Undefined => return Err(FaultKind::Undefined),
            Operand::Loop { .. } => return Err(FaultKind::Type),
            Operand::Bool(value) => Value::Bool(*value),
            Operand::Int(value) => Value::Int(*value),
            Operand::String(value) => Value::String(bytes::copy_of(value)),
            Operand::Value(value) => (*value).clone(),
            Operand::List { items, depth: _ } => {
                let mut each = items.iter();
                let made = List::with_capacity(u32::try_from(items.len()).expect("bounded list"));
                if let Some(first) = each.next() {
                    if stack.push((each, made)).is_err() {
                        return Err(FaultKind::Limit);
                    }
                    next = first;
                    continue;
                }
                Value::List(made.into_boxed())
            }
        };
        for _ in 0..=limits.nesting {
            let Some((mut each, mut made)) = stack.pop() else { return Ok(value) };
            made.push(value).expect("list element count");
            if let Some(item) = each.next() {
                stack.push((each, made)).expect("popped frame fits");
                next = item;
                break;
            }
            value = Value::List(made.into_boxed());
        }
    }
    Err(FaultKind::Limit)
}
