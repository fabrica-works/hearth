//! JSON and template syntax stays in the protocol machines.
use hearth_domain_render::vocabulary::{As, Decoded};
use hearth_protocol_builder::{limits::SHIPPED, translate};
use hearth_protocol_config::{self as config, json};
use skein_lib::{
    Env, Queue, Time, Wall,
    stream::{Down, Held},
};

pub(super) fn blob(bytes: &[u8], kind: As) -> Decoded {
    match kind {
        As::Link => Decoded::Link(bytes.into()),
        As::Template => Decoded::Template(translate::instructions::template(
            hearth_protocol_template::template::compile(bytes, &SHIPPED.template).expect("fixture template"),
        )),
        As::Document | As::Declaration => decode(bytes, kind),
        As::Cached => unreachable!("cached blobs arrive in the store"),
    }
}
fn decode(bytes: &[u8], kind: As) -> Decoded {
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: SHIPPED.config };
    let mut machine = json::Decoder::new(&env.limits);
    let mut above = Queue::with_capacity(1);
    let mut below = Queue::with_capacity(1);
    let mut held = Held::new(bytes.into());
    json::down(&mut machine, &env, json::Request::Decode, &mut above, &mut below);
    for _ in 0..bytes.len().saturating_mul(4).saturating_add(16) {
        if let Some(event) = above.pop() {
            let result = match event {
                json::Event::Decoded(value) => match kind {
                    As::Document => config::document::document(value)
                        .map(|document| Decoded::Document(translate::documents::document(document))),
                    As::Declaration => config::declaration::declaration(value, &env.limits).map(|declaration| {
                        match translate::declarations::declaration(declaration, &SHIPPED) {
                            Ok(declaration) => Decoded::Declaration(declaration),
                            Err(refusal) => Decoded::Refused { at: refusal.at, fault: refusal.fault, also: None },
                        }
                    }),
                    As::Cached | As::Link | As::Template => unreachable!("JSON kinds only"),
                },
                json::Event::Failed(error) => Err(error),
                json::Event::Closed => unreachable!("no close before decoded"),
            };
            return match result {
                Ok(decoded) => decoded,
                Err(config::Error { at, fault }) => {
                    let (fault, also) =
                        translate::errors::fault(fault, &env.limits).expect("fixture within decoding limits");
                    Decoded::Refused { at: translate::documents::position(at), fault, also }
                }
            };
        }
        if let Some(Down::Demand { read, .. }) = below.pop() {
            json::up(&mut machine, &env, held.answer(read).expect("active JSON read"), &mut above, &mut below);
        }
    }
    unreachable!("bounded JSON decode settles")
}
