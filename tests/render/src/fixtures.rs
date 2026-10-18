//! Scripted repositories: the domain receives typed documents.
use hearth_domain_render::{
    parts::meta::{Meta, Placement as MetaPlacement},
    revision::{Document, Entry, Firmware, Layer, Mode, Oid, Placement, Position, Runs},
    vocabulary::Decoded,
};
use hearth_lib::encode::Digest;
#[derive(Clone, Debug)]
pub struct Repository {
    pub entries: Vec<Entry>,
    pub blobs: Vec<(Oid, Box<[u8]>)>,
    pub documents: Vec<(Oid, Decoded)>,
}
#[must_use]
pub fn oid(byte: u8) -> Oid {
    Oid(vec![byte; 40].into_boxed_slice())
}
#[must_use]
pub fn document(placement: Option<Placement>) -> Decoded {
    Decoded::Document(Document {
        members: None,
        at: Position { line: 1, column: 1 },
        layer: Layer { debian: Some(written_debian()), placement, tasks: Box::new([]) },
    })
}
#[must_use]
pub fn placement() -> Placement {
    Placement {
        at: Position { line: 1, column: 15 },
        kind: Runs::Metal,
        firmware: Some(Firmware::Efi),
        hardware: Some(Box::from(b"7th-gen".as_slice())),
        on: None,
        mutable: false,
    }
}
#[must_use]
pub fn x1_prod() -> Repository {
    Repository {
        blobs: Vec::new(),
        entries: vec![
            entry(b"envs/prod.json", b'1'),
            entry(b"hosts/x1/x1.json", b'2'),
            entry(b"hosts/x1/x1-prod.json", b'3'),
            entry(b"tasks/hearth/task.json", b'h'),
        ],
        documents: vec![
            (oid(b'1'), document(None)),
            (oid(b'2'), document(None)),
            (oid(b'3'), document(Some(placement()))),
            (oid(b'h'), Decoded::Declaration(crate::declarations::decode(b"{}"))),
        ],
    }
}
#[must_use]
pub fn entry(path: &[u8], byte: u8) -> Entry {
    Entry { path: Box::from(path), mode: Mode::File, oid: oid(byte), size: 2 }
}
#[must_use]
pub fn x1_prod_meta() -> Meta {
    Meta {
        target: Box::from(b"x1-prod".as_slice()),
        base: Box::from(b"x1".as_slice()),
        member: None,
        environment: Box::from(b"prod".as_slice()),
        placement: MetaPlacement {
            kind: Runs::Metal,
            firmware: Some(Firmware::Efi),
            hardware: Some(Box::from(b"7th-gen".as_slice())),
            on: None,
        },
    }
}
#[must_use]
pub fn digest(text: &str) -> Digest {
    Digest(core::array::from_fn(|i| u8::from_str_radix(&text[i * 2..i * 2 + 2], 16).expect("digest hex")))
}

use hearth_domain_render::{
    evaluate::FaultKind,
    limits::{Limits, SHIPPED},
};
use skein_lib::Rng;
use std::collections::BTreeSet;
#[must_use]
pub fn limits(rng: &mut Rng) -> Limits {
    if rng.below(4) != 0 {
        SHIPPED
    } else {
        Limits {
            made: rng.between(16, 4096),
            nesting: u32::try_from(rng.between(1, 8)).expect("small"),
            expansion: u32::try_from(rng.between(1, 16)).expect("small"),
            expanded: rng.between(16, 4096),
            ..SHIPPED
        }
    }
}
#[must_use]
pub const fn kind(kind: FaultKind) -> &'static str {
    match kind {
        FaultKind::Unknown => "unknown",
        FaultKind::NoKey => "key",
        FaultKind::NoIndex => "index",
        FaultKind::Undefined => "undefined",
        FaultKind::Type => "type",
        FaultKind::Overflow => "overflow",
        FaultKind::Empty => "empty",
        FaultKind::Path => "path",
        FaultKind::Output => "output",
        FaultKind::Limit => "limit",
    }
}
pub fn assert_fell(fell: &BTreeSet<&str>, kinds: &[&str]) {
    for kind in kinds {
        assert!(fell.contains(kind), "missing {kind}");
    }
}

