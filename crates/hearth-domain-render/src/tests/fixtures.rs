use alloc::boxed::Box;
use skein_lib::bytes;

use crate::{
    parts::meta::{Meta, Placement},
    revision::{Firmware, Runs},
};
use hearth_lib::encode::Digest;

pub fn hex(text: &str) -> Box<[u8]> {
    let mut digits = 0_usize;
    for byte in text.bytes() {
        if byte != b' ' {
            digits = digits.checked_add(1).expect("the fixture fits in memory");
        }
    }
    assert_eq!(digits % 2, 0, "hex pairs are whole");
    let mut result = bytes::zeroed(digits / 2);
    let mut offset = 0_usize;
    let mut high = None;
    for byte in text.bytes() {
        if byte == b' ' {
            continue;
        }
        let digit =
            u8::try_from(char::from(byte).to_digit(16).expect("a hex digit")).expect("a hex digit fits in a byte");
        if let Some(first) = high {
            let value: u8 = first;
            *result.get_mut(offset).expect("a pair has room") =
                value.checked_mul(16).expect("a high digit fits").checked_add(digit).expect("a byte fits");
            offset = offset.checked_add(1).expect("the fixture fits in memory");
            high = None;
        } else {
            high = Some(digit);
        }
    }
    result
}

pub fn digest(text: &str) -> Digest {
    let bytes = hex(text);
    Digest(bytes.as_ref().try_into().expect("a digest has thirty-two bytes"))
}

pub fn meta() -> Meta {
    Meta {
        target: bytes::copy_of(b"x1-prod"),
        base: bytes::copy_of(b"x1"),
        member: None,
        environment: bytes::copy_of(b"prod"),
        placement: Placement {
            kind: Runs::Metal,
            firmware: Some(Firmware::Efi),
            hardware: Some(bytes::copy_of(b"7th-gen")),
            on: None,
        },
    }
}

use crate::evaluate::{
    self, Bindings, Budget, Context, Entry, Evaluator, Expression, Fault, Missing, Op, Operand, Operation, Value,
};
use crate::limits;
#[derive(Debug)]
pub struct Roots {
    pub target: Value,
    pub debian: Value,
    pub settings: Value,
    pub tasks: Value,
    pub exports: Value,
}
impl Roots {
    pub fn context(&self) -> Context<'_> {
        Context {
            target: &self.target,
            debian: &self.debian,
            settings: &self.settings,
            tasks: &self.tasks,
            exports: &self.exports,
        }
    }
    pub fn x1_prod() -> Self {
        Self {
            target: evaluate::context::target(&meta()),
            debian: map(&[(b"suite", Some(string(b"forky")))]),
            settings: map(&[
                (b"enabled", Some(Value::Bool(true))),
                (b"name", Some(string(b"x1"))),
                (
                    b"peers",
                    Some(map(&[
                        (b"a", Some(map(&[(b"port", Some(Value::Int(51820)))]))),
                        (b"b", Some(map(&[(b"port", Some(Value::Int(51821)))]))),
                    ])),
                ),
                (b"port", Some(Value::Int(2222))),
                (b"tags", Some(list(&[string(b"sudo"), string(b"wheel")]))),
                (b"wifi", None),
            ]),
            tasks: map(&[(b"core", Some(map(&[(b"domain", Some(string(b"example.org")))]))), (b"wifi", None)]),
            exports: map(&[]),
        }
    }
}
pub fn string(value: &[u8]) -> Value {
    Value::String(bytes::copy_of(value))
}
pub fn list(values: &[Value]) -> Value {
    Value::List(Box::from(values))
}
pub fn map(values: &[(&[u8], Option<Value>)]) -> Value {
    let mut entries = skein_lib::List::with_capacity(u32::try_from(values.len()).expect("fixture fits"));
    for (key, value) in values {
        entries.push(Entry { key: bytes::copy_of(key), value: value.clone() }).expect("fixture fits");
    }
    Value::Map(entries.into_boxed())
}
pub fn op(at: u32, op: Op) -> Operation {
    Operation { at, op }
}
pub fn name(at: u32, text: &[u8], missing: Missing) -> Operation {
    op(at, Op::Name { name: bytes::copy_of(text), missing })
}
pub fn key(at: u32, text: &[u8], missing: Missing) -> Operation {
    op(at, Op::Key { name: bytes::copy_of(text), missing })
}
pub fn evaluate(roots: &Roots, operations: &[Operation]) -> Result<Option<Value>, Fault> {
    evaluate_at(roots, operations, &limits::SHIPPED)
}
pub fn evaluate_at(roots: &Roots, operations: &[Operation], limits: &limits::Limits) -> Result<Option<Value>, Fault> {
    let mut evaluator = Evaluator::new(limits);
    let expression = Expression {
        operations: Box::from(operations),
        values: u32::try_from(operations.len()).expect("fixture fits"),
    };
    match evaluate::expression(
        &mut evaluator,
        &expression,
        &roots.context(),
        &Bindings::with_capacity(0),
        &mut Budget::new(limits),
    ) {
        Ok(value) => Ok(owned(value)),
        Err(fault) => Err(fault),
    }
}
pub fn owned(value: Operand<'_>) -> Option<Value> {
    Some(match value {
        Operand::Undefined => return None,
        Operand::Value(v) => v.clone(),
        Operand::Bool(v) => Value::Bool(v),
        Operand::Int(v) => Value::Int(v),
        Operand::String(v) => Value::String(v),
        Operand::List { items, depth: _ } => {
            let mut values = skein_lib::List::with_capacity(u32::try_from(items.len()).expect("fixture fits"));
            for item in items {
                values.push(owned(item).expect("list items defined")).expect("fixture fits");
            }
            Value::List(values.into_boxed())
        }
    })
}

