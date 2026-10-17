//! Validated values and expression operands (format.md, 6.2, 7.4, 7.6).
use super::{Budget, ELEMENT, FaultKind};
use alloc::boxed::Box;
use skein_lib::{List, Stack, bytes};
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Value {
    Bool(bool),
    Int(i64),
    String(Box<[u8]>),
    List(Box<[Value]>),
    Map(Box<[Entry]>),
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Entry {
    pub key: Box<[u8]>,
    pub value: Option<Value>,
}
#[derive(Clone, Debug)]
pub enum Operand<'c> {
    Undefined,
    Loop { index0: u32, length: u32 },
    Value(&'c Value),
    Bool(bool),
    Int(i64),
    String(Box<[u8]>),
    List { depth: u32, items: Box<[Operand<'c>]> },
}
impl Operand<'_> {
    #[must_use]
    pub const fn undefined(&self) -> bool {
        match self {
            Self::Undefined => true,
            Self::Loop { .. } | Self::Value(_) | Self::Bool(_) | Self::Int(_) | Self::String(_) | Self::List { .. } => {
                false
            }
        }
    }
}
// Iterators keep this walk proportional to nesting, even for wide lists.
pub(super) fn copy<'c>(value: &Operand<'c>, budget: &mut Budget, nesting: u32) -> Result<Operand<'c>, FaultKind> {
    let mut frames = Stack::with_capacity(nesting);
    let mut next = value;
    for _ in 0..=budget.left.saturating_mul(2).saturating_add(u64::from(nesting)).saturating_add(1) {
        let copied = match next {
            Operand::Undefined => Operand::Undefined,
            Operand::Loop { index0, length } => Operand::Loop { index0: *index0, length: *length },
            Operand::Value(v) => Operand::Value(v),
            Operand::Bool(v) => Operand::Bool(*v),
            Operand::Int(v) => Operand::Int(*v),
            Operand::String(v) => {
                budget.charge(u64::try_from(v.len()).expect("bytes fit"))?;
                Operand::String(bytes::copy_of(v))
            }
            Operand::List { depth, items } => {
                let length = u32::try_from(items.len()).expect("bounded list");
                budget.charge(u64::from(length).saturating_mul(ELEMENT))?;
                let mut source = items.iter();
                let made = List::with_capacity(length);
                if let Some(first) = source.next() {
                    if frames.push((source, made, *depth)).is_err() {
                        return Err(FaultKind::Limit);
                    }
                    next = first;
                    continue;
                }
                Operand::List { depth: *depth, items: made.into_boxed() }
            }
        };
        let mut complete = copied;
        for _ in 0..=nesting {
            let Some((mut source, mut made, depth)) = frames.pop() else {
                return Ok(complete);
            };
            made.push(complete).expect("one copy per source item");
            match source.next() {
                Some(item) => {
                    frames.push((source, made, depth)).expect("just popped");
                    next = item;
                    break;
                }
                None => {
                    complete = Operand::List { depth, items: made.into_boxed() };
                }
            }
        }
    }
    Err(FaultKind::Limit)
}
