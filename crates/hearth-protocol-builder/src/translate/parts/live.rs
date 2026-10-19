//! Live records translated exhaustively (domain/render.md, 4.4).
use crate::{records::parts::live as target, render::parts::live as source};
use alloc::boxed::Box;
#[must_use]
pub fn live(live: source::Live) -> target::Live {
    let source::Live { paths } = live;
    let mut covered = skein_lib::List::with_capacity(u32::try_from(paths.len()).expect("path count"));
    for source::Covered { path, consumers } in paths {
        covered.push(target::Covered { path, consumers: consumers_rows(consumers) }).expect("path count");
    }
    target::Live { paths: covered.into_boxed() }
}

fn consumers_rows(values: Box<[source::Consumer]>) -> Box<[target::Consumer]> {
    let mut rows = skein_lib::List::with_capacity(u32::try_from(values.len()).expect("consumer count"));
    for source::Consumer { task, unit, action } in values {
        rows.push(target::Consumer {
            task,
            unit,
            action: match action {
                source::Action::Reload => target::Action::Reload,
                source::Action::Restart => target::Action::Restart,
                source::Action::ReloadOrRestart => target::Action::ReloadOrRestart,
                source::Action::Start => target::Action::Start,
                source::Action::None => target::Action::None,
            },
        })
        .expect("consumer count");
    }
    rows.into_boxed()
}

#[must_use]
pub fn change(value: source::Change) -> target::Change {
    target::Change { paths: changes(value.paths) }
}
fn changes(values: Box<[source::ConsumersKeyed]>) -> Box<[target::ConsumersKeyed]> {
    let mut rows = skein_lib::List::with_capacity(u32::try_from(values.len()).expect("entry count"));
    for value in values {
        rows.push(match value {
            source::ConsumersKeyed::Added { key, value } => {
                target::ConsumersKeyed::Added { key, value: consumers_rows(value) }
            }
            source::ConsumersKeyed::Removed { key, value } => {
                target::ConsumersKeyed::Removed { key, value: consumers_rows(value) }
            }
            source::ConsumersKeyed::Changed { key, before, after } => {
                target::ConsumersKeyed::Changed { key, before: consumers_rows(before), after: consumers_rows(after) }
            }
        })
        .expect("entry count");
    }
    rows.into_boxed()
}