#[must_use]
pub fn with_task() -> Repository {
    let mut repository = x1_prod();
    repository.documents.iter_mut().find(|(id, _)| *id == oid(b'2')).expect("base document").1 =
        Decoded::Document(Document {
            members: None,
            at: Position { line: 1, column: 1 },
            layer: Layer {
                debian: None,
                placement: None,
                tasks: Box::new([hearth_domain_render::revision::Task {
                    name: b"motd".as_slice().into(),
                    at: Position { line: 1, column: 2 },
                    value: hearth_domain_render::revision::Value {
                        at: Position { line: 1, column: 9 },
                        kind: hearth_domain_render::revision::Kind::Map(Box::new([])),
                    },
                }]),
            },
        });
    repository.entries.push(entry(b"tasks/motd/task.json", b'4'));
    repository.documents.push((
        oid(b'4'),
        Decoded::Declaration(hearth_domain_render::revision::declaration::Declaration {
            at: Position { line: 1, column: 1 },
            types: Box::new([]),
            settings: Box::new([]),
            requires: Box::new([]),
            scripts: Box::new([]),
            exports: Box::new([]),
            entries: Box::new([]),
            reads: Box::new([]),
        }),
    ));
    repository
}

pub fn only<T>(items: &[T]) -> &T {
    let [item] = items else { panic!("one item") };
    item
}
#[must_use]
pub fn two_targets() -> Repository {
    let mut repository = x1_prod();
    repository.entries.push(entry(b"hosts/a-b/a-b-prod.json", b'4'));
    repository.documents.push((oid(b'4'), document(Some(placement()))));
    sort_entries(&mut repository);
    repository
}
pub fn set_document(repository: &mut Repository, byte: u8, document: Decoded) {
    repository.documents.iter_mut().find(|(id, _)| *id == oid(byte)).expect("fixture document").1 = document;
}

#[must_use]
pub fn tasks(names: &[(&[u8], bool)], placement: Option<Placement>) -> Decoded {
    use hearth_domain_render::revision::{Kind, Task, Value};
    Decoded::Document(Document {
        members: None,
        at: Position { line: 1, column: 1 },
        layer: Layer {
            debian: None,
            placement,
            tasks: names
                .iter()
                .map(|(name, removed)| Task {
                    name: Box::from(*name),
                    at: Position { line: 1, column: 2 },
                    value: Value {
                        at: Position { line: 1, column: 9 },
                        kind: if *removed { Kind::Null } else { Kind::Map(Box::new([])) },
                    },
                })
                .collect(),
        },
    })
}

use crate::expressions::Roots;
use hearth_domain_render::evaluate::{Entry as ValueEntry, Expression, Value};

// Head, repeated opening, count, middle, repeated closing, tail, description.
const EVALUATION_TEXTS: &[(&str, &str, usize, &str, &str, &str, &str)] = &[
    ("[0", ",0", 2046, "", "", "] ", "4096-byte list"),
    ("", "[", 64, "0", "]", "", "64 nested lists"),
    ("settings.text", " ~ settings.text", 16, "", "", "", "concatenation past made"),
    ("settings.text | replace('a', 'aaaaaaaaaaaaaaaaa')", "", 0, "", "", "", "replacement past made"),
    ("settings.large | tojson", "", 0, "", "", "", "tojson of 4096 values"),
    ("settings.large | sort", "", 0, "", "", "", "sort of 4096 values"),
    ("settings.text | split('aaa')", "", 0, "", "", "", "split of a MiB string"),
];
#[must_use]
pub fn at_the_limits() -> Vec<(Expression, &'static str)> {
    EVALUATION_TEXTS
        .iter()
        .map(|&(head, open, count, middle, close, tail, what)| {
            let text = [head, &open.repeat(count), middle, &close.repeat(count), tail].concat();
            let expression = hearth_protocol_template::expression::compile(
                text.as_bytes(),
                &hearth_protocol_template::limits::SHIPPED,
            )
            .expect("shipped-limit expression compiles");
            (hearth_protocol_builder::translate::operations::expression(expression), what)
        })
        .collect()
}
#[must_use]
pub fn wide_roots() -> Roots {
    let empty = || Value::Map(Box::new([]));
    Roots {
        target: empty(),
        debian: empty(),
        tasks: empty(),
        exports: empty(),
        bindings: Vec::new(),
        settings: Value::Map(Box::new([
            ValueEntry {
                key: Box::from(b"large".as_slice()),
                value: Some(Value::List((0..4096).rev().map(Value::Int).collect())),
            },
            ValueEntry {
                key: Box::from(b"text".as_slice()),
                value: Some(Value::String(vec![b'a'; 1_048_576].into_boxed_slice())),
            },
        ])),
    }
}

