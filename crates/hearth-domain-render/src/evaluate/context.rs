//! The five roots and scoped names (format.md, 7.7).
use super::{
    Budget, FaultKind,
    value::{Entry, Operand, Value},
};
use crate::{
    parts::meta::Meta,
    revision::{Firmware, Runs},
};
use skein_lib::{List, bytes};
#[derive(Clone, Copy, Debug)]
pub struct Context<'c> {
    pub target: &'c Value,
    pub debian: &'c Value,
    pub settings: &'c Value,
    pub tasks: &'c Value,
    pub exports: &'c Value,
}
#[derive(Clone, Debug)]
pub struct Binding<'c> {
    pub name: &'c [u8],
    pub value: Operand<'c>,
}
#[derive(Debug)]
pub struct Bindings<'c> {
    entries: List<Binding<'c>>,
}
impl<'c> Bindings<'c> {
    #[must_use]
    pub fn with_capacity(capacity: u32) -> Self {
        Self { entries: List::with_capacity(capacity) }
    }
    pub fn push(&mut self, binding: Binding<'c>) -> Result<(), Binding<'c>> {
        self.entries.push(binding)
    }
    pub fn truncate(&mut self, length: u32) {
        self.entries.truncate(length);
    }
    #[must_use]
    pub fn len(&self) -> u32 {
        self.entries.len()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub(super) fn find(&self, name: &[u8], budget: &mut Budget, nesting: u32) -> Result<Operand<'c>, FaultKind> {
        for binding in self.entries.iter().rev() {
            if binding.name == name {
                return super::value::copy(&binding.value, budget, nesting);
            }
        }
        Err(FaultKind::Unknown)
    }
}
fn entry(key: &[u8], value: Option<Value>) -> Entry {
    Entry { key: bytes::copy_of(key), value }
}
fn string(value: &[u8]) -> Value {
    Value::String(bytes::copy_of(value))
}
#[must_use]
pub fn target(meta: &Meta) -> Value {
    let mut placement = List::with_capacity(4);
    if let Some(firmware) = meta.placement.firmware {
        placement
            .push(entry(
                b"firmware",
                Some(string(match firmware {
                    Firmware::Efi => b"efi",
                    Firmware::Bios => b"bios",
                })),
            ))
            .expect("four fields");
    }
    if let Some(hardware) = &meta.placement.hardware {
        placement.push(entry(b"hardware", Some(string(hardware)))).expect("four fields");
    }
    placement
        .push(entry(
            b"kind",
            Some(string(match meta.placement.kind {
                Runs::Metal => b"metal",
                Runs::Vm => b"vm",
                Runs::Container => b"container",
            })),
        ))
        .expect("four fields");
    if let Some(on) = &meta.placement.on {
        placement.push(entry(b"on", Some(string(on)))).expect("four fields");
    }
    let mut member = None;
    if let Some(v) = &meta.member {
        member = Some(string(v));
    }
    Value::Map(Box::new([
        entry(b"base", Some(string(&meta.base))),
        entry(b"containers", Some(Value::Map(Box::new([])))),
        entry(b"environment", Some(string(&meta.environment))),
        entry(b"member", member),
        entry(b"name", Some(string(&meta.target))),
        entry(b"placement", Some(Value::Map(placement.into_boxed()))),
    ]))
}
use alloc::boxed::Box;

/// One task's owned context roots (format.md, 7.7).
#[derive(Debug)]
pub struct Roots {
    pub target: Value,
    pub debian: Value,
    pub settings: Value,
    pub tasks: Value,
    pub exports: Value,
}
impl Roots {
    #[must_use]
    pub fn context(&self) -> Context<'_> {
        Context {
            target: &self.target,
            debian: &self.debian,
            settings: &self.settings,
            tasks: &self.tasks,
            exports: &self.exports,
        }
    }
}
