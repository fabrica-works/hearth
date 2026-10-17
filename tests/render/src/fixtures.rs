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
        layer: Layer { placement, tasks: Box::new([]) },
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
        ],
        documents: vec![
            (oid(b'1'), document(None)),
            (oid(b'2'), document(None)),
            (oid(b'3'), document(Some(placement()))),
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
        Limits { made: rng.between(16, 4096), nesting: u32::try_from(rng.between(1, 8)).expect("small"), ..SHIPPED }
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
    repository.entries.sort_by(|a, b| a.path.cmp(&b.path));
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
                layer: if *removed { None } else { Some(Layer { placement: None, tasks: Box::new([]) }) },
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
    use hearth_domain_render::revision::declaration::{Declaration, Spec, Text, Type};
    let mut repository = with_task();
    let at = Position { line: 2, column: 7 };
    set_document(
        &mut repository,
        b'4',
        Decoded::Declaration(Declaration {
            at,
            types: Box::new([Type { at, form }]),
            reads: Box::new([]),
            settings: Box::new([Spec {
                name: Text { at, text: Box::from(b"x".as_slice()) },
                at,
                ty: 0,
                default: None,
                optional: true,
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
