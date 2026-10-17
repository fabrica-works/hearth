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
    let source::Layer { tasks: _, debian: _, placement } = layer;
    target::Document { at: position(at), layer: target::Layer { placement: optional_placement(placement) } }
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
