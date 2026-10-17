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
        crate::revision::Entry {
            path: bytes::copy_of(b"tasks/motd/task.json"),
            mode: crate::revision::Mode::File,
            oid: Oid(bytes::copy_of(b"declaration")),
            size: 2,
        },
    ]);
    call.documents = crate::revision::Documents::new(&call.entries);
    call.documents.set(
        &base,
        Document {
            members: None,
            at,
            layer: Layer {
                placement: None,
                tasks: Box::new([Task { name: bytes::copy_of(b"motd"), at, value: composition::map(at, &[]) }]),
            },
        },
    );
    call.documents.set(
        &host,
        Document {
            members: None,
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
    call.settings = Box::new([crate::compose::compose(&target, &call.documents, &call.defaults, &mut call.merge)]);
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

pub fn error_call(limits: &limits::Limits) -> crate::call::Side {
    use crate::{
        revision::{Entry, Mode, Oid},
        vocabulary::Output,
    };
    let entries = [
        Entry {
            path: bytes::copy_of(b"envs/prod.json"),
            mode: Mode::File,
            oid: Oid(bytes::copy_of(b"environment")),
            size: 24,
        },
        Entry {
            path: bytes::copy_of(b"hosts/x1/x1-prod.json"),
            mode: Mode::File,
            oid: Oid(bytes::copy_of(b"host")),
            size: 24,
        },
        Entry {
            path: bytes::copy_of(b"hosts/x1/x1.json"),
            mode: Mode::File,
            oid: Oid(bytes::copy_of(b"base")),
            size: 24,
        },
    ];
    let mut call = crate::call::Side::new(bytes::copy_of(b"/repo"), Box::new([]), limits);
    let mut out = skein_lib::Queue::<Output>::with_capacity(1);
    call.listed(*limits, Oid(bytes::copy_of(b"commit")), &entries, &mut out);
    call
}

pub fn filter(input: Value, filter: evaluate::Filter, arguments: &[Value]) -> Result<Option<Value>, Fault> {
    let mut roots = Roots::x1_prod();
    roots.target = input;
    let mut arguments = arguments.iter();
    if let Some(value) = arguments.next() {
        roots.debian = value.clone();
    }
    if let Some(value) = arguments.next() {
        roots.settings = value.clone();
    }
    let mut operations = skein_lib::List::with_capacity(4);
    operations.push(name(0, b"target", Missing::Error)).expect("room");
    if filter.arity() != 0 {
        operations.push(name(1, b"debian", Missing::Error)).expect("room");
    }
    if filter.arity() == 2 {
        operations.push(name(2, b"settings", Missing::Error)).expect("room");
    }
    operations.push(op(3, Op::Filter(filter))).expect("room");
    evaluate(&roots, operations.as_slice())
}

pub fn changed_file(
    before: u64,
    after: u64,
    digest: Digest,
) -> (crate::parts::config::EntryChange, crate::call::Sizes, crate::call::Sizes) {
    use crate::{parts::config, revision::Mode};
    let mut old = crate::call::Sizes::with_capacity(1);
    old.insert(Digest([1; 32]), before).expect("one cached digest");
    let mut new = crate::call::Sizes::with_capacity(1);
    new.insert(digest, after).expect("one cached digest");
    (
        config::EntryChange::Changed {
            path: bytes::copy_of(b"/etc/motd"),
            before: config::file(b"etc/motd", Mode::File, Digest([1; 32])).node,
            after: config::file(b"etc/motd", Mode::Executable, digest).node,
            sizes: None,
            hunks: None,
        },
        old,
        new,
    )
}
pub fn hunk_call() -> (crate::Render, skein_lib::Env<limits::Limits>, skein_lib::Queue<crate::vocabulary::Output>) {
    use crate::{
        call::Call,
        diff::Made,
        parts::{
            Part, PartRecord,
            config::{Config, EntryChange, TreeEntry},
        },
        vocabulary::Output,
    };
    use skein_lib::{Env, Queue, Time, Wall};
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits::SHIPPED };
    let mut out = Queue::with_capacity(1);
    let mut call = Call::diff(bytes::copy_of(b"/repo"), Box::new([]), bytes::copy_of(b"after"), &env.limits);
    let (EntryChange::Changed { before, after, .. }, old, new) = changed_file(9, 22, Digest([2; 32])) else {
        panic!("changed file")
    };
    for (node, sizes, digest) in [(before, old, Digest([1; 32])), (after, new, Digest([2; 32]))] {
        call.side.files = true;
        call.side.ready = true;
        call.side.commit = Some(crate::revision::Oid(bytes::copy_of(b"a")));
        call.side.sizes = sizes;
        call.side.made = skein_lib::Map::with_capacity(1);
        let mut made = spec(
            b"x1-prod",
            &[Part {
                digest,
                record: PartRecord::Config(Config {
                    tree: Box::new([TreeEntry { path: bytes::copy_of(b"/etc/motd"), node }]),
                }),
            }],
        );
        made.digest = digest;
        call.side.made.insert(bytes::copy_of(b"x1-prod"), Made::Spec(Box::new(made))).expect("one target");
        call.resume(env.limits, &mut out);
        drop(out.pop());
    }
    call.resume(env.limits, &mut out);
    assert_eq!(
        out.pop(),
        Some(Output::Request(crate::vocabulary::Request::Hunks {
            request: skein_lib::Token::new(3),
            before: Digest([1; 32]),
            after: Digest([2; 32])
        }))
    );
    (crate::Render { call: Some(call) }, env, out)
}

pub fn declaration_call() -> crate::call::Side {
    use crate::revision::{Entry, Mode, Oid};
    let entries = [
        Entry {
            path: bytes::copy_of(b"tasks/motd/task.json"),
            mode: Mode::File,
            oid: Oid(bytes::copy_of(b"motd")),
            size: 2,
        },
        Entry {
            path: bytes::copy_of(b"tasks/core/task.json"),
            mode: Mode::File,
            oid: Oid(bytes::copy_of(b"core")),
            size: 2,
        },
    ];
    let mut call = crate::call::Side::new(bytes::copy_of(b"/repo"), Box::new([]), &limits::SHIPPED);
    let mut out = skein_lib::Queue::with_capacity(1);
    call.listed(limits::SHIPPED, Oid(bytes::copy_of(b"commit")), &entries, &mut out);
    call
}

/// A place and target, using zero coordinates for a whole file or no place.
pub type Spot<'a> = (&'a [u8], u32, u32, &'a [u8]);
pub fn limits_with_errors(errors: u32) -> limits::Limits {
    limits::Limits { errors, ..limits::SHIPPED }
}
pub fn error_at(path: &[u8], line: u32, column: u32, target: &[u8]) -> crate::errors::Error {
    let mut error =
        crate::errors::refused(path, Some(composition::at(line, column)), crate::errors::Decode::Unexpected, None);
    error.target = Some(bytes::copy_of(target));
    error
}
pub fn unplaced(target: &[u8]) -> crate::errors::Error {
    let mut error = error_at(b"", 0, 0, target);
    error.at = None;
    error
}
pub fn collector(bound: u32) -> crate::errors::Collector {
    let mut collector = crate::errors::Collector::new(&limits_with_errors(bound));
    for path in [b"d", b"b", b"a", b"c"] {
        collector.push(error_at(path, 1, 1, b"t"));
    }
    collector
}
pub fn kept(call: &crate::call::Side) -> Box<[crate::errors::Error]> {
    call.errors.errors()
}
pub fn places(errors: &[crate::errors::Error]) -> Box<[Spot<'_>]> {
    let mut places = skein_lib::List::with_capacity(u32::try_from(errors.len()).expect("test errors"));
    for error in errors {
        let (path, line, column) = match &error.at {
            Some(place) => match place.position {
                Some(position) => (place.path.as_ref(), position.line, position.column),
                None => (place.path.as_ref(), 0, 0),
            },
            None => (b"".as_slice(), 0, 0),
        };
        places.push((path, line, column, error.target.as_deref().unwrap_or(b""))).expect("error count");
    }
    places.into_boxed()
}

pub fn declaration(
    settings: &[crate::revision::declaration::Spec],
    reads: &[crate::revision::declaration::Read],
) -> crate::revision::declaration::Declaration {
    crate::revision::declaration::Declaration {
        at: composition::at(1, 1),
        types: Box::new([]),
        settings: settings.into(),
        reads: reads.into(),
    }
}

pub fn declared_spec(name: &[u8]) -> crate::revision::declaration::Spec {
    use crate::revision::declaration::{Spec, Text};
    Spec {
        name: Text { at: composition::at(1, 2), text: bytes::copy_of(name) },
        at: composition::at(1, 12),
        ty: 0,
        default: None,
        optional: false,
        min: None,
        max: None,
        max_length: None,
    }
}
pub fn read(task: &[u8], settings: &[crate::revision::declaration::Text]) -> crate::revision::declaration::Read {
    use crate::revision::declaration::{Read, Text};
    Read {
        task: Text { at: composition::at(1, 12), text: bytes::copy_of(task) },
        at: composition::at(1, 20),
        settings: settings.into(),
        optional: false,
        exports: false,
    }
}
pub fn declarations(
    entries: &[(&[u8], Option<crate::revision::declaration::Declaration>)],
) -> crate::revision::Documents {
    use crate::revision::{Documents, Entry, Mode, Oid};
    let mut listed = skein_lib::List::with_capacity(u32::try_from(entries.len()).expect("fixture count"));
    for (task, _) in entries {
        let mut path = skein_lib::Writer::new(task.len().saturating_add(16));
        path.put(b"tasks/").expect("room");
        path.put(task).expect("room");
        path.put(b"/task.json").expect("room");
        listed
            .push(Entry { path: path.finish(), oid: Oid(bytes::copy_of(task)), mode: Mode::File, size: 2 })
            .expect("room");
    }
    let mut documents = Documents::new(listed.as_slice());
    for (task, declaration) in entries {
        if let Some(declaration) = declaration {
            documents.set_declaration(&Oid(bytes::copy_of(task)), declaration.clone());
        }
    }
    documents
}
pub fn check_declarations(documents: &crate::revision::Documents) -> Box<[(Box<[u8]>, crate::schema::Fault)]> {
    let mut faults = skein_lib::List::with_capacity(4);
    drop(crate::schema::declarations(
        documents,
        &mut crate::schema::Validator::new(&limits::SHIPPED),
        &mut crate::compose::Merge::new(&limits::SHIPPED),
        &mut faults,
    ));
    faults.into_boxed()
}

pub mod schemas;

pub fn listing(
    paths: &[(&[u8], crate::revision::Mode)],
    limits: &limits::Limits,
) -> (bool, Box<[crate::errors::Error]>) {
    let mut entries = skein_lib::List::with_capacity(u32::try_from(paths.len()).expect("fixture count"));
    for &(path, mode) in paths {
        entries
            .push(crate::revision::Entry {
                path: bytes::copy_of(path),
                mode,
                oid: crate::revision::Oid(bytes::copy_of(path)),
                size: 2,
            })
            .expect("fixture count");
    }
    let mut errors = crate::errors::Collector::new(limits);
    let valid = crate::compose::listing(entries.as_slice(), limits, &mut errors).is_ok();
    (valid, errors.finish().0)
}

pub fn hunk_failure() -> crate::vocabulary::Event {
    crate::vocabulary::Event::Failed {
        request: skein_lib::Token::new(3),
        class: crate::vocabulary::Class::Operational,
        at: None,
        failure: crate::vocabulary::Failure::Tool(crate::errors::Tool {
            command: crate::errors::Command::Diff,
            ended: crate::errors::Ended::Exit(2),
            tail: Box::new([]),
        }),
    }
}

pub fn template_call(path: &[u8], mode: crate::revision::Mode, size: u64, limits: limits::Limits) -> crate::call::Side {
    let (mut call, target) = removed_task();
    retain_task(&mut call);
    call.targets = Box::new([target]);
    call.commit = Some(crate::revision::Oid(bytes::copy_of(b"commit")));
    let mut entries = skein_lib::List::with_capacity(4);
    for entry in call.entries {
        entries.push(entry).expect("four entries");
    }
    entries
        .push(crate::revision::Entry {
            path: bytes::copy_of(path),
            mode,
            oid: crate::revision::Oid(bytes::copy_of(b"template")),
            size,
        })
        .expect("fourth entry");
    call.entries = entries.into_boxed();
    let mut out = skein_lib::Queue::with_capacity(1);
    call.files(limits, &mut out);
    call
}

pub fn shared_template_call() -> crate::call::Side {
    let mut call = template_call(b"tasks/motd/tree/etc/issue.j2", crate::revision::Mode::File, 20, limits::SHIPPED);
    let mut entries = skein_lib::List::with_capacity(5);
    for entry in call.entries {
        entries.push(entry).expect("five entries");
    }
    entries
        .push(crate::revision::Entry {
            path: bytes::copy_of(b"tasks/motd/tree/etc/motd.j2"),
            mode: crate::revision::Mode::File,
            oid: crate::revision::Oid(bytes::copy_of(b"template")),
            size: 20,
        })
        .expect("fifth entry");
    call.entries = entries.into_boxed();
    call
}

pub fn finish_render(
    render: &mut crate::Render,
    env: &skein_lib::Env<limits::Limits>,
    out: &mut skein_lib::Queue<crate::vocabulary::Output>,
) -> Box<[crate::vocabulary::Answer]> {
    let mut answers = skein_lib::List::with_capacity(env.limits.errors.saturating_add(10));
    for _ in 0..env.limits.errors.saturating_add(20) {
        if !render.work_pending() {
            break;
        }
        crate::resume(render, env, out);
        if let Some(crate::vocabulary::Output::Answer(answer)) = out.pop() {
            answers.push(answer).expect("answer capacity");
        }
    }
    answers.into_boxed()
}
pub fn failed_diff(bound: u32, count: u32) -> Box<[crate::vocabulary::Answer]> {
    use crate::{
        call::Call,
        diff::Made,
        vocabulary::{Class, Output},
    };
    let limits = limits_with_errors(bound);
    let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits };
    let mut call = Call::diff(bytes::copy_of(b"/repo"), Box::new([]), bytes::copy_of(b"after"), &limits);
    let mut out = skein_lib::Queue::<Output>::with_capacity(1);
    for commit in [b"a", b"b"] {
        call.side.files = true;
        call.side.ready = true;
        call.side.commit = Some(crate::revision::Oid(bytes::copy_of(commit)));
        call.side.made = skein_lib::Map::with_capacity(1);
        call.side.made.insert(bytes::copy_of(b"x1-prod"), Made::Failed(Class::Configuration)).expect("target");
        for _ in 0..count {
            call.side.errors.push(error_at(b"fleet.json", 1, 1, b"x1-prod"));
        }
        call.resume(limits, &mut out);
        drop(out.pop());
    }
    finish_render(&mut crate::Render { call: Some(call) }, &env, &mut out)
}

mod discovery;
pub use discovery::{
    container_documents, member_documents, member_layers, members, retain_task, rule_document, with_document,
};