#[must_use]
pub fn broken_envs(count: u32) -> Repository {
    use hearth_domain_render::{errors::Decode, revision::Oid};
    let mut repository = x1_prod();
    for index in 0..count {
        let path = format!("envs/e{index:03}.json").into_bytes().into_boxed_slice();
        let oid = Oid(path.clone());
        repository.entries.push(Entry { path, oid: oid.clone(), mode: Mode::File, size: 24 });
        repository.documents.push((
            oid,
            Decoded::Refused {
                at: Position { line: 1, column: 14 },
                fault: Decode::Duplicate { key: b"motd".as_slice().into() },
                also: Some(Position { line: 1, column: 2 }),
            },
        ));
    }
    repository
}

#[must_use]
pub fn members(names: &[(&[u8], bool)]) -> Decoded {
    use hearth_domain_render::revision::{Member, Members};
    let Decoded::Document(mut document) = document(None) else { unreachable!("fixture document") };
    document.members = Some(Members {
        at: Position { line: 1, column: 2 },
        each: names
            .iter()
            .map(|(name, removed)| Member {
                name: Box::from(*name),
                at: Position { line: 1, column: 12 },
                layer: if *removed { None } else { Some(Layer { debian: None, placement: None, tasks: Box::new([]) }) },
            })
            .collect(),
    });
    Decoded::Document(document)
}
#[must_use]
pub fn cluster() -> Repository {
    let mut repository = x1_prod();
    set_document(&mut repository, b'2', members(&[(b"1", false), (b"2", false)]));
    repository
}
#[must_use]
pub fn with_int_setting() -> Repository {
    with_setting(hearth_domain_render::revision::declaration::Form::Int)
}
#[must_use]
pub fn with_setting(form: hearth_domain_render::revision::declaration::Form) -> Repository {
    with_default_setting(form, None)
}
fn with_default_setting(form: hearth_domain_render::revision::declaration::Form, default: Option<&[u8]>) -> Repository {
    use hearth_domain_render::revision::declaration::{Declaration, Spec, Text, Type};
    let mut repository = with_task();
    let at = Position { line: 2, column: 7 };
    set_document(
        &mut repository,
        b'4',
        Decoded::Declaration(Declaration {
            at,
            types: Box::new([Type { at, form }]),
            requires: Box::new([]),
            scripts: Box::new([]),
            exports: Box::new([]),
            entries: Box::new([]),
            reads: Box::new([]),
            settings: Box::new([Spec {
                name: Text { at, text: Box::from(b"x".as_slice()) },
                at,
                ty: 0,
                default: default.map(|value| hearth_domain_render::revision::Value {
                    at,
                    kind: hearth_domain_render::revision::Kind::String(Box::from(value)),
                }),
                optional: default.is_none(),
                min: None,
                max: None,
                max_length: None,
            }]),
        }),
    );
    repository
}
#[must_use]
pub fn task_patch(task: &[u8], setting: &[u8], kind: hearth_domain_render::revision::Kind) -> Decoded {
    use hearth_domain_render::revision::{Kind, Pair, Task, Value};
    let at = Position { line: 2, column: 7 };
    Decoded::Document(Document {
        members: None,
        at,
        layer: Layer {
            debian: None,
            placement: None,
            tasks: Box::new([Task {
                name: Box::from(task),
                at,
                value: Value {
                    at,
                    kind: Kind::Map(Box::new([Pair { key: Box::from(setting), at, value: Value { at, kind } }])),
                },
            }]),
        },
    })
}
#[must_use]
pub fn unused_object_typo() -> Repository {
    use hearth_domain_render::revision::{
        Kind, Pair, Value,
        declaration::{Form, Type},
    };
    let mut repository = with_int_setting();
    set_document(&mut repository, b'2', document(None));
    let (_, Decoded::Declaration(declaration)) =
        repository.documents.iter_mut().find(|(id, _)| *id == oid(b'4')).expect("task")
    else {
        panic!("declaration")
    };
    let at = declaration.at;
    let mut field = only(&declaration.settings).clone();
    field.ty = 1;
    field.at = Position { line: 2, column: 8 };
    declaration.types = Box::new([Type { at, form: Form::Object(Box::new([field])) }, Type { at, form: Form::Int }]);
    let [setting] = declaration.settings.as_mut() else { panic!("one setting") };
    setting.default = Some(Value {
        at,
        kind: Kind::Map(Box::new([Pair {
            key: Box::from(b"typo".as_slice()),
            at,
            value: Value { at, kind: Kind::Int(1) },
        }])),
    });
    repository
}

