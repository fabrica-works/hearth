use crate::{
    Builder, render,
    vocabulary::{Event, Request},
};
use render::revision::{self, Kind, Pair, Position, Value};
use skein_lib::{Queue, Token};
const AT: Position = Position { line: 1, column: 1 };
const PATHS: &[&[u8]] = &[b"envs/prod.json", b"fleet.json", b"hosts/x1/x1-prod.json", b"tasks/hearth/task.json"];
fn value(kind: Kind) -> Value {
    Value { at: AT, kind }
}
fn pair(key: &[u8], kind: Kind) -> Pair {
    Pair { at: AT, key: key.into(), value: value(kind) }
}
fn document(path: &[u8]) -> revision::Document {
    let mut doc = revision::Document {
        at: AT,
        members: None,
        layer: revision::Layer { debian: None, placement: None, tasks: Box::new([]) },
    };
    if path == b"fleet.json" {
        doc.layer.debian = Some(value(Kind::Map(Box::new([
            pair(b"architecture", Kind::String(b"amd64".as_slice().into())),
            pair(
                b"pockets",
                Kind::Map(Box::new([pair(b"security", Kind::Bool(false)), pair(b"updates", Kind::Bool(false))])),
            ),
            pair(
                b"snapshot",
                Kind::Map(Box::new([pair(b"archive", Kind::String(b"20261001T030405Z".as_slice().into()))])),
            ),
            pair(b"suite", Kind::String(b"forky".as_slice().into())),
        ]))));
    }
    if path == b"hosts/x1/x1-prod.json" {
        doc.layer.placement = Some(revision::Placement {
            at: AT,
            kind: revision::Runs::Vm,
            firmware: Some(revision::Firmware::Efi),
            hardware: Some(b"incus".as_slice().into()),
            on: None,
            mutable: false,
        });
    }
    doc
}
pub fn resolving_builder(env: &skein_lib::Env<crate::Limits>) -> (Builder, Queue<Request>) {
    let mut builder = super::opened_builder();
    builder.reply = Some(skein_lib::ReplyTo::new(Token::new(1)));
    let mut out = Queue::with_capacity(2);
    crate::step(
        &mut builder,
        env,
        Event::Render(render::vocabulary::Event::Render {
            resolve: true,
            repository: b"/repo".as_slice().into(),
            revision: b"HEAD".as_slice().into(),
            targets: Box::new([]),
        }),
        &mut out,
    );
    drop(out.pop());
    let mut entries = skein_lib::List::with_capacity(4);
    for path in PATHS {
        entries
            .push(revision::Entry {
                path: (*path).into(),
                oid: revision::Oid((*path).into()),
                mode: revision::Mode::File,
                size: 0,
            })
            .expect("fixture entries");
    }
    crate::step(
        &mut builder,
        env,
        Event::Render(render::vocabulary::Event::Listed {
            request: Token::new(1),
            commit: revision::Oid(b"commit".as_slice().into()),
            entries: entries.into_boxed(),
        }),
        &mut out,
    );
    drop(out.pop());
    for path in PATHS {
        let read = if *path == b"tasks/hearth/task.json" {
            render::vocabulary::Decoded::Declaration(revision::declaration::Declaration {
                at: AT,
                types: Box::new([]),
                scripts: Box::new([]),
                settings: Box::new([]),
                reads: Box::new([]),
                requires: Box::new([]),
                exports: Box::new([]),
                entries: Box::new([]),
            })
        } else {
            render::vocabulary::Decoded::Document(document(path))
        };
        crate::step(
            &mut builder,
            env,
            Event::Render(render::vocabulary::Event::Read {
                request: Token::new(2),
                oid: revision::Oid((*path).into()),
                read,
            }),
            &mut out,
        );
    }
    for _ in 0_u32..64 {
        if !out.is_empty() {
            break;
        }
        crate::resume(&mut builder, env, &mut out);
    }
    (builder, out)
}
pub fn record_builder() -> (Builder, Queue<Request>) {
    let env = super::env();
    let (mut builder, mut out) = resolving_builder(&env);
    drop(out.pop());
    drop(out.pop());
    crate::step(&mut builder, &env, resolved(), &mut out);
    drop(out.pop());
    (builder, out)
}
pub fn finish(builder: &mut Builder, out: &mut Queue<Request>) -> Box<[Request]> {
    let mut requests = skein_lib::List::with_capacity(64);
    for _ in 0_u32..64 {
        if let Some(request) = out.pop() {
            requests.push(request).expect("fixture requests");
        }
        if builder.is_empty() {
            break;
        }
        crate::resume(builder, &super::env(), out);
    }
    requests.into_boxed()
}
pub const NO_RESOLUTION_ROOM: crate::Limits =
    crate::Limits { store: crate::store::limits::Limits { resolutions: 0, ..super::LIMITS.store }, ..super::LIMITS };

