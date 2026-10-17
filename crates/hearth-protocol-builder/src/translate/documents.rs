//! The thin story takes placement and names later document fields (render.md, 3.1).
use crate::{
    config::{self, document as source},
    render::revision as target,
};
#[must_use]
pub fn position(at: config::Position) -> target::Position {
    let config::Position { offset: _, line, column } = at;
    target::Position { line, column }
}
#[must_use]
pub fn document(document: source::Document) -> target::Document {
    let source::Document { at, layer, members: _ } = document;
    let source::Layer { tasks, debian: _, placement } = layer;
    let mut translated = skein_lib::List::with_capacity(u32::try_from(tasks.len()).expect("bounded tasks"));
    for source::Task { name, at, value } in tasks {
        translated.push(target::Task { name, at: position(at), value: self::value(value) }).expect("task capacity");
    }
    target::Document {
        at: position(at),
        layer: target::Layer { placement: optional_placement(placement), tasks: translated.into_boxed() },
    }
}
#[must_use]
pub fn placement_to_render(placement: source::Placement) -> target::Placement {
    let source::Placement { at, kind, firmware, hardware, on, mutable } = placement;
    target::Placement {
        at: position(at),
        kind: runs(kind),
        firmware: optional_firmware(firmware),
        hardware,
        on: host(on),
        mutable,
    }
}
fn optional_placement(placement: Option<source::Placement>) -> Option<target::Placement> {
    Some(placement_to_render(placement?))
}
fn optional_firmware(firmware: Option<source::Firmware>) -> Option<target::Firmware> {
    Some(boot(firmware?))
}
fn host(on: Option<source::On>) -> Option<target::On> {
    let source::On { target, at } = on?;
    Some(target::On { target, at: position(at) })
}
#[must_use]
pub fn runs(kind: source::Runs) -> target::Runs {
    match kind {
        source::Runs::Metal => target::Runs::Metal,
        source::Runs::Vm => target::Runs::Vm,
        source::Runs::Container => target::Runs::Container,
    }
}
#[must_use]
pub fn boot(firmware: source::Firmware) -> target::Firmware {
    match firmware {
        source::Firmware::Efi => target::Firmware::Efi,
        source::Firmware::Bios => target::Firmware::Bios,
    }
}

/// Translate each node on a bounded stack, retaining every position.
#[must_use]
pub fn value(value: config::json::Value) -> target::Value {
    let mut stack = skein_lib::Stack::with_capacity(config::limits::SHIPPED.depth.saturating_add(1));
    let mut pending = Some(value);
    let mut finished = None;
    for _ in 0..u64::from(config::limits::SHIPPED.document).saturating_mul(3).saturating_add(3) {
        if let Some(value) = pending.take() {
            let config::json::Value { at, kind } = value;
            let at = position(at);
            match kind {
                config::json::Kind::Null => finished = Some(target::Value { at, kind: target::Kind::Null }),
                config::json::Kind::Bool(v) => finished = Some(target::Value { at, kind: target::Kind::Bool(v) }),
                config::json::Kind::Int(v) => finished = Some(target::Value { at, kind: target::Kind::Int(v) }),
                config::json::Kind::String(v) => finished = Some(target::Value { at, kind: target::Kind::String(v) }),
                config::json::Kind::List(values) => {
                    let out = skein_lib::List::with_capacity(u32::try_from(values.len()).expect("bounded values"));
                    stack.push(Frame::List { at, values, next: 0, out }).expect("decoded depth");
                }
                config::json::Kind::Map(pairs) => {
                    let out = skein_lib::List::with_capacity(u32::try_from(pairs.len()).expect("bounded pairs"));
                    stack.push(Frame::Map { at, pairs, next: 0, key: None, out }).expect("decoded depth");
                }
            }
        }
        let Some(mut frame) = stack.pop() else {
            return finished.expect("translated root");
        };
        match &mut frame {
            Frame::List { at: _, values, next, out } => {
                if let Some(value) = finished.take() {
                    out.push(value).expect("value count");
                }
                if let Some(value) = values.get_mut(*next) {
                    *next = next.saturating_add(1);
                    pending = Some(take(value));
                } else {
                    let Frame::List { at, values: _, next: _, out } = frame else { unreachable!("list") };
                    finished = Some(target::Value { at, kind: target::Kind::List(out.into_boxed()) });
                    continue;
                }
            }
            Frame::Map { at: _, pairs, next, key, out } => {
                if let Some((name, at)) = key.take() {
                    out.push(target::Pair { key: name, at, value: finished.take().expect("translated child") })
                        .expect("pair count");
                }
                if let Some(pair) = pairs.get_mut(*next) {
                    *next = next.saturating_add(1);
                    *key = Some((pair.key.clone(), position(pair.at)));
                    pending = Some(take(&mut pair.value));
                } else {
                    let Frame::Map { at, pairs: _, next: _, key: _, out } = frame else { unreachable!("map") };
                    finished = Some(target::Value { at, kind: target::Kind::Map(out.into_boxed()) });
                    continue;
                }
            }
        }
        stack.push(frame).expect("popped frame fits");
    }
    unreachable!("bounded decoded value")
}
use alloc::boxed::Box;
#[derive(Debug)]
enum Frame {
    List {
        at: target::Position,
        values: Box<[config::json::Value]>,
        next: usize,
        out: skein_lib::List<target::Value>,
    },
    Map {
        at: target::Position,
        pairs: Box<[config::json::Entry]>,
        next: usize,
        key: Option<(Box<[u8]>, target::Position)>,
        out: skein_lib::List<target::Pair>,
    },
}
fn take(value: &mut config::json::Value) -> config::json::Value {
    core::mem::replace(value, config::json::Value { at: value.at, kind: config::json::Kind::Null })
}