#[must_use]
pub fn containers() -> Repository {
    let mut repository = two_targets();
    container_on_host(&mut repository, b"a-b-prod");
    repository
}

#[must_use]
pub fn container_with_unsound_host() -> Repository {
    use hearth_domain_render::revision::Kind;
    let mut repository = with_int_setting();
    container_on_host(&mut repository, b"a-b-prod");
    repository.entries.push(entry(b"hosts/a-b/a-b-prod.json", b'5'));
    let Decoded::Document(mut host) = task_patch(b"motd", b"x", Kind::String(Box::from(b"bad".as_slice()))) else {
        panic!("document")
    };
    host.layer.placement = Some(placement());
    repository.documents.push((oid(b'5'), Decoded::Document(host)));
    repository
}

fn container_on_host(repository: &mut Repository, on: &[u8]) {
    let mut placed = placement();
    placed.kind = Runs::Container;
    placed.firmware = None;
    placed.hardware = None;
    placed.on =
        Some(hearth_domain_render::revision::On { target: Box::from(on), at: Position { line: 1, column: 43 } });
    set_document(repository, b'3', document(Some(placed)));
}

#[must_use]
pub fn tree_templates(files: &[(&[u8], Mode, &[u8])]) -> Repository {
    let mut repository = with_task();
    for (index, &(path, mode, body)) in files.iter().enumerate() {
        let byte = b'5' + u8::try_from(index).expect("small fixture");
        let mut entry = entry(path, byte);
        entry.mode = mode;
        entry.size = u64::try_from(body.len()).expect("fixture bytes");
        repository.entries.push(entry);
        repository.blobs.push((oid(byte), body.into()));
    }
    sort_entries(&mut repository);
    repository
}

#[must_use]
pub fn templates_for_many_targets() -> Repository {
    const NAMES: &[&str] = &["a", "b", "c", "d", "e", "f", "g", "h"];
    let mut repository = tree_templates(&[
        (b"tasks/motd/tree/etc/a.j2", Mode::File, b"{{ target.name }}a"),
        (b"tasks/motd/tree/etc/b.j2", Mode::File, b"{{ target.name }}b"),
        (b"tasks/motd/tree/etc/c.j2", Mode::File, b"{{ target.name }}c"),
    ]);
    for (index, name) in NAMES.iter().enumerate() {
        let byte = b'8' + u8::try_from(index).expect("small fixture");
        repository.entries.push(entry(format!("hosts/{name}/{name}-prod.json").as_bytes(), byte));
        repository.documents.push((oid(byte), tasks(&[(b"motd", false)], Some(placement()))));
    }
    sort_entries(&mut repository);
    repository
}

#[must_use]
pub fn with_default_target(peer: &[u8]) -> Repository {
    with_default_setting(hearth_domain_render::revision::declaration::Form::Target, Some(peer))
}

#[must_use]
pub fn with_peer_targets(peer: &[u8]) -> Repository {
    let mut repository = with_default_target(peer);
    repository.entries.extend([
        entry(b"envs/lab.json", b'5'),
        entry(b"hosts/x2/x2-prod.json", b'6'),
        entry(b"hosts/x2/x2-lab.json", b'7'),
    ]);
    repository.documents.extend([
        (oid(b'5'), document(None)),
        (oid(b'6'), document(Some(placement()))),
        (oid(b'7'), document(Some(placement()))),
    ]);
    repository
}