pub fn code(operations: &[Op]) -> Box<[Operation]> {
    let mut result = skein_lib::List::with_capacity(u32::try_from(operations.len()).expect("fixture fits"));
    for (at, operation) in operations.iter().enumerate() {
        result.push(op(u32::try_from(at).expect("fixture fits"), operation.clone())).expect("room");
    }
    result.into_boxed()
}
#[derive(Clone, Copy, Debug)]
pub enum Input {
    Undefined,
    Bool(bool),
    Int(i64),
    Text(&'static [u8]),
    List(&'static [Input]),
}
pub fn operand(input: Input) -> Operand<'static> {
    match input {
        Input::Undefined => Operand::Undefined,
        Input::Bool(v) => Operand::Bool(v),
        Input::Int(v) => Operand::Int(v),
        Input::Text(v) => Operand::String(bytes::copy_of(v)),
        Input::List(v) => {
            let mut items = skein_lib::List::with_capacity(u32::try_from(v.len()).expect("small"));
            for value in v {
                items.push(operand(*value)).expect("room");
            }
            Operand::List { depth: 1, items: items.into_boxed() }
        }
    }
}

pub const FALSE: bool = false;
pub fn config() -> crate::parts::config::Config {
    use crate::{
        parts::config::{self, Config},
        revision::Mode,
    };
    Config {
        tree: Box::new([
            config::file(
                b"etc/motd",
                Mode::File,
                digest("775324fcea7f2cdcbf11df25550e362669caceb6e69c8370374d2d436bb63123"),
            ),
            config::file(
                b"usr/local/bin/motd",
                Mode::Executable,
                digest("f487088ec8b18ff569dfede6f12e5fa91983f9f6dd2f3e0b6cee4349a231e745"),
            ),
        ]),
    }
}

pub fn removed_task() -> (crate::call::Side, crate::compose::Target) {
    use crate::revision::{Document, Layer, Oid, Position, Task};
    let at = Position { line: 1, column: 1 };
    let mut call = crate::call::Side::new(bytes::copy_of(b"/repo"), Box::new([]), &limits::SHIPPED);
    let base = Oid(bytes::copy_of(b"base"));
    let host = Oid(bytes::copy_of(b"host"));
    call.entries = Box::new([
        crate::revision::Entry {
            path: bytes::copy_of(b"hosts/x1/x1.json"),
            mode: crate::revision::Mode::File,
            oid: base.clone(),
            size: 2,
        },
        crate::revision::Entry {
            path: bytes::copy_of(b"hosts/x1/x1-prod.json"),
            mode: crate::revision::Mode::File,
            oid: host.clone(),
            size: 2,
        },
    ]);
    call.documents = crate::revision::Documents::new(&call.entries);
    call.documents.set(
        &base,
        Document {
            at,
            layer: Layer {
                placement: Some(crate::revision::Placement {
                    at,
                    kind: Runs::Metal,
                    firmware: Some(Firmware::Efi),
                    hardware: None,
                    on: None,
                    mutable: false,
                }),
                tasks: Box::new([Task { name: bytes::copy_of(b"motd"), at, value: composition::map(at, &[]) }]),
            },
        },
    );
    call.documents.set(
        &host,
        Document {
            at,
            layer: Layer {
                placement: Some(crate::revision::Placement {
                    at,
                    kind: Runs::Metal,
                    firmware: Some(Firmware::Efi),
                    hardware: None,
                    on: None,
                    mutable: false,
                }),
                tasks: Box::new([Task { name: bytes::copy_of(b"motd"), at, value: composition::null(at) }]),
            },
        },
    );
    let target = crate::compose::Target {
        name: bytes::copy_of(b"x1-prod"),
        base: bytes::copy_of(b"x1"),
        environment: bytes::copy_of(b"prod"),
        member: None,
        fleet: None,
        base_document: Some(1),
        environment_document: 1,
        base_env: 0,
    };
    call.settings = Box::new([crate::compose::compose(&target, &call.documents, &mut call.merge)]);
    (call, target)
}

pub fn only<T>(items: &[T]) -> &T {
    let [item] = items else { panic!("one item") };
    item
}
pub mod composition {
    use crate::revision::{Kind, Pair, Position, Value};
    pub const fn at(line: u32, column: u32) -> Position {
        Position { line, column }
    }
    pub fn int(at: Position, value: i64) -> Value {
        Value { at, kind: Kind::Int(value) }
    }
    pub fn null(at: Position) -> Value {
        Value { at, kind: Kind::Null }
    }
    pub fn string(at: Position, value: &[u8]) -> Value {
        Value { at, kind: Kind::String(skein_lib::bytes::copy_of(value)) }
    }
    pub fn map(at: Position, values: &[(&[u8], Position, Value)]) -> Value {
        let mut pairs = skein_lib::List::with_capacity(u32::try_from(values.len()).expect("fixture count"));
        for (key, at, value) in values {
            pairs
                .push(Pair { key: skein_lib::bytes::copy_of(key), at: *at, value: value.clone() })
                .expect("pair count");
        }
        Value { at, kind: Kind::Map(pairs.into_boxed()) }
    }
}

pub fn spec(target: &[u8], parts: &[crate::parts::Part]) -> crate::parts::Spec {
    let mut meta = meta();
    meta.target = bytes::copy_of(target);
    crate::parts::Spec {
        meta,
        meta_digest: Digest([1; 32]),
        digest: Digest([2; 32]),
        commit: crate::revision::Oid(bytes::copy_of(b"0123456789abcdef0123456789abcdef01234567")),
        parts: parts.into(),
    }
}

pub fn diff_trees() -> (crate::parts::config::Config, crate::parts::config::Config) {
    use crate::parts::config::{self, Config};
    let before = Config {
        tree: Box::new([
            config::file(b"changed", crate::revision::Mode::File, Digest([1; 32])),
            config::file(b"removed", crate::revision::Mode::File, Digest([1; 32])),
        ]),
    };
    let after = Config {
        tree: Box::new([
            config::file(b"added", crate::revision::Mode::File, Digest([2; 32])),
            config::file(b"changed", crate::revision::Mode::File, Digest([2; 32])),
        ]),
    };
    (before, after)
}

pub fn changed_specs() -> (crate::parts::Spec, crate::parts::Spec) {
    use crate::parts::{
        Part, PartRecord,
        config::{Config, Node, TreeEntry},
    };
    let before = spec(
        b"x1-prod",
        &[Part { digest: Digest([1; 32]), record: PartRecord::Config(Config { tree: Box::new([]) }) }],
    );
    let mut after = spec(
        b"x1-prod",
        &[Part {
            digest: Digest([2; 32]),
            record: PartRecord::Config(Config {
                tree: Box::new([TreeEntry {
                    path: bytes::copy_of(b"/added"),
                    node: Node::File {
                        content_digest: Digest([3; 32]),
                        mode: 0o644,
                        owner: bytes::copy_of(b"root"),
                        group: bytes::copy_of(b"root"),
                    },
                }]),
            }),
        }],
    );
    after.digest = Digest([3; 32]);
    (before, after)
}

pub fn diff_call() -> (crate::Render, skein_lib::Env<limits::Limits>, skein_lib::Queue<crate::vocabulary::Output>) {
    use crate::{Render, limits, vocabulary::Event};
    use skein_lib::{Env, Queue, Time, Wall};
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits::SHIPPED };
    let mut render = Render::new(&env.limits);
    let mut out = Queue::with_capacity(1);
    crate::step(
        &mut render,
        &env,
        Event::Diff {
            repository: bytes::copy_of(b"/repo"),
            from: bytes::copy_of(b"before"),
            to: bytes::copy_of(b"after"),
            targets: Box::new([]),
        },
        &mut out,
    );
    (render, env, out)
}
