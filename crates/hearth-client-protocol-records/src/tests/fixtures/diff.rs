use crate::{
    diff::*,
    form::{self, Form},
    limits,
    outcome::Outcome,
    records::{Record, Terminal, Type},
};
use alloc::boxed::Box;
use skein_lib::bytes::copy_of;
pub fn diff_change(target: &[u8], change: TargetChange) -> Record {
    Record::Change(Change { target: copy_of(target), change })
}
pub fn diff_text(record: &Record) -> Result<Box<[u8]>, form::Refusal> {
    form::print(Form::Text, record, Some(Type::Change), &limits::SHIPPED)
}
pub fn diff_terminal(targets: u32, changed: u32, unchanged: u32) -> Record {
    Record::Terminal(Terminal {
        outcome: Outcome::Negative,
        diff: Some(super::tally(targets, changed, unchanged)),
        left_out: 0,
    })
}
pub fn fixture_hunks(value: &mut Change) -> &mut Hunks {
    match &mut value.change {
        TargetChange::Changed { parts, .. } => match super::only_mut(parts) {
            PartChange::Meta(_) | PartChange::Packages(_) | PartChange::Boot(_) | PartChange::Exports(_) => {
                panic!("config change")
            }
            PartChange::Config(config) => match super::only_mut(&mut config.tree) {
                EntryChange::Changed { hunks, .. } => hunks.as_mut().expect("fixture hunks"),
                EntryChange::Added { .. } | EntryChange::Removed { .. } => unreachable!("changed file fixture"),
            },
        },
        TargetChange::Added { .. } | TargetChange::Removed | TargetChange::Same | TargetChange::Failed { .. } => {
            unreachable!("changed fixture")
        }
    }
}