#[must_use]
pub fn templates_at_the_limits() -> [(hearth_domain_render::evaluate::template::Template, &'static str); 4] {
    let loops = format!("{}{{{{ loop.index }}}}{}", "{% for x in [1] %}".repeat(32), "{% endfor %}".repeat(32));
    let output = "{% for x in [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16] %}{{ settings.text }}{% endfor %}".to_owned();
    let sets = "{% set x = 1 %}".repeat(65_000);
    let chain = format!("{{{{ {} }}}}", vec!["settings.text"; 20].join(" ~ "));
    [(loops, "32 nested loops"), (output, "16 MiB output"), (sets, "set bindings"), (chain, "concat past made")].map(
        |(source, what)| {
            let compiled = hearth_protocol_template::template::compile(
                source.as_bytes(),
                &hearth_protocol_template::limits::SHIPPED,
            )
            .expect("limit template compiles");
            (hearth_protocol_builder::translate::instructions::template(compiled), what)
        },
    )
}

pub const EXPANSION_LIMITS: Limits = Limits { expansion: 4096, ..SHIPPED };
#[must_use]
pub fn entries_at_the_limits() -> [(hearth_domain_render::revision::declaration::Declaration, &'static str); 2] {
    [
        (
            crate::declarations::decode(br#"{"groups":{"{{ key }}":{"for_each":"settings.large","from":"x"}}}"#),
            "entry count",
        ),
        (
            crate::declarations::decode(
                br#"{"groups":{"{{ key }}":{"for_each":"settings.large","from":"{{ settings.text }}"}}}"#,
            ),
            "expanded bytes",
        ),
    ]
}

#[must_use]
pub fn render(repository: &Repository) -> crate::parent::Run {
    render_seeded(repository, 0)
}
#[must_use]
pub fn render_seeded(repository: &Repository, seed: u64) -> crate::parent::Run {
    render_targets(repository, &[], seed)
}
#[must_use]
pub fn configuration_error(
    kind: hearth_domain_render::errors::Kind,
    path: &[u8],
    task: Option<&[u8]>,
) -> hearth_domain_render::errors::Error {
    let mut name = None;
    if let Some(task) = task {
        name = Some(Box::from(task));
    }
    hearth_domain_render::errors::Error {
        side: None,
        kind,
        class: hearth_domain_render::vocabulary::Class::Configuration,
        at: Some(hearth_domain_render::errors::Place { path: Box::from(path), position: None }),
        also: None,
        target: None,
        task: name,
    }
}
pub struct Survivor<'a> {
    pub spec: &'a hearth_domain_render::parts::Spec,
    pub error: &'a hearth_domain_render::errors::Error,
}
#[must_use]
pub fn survivor(answers: &[hearth_domain_render::vocabulary::Answer]) -> Survivor<'_> {
    use hearth_domain_render::vocabulary::Answer;
    let mut answers = answers.iter();
    let Some(Answer::Spec(spec)) = answers.next() else { panic!("one surviving spec") };
    let Some(Answer::Error(error)) = answers.next() else { panic!("one error after the spec") };
    assert_eq!(answers.next(), None, "only the spec and its error");
    Survivor { spec, error }
}
#[must_use]
pub fn read_blobs(requests: &[hearth_domain_render::vocabulary::Request]) -> &[hearth_domain_render::vocabulary::Blob] {
    use hearth_domain_render::vocabulary::Request;
    let Request::Read { blobs, .. } =
        requests.iter().find(|rq| matches!(rq, Request::Read { .. })).expect("read request")
    else {
        panic!("read follows list")
    };
    blobs
}

#[must_use]
pub fn render_targets(repository: &Repository, targets: &[&[u8]], seed: u64) -> crate::parent::Run {
    crate::parent::run(repository, targets, seed)
}

#[must_use]
pub fn written_debian() -> hearth_domain_render::revision::Value {
    use hearth_domain_render::revision::{Kind, Pair, Value};
    let at = Position { line: 1, column: 1 };
    let text = |value: &[u8]| Value { at, kind: Kind::String(value.into()) };
    let pair = |name: &[u8], value| Pair { key: name.into(), at, value };
    Value {
        at,
        kind: Kind::Map(
            vec![
                pair(b"architecture", text(b"amd64")),
                pair(
                    b"pockets",
                    Value {
                        at,
                        kind: Kind::Map(
                            vec![
                                pair(b"security", Value { at, kind: Kind::Bool(false) }),
                                pair(b"updates", Value { at, kind: Kind::Bool(false) }),
                            ]
                            .into_boxed_slice(),
                        ),
                    },
                ),
                pair(
                    b"snapshot",
                    Value { at, kind: Kind::Map(vec![pair(b"archive", text(b"20261001T030405Z"))].into_boxed_slice()) },
                ),
                pair(b"suite", text(b"forky")),
            ]
            .into_boxed_slice(),
        ),
    }
}

/// The config part remains identifiable as other parts arrive.
#[must_use]
pub fn config_part(parts: &[hearth_domain_render::parts::Part]) -> &hearth_domain_render::parts::config::Config {
    parts
        .iter()
        .find_map(|part| match &part.record {
            hearth_domain_render::parts::PartRecord::Config(config) => Some(config),
            hearth_domain_render::parts::PartRecord::Data(_)
            | hearth_domain_render::parts::PartRecord::Secrets(_)
            | hearth_domain_render::parts::PartRecord::Software(_)
            | hearth_domain_render::parts::PartRecord::Live(_)
            | hearth_domain_render::parts::PartRecord::Packages(_)
            | hearth_domain_render::parts::PartRecord::Boot(_)
            | hearth_domain_render::parts::PartRecord::Exports(_) => None,
        })
        .expect("config part")
}

#[must_use]
pub fn template_and_link() -> Repository {
    let mut repository = with_task();
    repository.entries.push(entry(b"tasks/motd/tree/etc/template.j2", b'5'));
    repository.blobs.push((oid(b'5'), Box::from(b"ok".as_slice())));
    let mut link = entry(b"tasks/motd/tree/etc/link", b'6');
    link.mode = hearth_domain_render::revision::Mode::Link;
    repository.entries.push(link);
    repository.blobs.push((oid(b'6'), Box::from(b"/target".as_slice())));
    repository
}

mod resolution;
pub use resolution::configuration_requests;
pub mod exports;

/// Composition stories sample eight deterministic schedules.
pub const COMPOSITION_SEEDS: core::ops::Range<u64> = 0..8;

#[must_use]
pub fn container_with_on(on: &[u8]) -> Repository {
    let mut repository = two_targets();
    container_on_host(&mut repository, on);
    repository
}
#[must_use]
pub fn on_error(on: &[u8], host: hearth_domain_render::errors::Host) -> hearth_domain_render::errors::Kind {
    hearth_domain_render::errors::Kind::Composition(hearth_domain_render::errors::Composition::On {
        on: Box::from(on),
        host,
    })
}

pub mod goldens;

// render.md, 7.1: divide the retained 16 MiB among its 256 records.
pub const ERROR_RECORD_BYTES: usize = 16_777_216 / 256;
// at, also, target, task and guard name each contribute one byte.
pub const ERROR_FIELDS_BYTES: usize = 5;
#[must_use]
pub fn error_at_budget(bytes: usize) -> hearth_domain_render::errors::Error {
    use hearth_domain_render::errors::{Guard, Kind, Place};
    let mut error = configuration_error(
        Kind::Guard(Guard { name: Box::from(b"n".as_slice()), message: vec![b'x'; bytes - ERROR_FIELDS_BYTES].into() }),
        b"a",
        Some(b"s"),
    );
    error.target = Some(Box::from(b"t".as_slice()));
    error.also = Some(Place { path: Box::from(b"a".as_slice()), position: None });
    error
}

pub mod laws;
pub mod oracle;
pub use oracle::draw;
pub mod properties;
pub use crate::golden::repository_of;
pub use laws::merged;
pub use properties::digests;
mod decoded;
pub use decoded::*;
pub mod memory;
pub mod settings;

pub fn only_mut<T>(items: &mut [T]) -> &mut T {
    let [item] = items else { panic!("one item") };
    item
}
pub mod decoders;

pub mod specs;

fn sort_entries(repository: &mut Repository) {
    repository.entries.sort_by(|a, b| a.path.cmp(&b.path));
}
