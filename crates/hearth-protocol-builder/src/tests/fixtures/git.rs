use super::{Box, Layer, REQUEST, builder, bytes, io, render};
use crate::Asked;
use io::kernel::Exit;

pub fn listing_layer(limits: crate::limits::Limits) -> Layer {
    let mut layer = Layer::with_form(crate::Form::Text, limits);
    let Asked::Io(io::Request::Spawn { owner, .. }) = layer.resolve() else { panic!("listing spawn") };
    layer.spawned(owner, false, Exit::Code(0));
    layer
}

pub fn tool_failure(
    command: render::errors::Command,
    ended: render::errors::Ended,
    tail: Box<[Box<[u8]>]>,
) -> builder::vocabulary::Event {
    builder::vocabulary::Event::Render(render::vocabulary::Event::Failed {
        request: REQUEST,
        class: render::vocabulary::Class::Operational,
        at: None,
        failure: render::vocabulary::Failure::Tool(render::errors::Tool { command, ended, tail }),
    })
}

pub fn limit_failure(limit: render::errors::Limit, max: u64, path: &[u8]) -> builder::vocabulary::Event {
    builder::vocabulary::Event::Render(render::vocabulary::Event::Failed {
        request: REQUEST,
        class: render::vocabulary::Class::Configuration,
        at: Some(bytes::copy_of(path)),
        failure: render::vocabulary::Failure::TooLarge { limit, max },
    })
}

pub fn binary_hunks() -> builder::vocabulary::Event {
    builder::vocabulary::Event::Render(render::vocabulary::Event::Hunks {
        request: REQUEST,
        hunks: render::diff::Hunks { lines: Box::new([]), left_out: 0, binary: true },
    })
}
