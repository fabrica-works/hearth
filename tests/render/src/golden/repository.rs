//! Decode a golden once for the scripted domain parent.
use super::Golden;
use crate::fixtures::goldens::{JSON_QUEUE, JSON_TURN_GRACE, JSON_TURNS_PER_BYTE};
use hearth_domain_render::{
    revision::{Entry, Mode, Oid},
    vocabulary::Decoded,
};
use hearth_protocol_builder::translate;
use hearth_protocol_config::{self as config, json};
use skein_lib::{
    Env, Queue, Time, Wall,
    stream::{Down, Held},
};
impl Golden {
    #[must_use]
    pub fn repository(&self) -> crate::fixtures::Repository {
        let mut repository =
            crate::fixtures::Repository { entries: Vec::new(), documents: Vec::new(), blobs: Vec::new() };
        for leaf in &self.tree {
            let oid = Oid(leaf.path.clone().into_boxed_slice());
            let mode = match leaf.kind {
                super::Kind::File => Mode::File,
                super::Kind::Executable => Mode::Executable,
                super::Kind::Link => Mode::Link,
            };
            repository.entries.push(Entry {
                path: leaf.path.clone().into_boxed_slice(),
                mode,
                oid: oid.clone(),
                size: u64::try_from(leaf.bytes.len()).expect("file size"),
            });
            if leaf.path.ends_with(b".json") {
                repository.documents.push((oid.clone(), decoded(&leaf.bytes, leaf.path.ends_with(b"/task.json"))));
            }
            repository.blobs.push((oid, leaf.bytes.clone().into_boxed_slice()));
        }
        repository.entries.sort_by(|a, b| a.path.cmp(&b.path));
        repository
    }
}
fn decoded(bytes: &[u8], declaration: bool) -> Decoded {
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: config::limits::SHIPPED };
    let mut decoder = json::Decoder::new(&env.limits);
    let (mut above, mut below) = (Queue::with_capacity(JSON_QUEUE), Queue::with_capacity(JSON_QUEUE));
    let mut held = Held::new(bytes.into());
    json::down(&mut decoder, &env, json::Request::Decode, &mut above, &mut below);
    for _ in 0..bytes.len().saturating_mul(JSON_TURNS_PER_BYTE).saturating_add(JSON_TURN_GRACE) {
        if let Some(event) = above.pop() {
            let value = match event {
                json::Event::Decoded(value) => value,
                json::Event::Failed(error) => return refused(error),
                json::Event::Closed => panic!("active decoder"),
            };
            if declaration {
                return match config::declaration::declaration(value, &env.limits) {
                    Ok(declaration) => match translate::declarations::declaration(
                        declaration,
                        &hearth_protocol_builder::limits::SHIPPED,
                    ) {
                        Ok(declaration) => Decoded::Declaration(declaration),
                        Err(error) => Decoded::Refused { at: error.at, fault: error.fault, also: None },
                    },
                    Err(error) => refused(error),
                };
            }
            return match config::document::document(value) {
                Ok(document) => Decoded::Document(translate::documents::document(document)),
                Err(error) => refused(error),
            };
        }
        if let Some(Down::Demand { read, room: 0 }) = below.pop() {
            json::up(&mut decoder, &env, held.answer(read).expect("source"), &mut above, &mut below);
        }
    }
    panic!("golden decoder settles")
}
fn refused(error: config::Error) -> Decoded {
    let (fault, also) = translate::errors::fault(error.fault, &config::limits::SHIPPED).expect("configuration fault");
    Decoded::Refused { at: translate::documents::position(error.at), fault, also }
}