pub fn error_count(requests: &[Request]) -> u32 {
    let mut count = 0_u32;
    for request in requests {
        if let Request::Answer(crate::vocabulary::Answer::Error(_)) = request {
            count = count.saturating_add(1);
        }
    }
    count
}

#[must_use]
pub fn resolutions() -> [crate::store::vocabulary::Resolution; 2] {
    let first = super::store_resolution();
    let mut second = first.clone();
    second.made.at = skein_lib::Wall::from_nanos(77);
    second.made.apt = b"3.3.3".as_slice().into();
    second.sources = Box::new([crate::store::vocabulary::Line {
        uri: b"http://one/".as_slice().into(),
        suite: b"forky".as_slice().into(),
        components: Box::new([b"main".as_slice().into()]),
        indices: Box::new([crate::store::vocabulary::Index {
            path: b"InRelease".as_slice().into(),
            size: 7,
            sha256: super::FIND_KEY,
        }]),
    }]);
    second.closure = Box::new([crate::store::vocabulary::Closed {
        name: b"apt".as_slice().into(),
        architecture: b"amd64".as_slice().into(),
        version: b"3.3.3".as_slice().into(),
        source: 0,
        filename: b"pool/apt.deb".as_slice().into(),
        size: 7,
        sha256: super::FIND_KEY,
    }]);
    [first, second]
}

pub fn resolved() -> Event {
    Event::Render(render::vocabulary::Event::Resolved {
        request: Token::new(6),
        lines: Box::new([]),
        tools: render::resolution::Tools { apt: Box::new([]), mmdebstrap: Box::new([]) },
        costs: render::resolution::Costs { took: skein_lib::Duration::from_millis(7), fetched: 0 },
    })
}
pub fn index() -> Event {
    Event::Index {
        request: Token::new(6),
        arrival: Token::new(21),
        sha256: FIND_INDEX,
        size: 3,
        took: skein_lib::Duration::ZERO,
    }
}
pub const FIND_INDEX: render::vocabulary::Digest = render::vocabulary::Digest([3; 32]);
pub fn resolving_request() -> (Builder, Queue<Request>, render::vocabulary::Digest) {
    let (builder, mut out) = resolving_builder(&super::env());
    drop(out.pop());
    let Some(Request::Render(render::vocabulary::Request::Resolve { key, .. })) = out.pop() else {
        panic!("resolve request")
    };
    (builder, out, key)
}
pub fn needed_count(requests: &[Request]) -> usize {
    let mut count = 0_usize;
    for request in requests {
        if let Request::Answer(crate::vocabulary::Answer::Spec(spec)) = request
            && let Some(render::keys::Resolution::Needed { .. }) = spec.resolution.as_deref()
        {
            count = count.saturating_add(1);
        }
    }
    count
}

pub fn finish_resolution(builder: &mut Builder, out: &mut Queue<Request>) -> Box<[Request]> {
    let mut requests = skein_lib::List::with_capacity(64);
    if let Some(request) = out.pop() {
        requests.push(request).expect("store fault");
    }
    crate::step(builder, &super::env(), resolved(), out);
    for request in finish(builder, out) {
        requests.push(request).expect("resolution requests");
    }
    requests.into_boxed()
}
pub fn resolved_render() -> render::vocabulary::Event {
    let Event::Render(event) = resolved() else { unreachable!("render input") };
    event
}
pub fn resolving_progress() -> (Builder, Queue<Request>) {
    let (_, mut requests) = resolving_builder(&super::env());
    drop(requests.pop());
    let Some(Request::Render(mut resolve)) = requests.pop() else { panic!("resolve input") };
    let render::vocabulary::Request::Resolve { key, .. } = &mut resolve else { panic!("resolve") };
    *key = super::FIND_KEY;
    let mut builder = super::opened_builder();
    builder.resolving = Some(resolve);
    (builder, requests)
}
