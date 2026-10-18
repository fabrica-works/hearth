use super::{composition::at, declaration, expr, key, name, op};
use crate::{
    evaluate::{Missing, Op},
    revision::{
        Position,
        declaration::{Condition, Declaration, Guard, Text},
    },
};
use alloc::boxed::Box;
use skein_lib::bytes::copy_of;
pub fn guard(name: &[u8], at: Position, when: Condition, message: &[u8]) -> Guard {
    Guard { name: Text { at, text: copy_of(name) }, at, when, message: Text { at, text: copy_of(message) } }
}
pub fn requires<const N: usize>(guards: [Guard; N]) -> Declaration {
    let mut declared = declaration(&[], &[]);
    declared.requires = Box::new(guards);
    declared
}
pub fn port_is(value: i64, at: Position) -> Condition {
    Condition {
        at,
        expression: expr(
            2,
            [
                name(0, b"settings", Missing::Error),
                key(9, b"port", Missing::Error),
                op(17, Op::Integer(value)),
                op(14, Op::Equal),
            ],
        ),
    }
}
pub fn optional_guard() -> Declaration {
    requires([guard(
        b"wifi",
        at(3, 5),
        Condition {
            at: at(3, 20),
            expression: expr(1, [name(0, b"settings", Missing::Error), key(9, b"wifi", Missing::Error)]),
        },
        b"wifi needed",
    )])
}
pub fn guarded_tasks() -> (crate::call::Side, crate::schema::Validated) {
    let declared = requires([guard(b"port", at(3, 5), port_is(22, at(3, 20)), b"port needed")]);
    let mut side = super::declaration_call();
    side.documents = super::declarations(&[(b"core", Some(declared.clone())), (b"motd", Some(declared))]);
    let roots = super::Roots::x1_prod();
    let tasks = [b"core".as_slice(), b"motd".as_slice()].map(|name| crate::schema::Checked {
        name: copy_of(name),
        settings: roots.settings.clone(),
        tasks: roots.tasks.clone(),
    });
    (side, crate::schema::Validated { tasks: Box::new(tasks) })
}
pub fn only_mut<T>(items: &mut [T]) -> &mut T {
    let [item] = items else { panic!("one item") };
    item
}

use crate::{
    evaluate::template::{Does, Instruction},
    revision::declaration::{Entries, Entry, Interpolated, Map},
};
pub const THREE_ENTRIES: crate::limits::Limits = crate::limits::Limits { expansion: 3, ..crate::limits::SHIPPED };
pub fn parts<const N: usize>(parts: [Does; N]) -> Interpolated {
    Interpolated::Parts { at: at(3, 5), parts: Box::new(parts.map(instruction)) }
}
fn instruction(does: Does) -> Instruction {
    Instruction { at: 0, does }
}
pub fn entry_output<const N: usize>(operations: [crate::evaluate::Operation; N]) -> Does {
    super::output(0, expr(8, operations)).does
}
pub fn condition<const N: usize>(operations: [crate::evaluate::Operation; N]) -> Condition {
    Condition { at: at(3, 20), expression: expr(8, operations) }
}
pub fn plain(text: &[u8]) -> Interpolated {
    Interpolated::Plain { at: at(3, 5), text: copy_of(text) }
}
pub fn entry(key: Interpolated, for_each: Option<Condition>, when: Option<Condition>) -> Entry {
    Entry { key, at: at(3, 15), when, for_each, fields: Box::new([]) }
}
pub fn entry_maps<const N: usize>(maps: [Entries; N]) -> Declaration {
    let mut declaration = declaration(&[], &[]);
    declaration.entries = Box::new(maps);
    declaration
}
pub fn entries<const N: usize>(entries: [Entry; N]) -> Declaration {
    entry_maps([entry_map(Map::Groups, entries)])
}
pub fn entry_map<const N: usize>(map: Map, entries: [Entry; N]) -> Entries {
    Entries { map, at: at(2, 11), entries: Box::new(entries) }
}
pub fn setting(name: &[u8]) -> Condition {
    condition([super::name(0, b"settings", Missing::Error), key(9, name, Missing::Error)])
}
pub fn list_entry() -> Entry {
    entry(
        parts([
            Does::Text(copy_of(b"/etc/")),
            entry_output([name(0, b"key", Missing::Error)]),
            Does::Text(copy_of(b"-")),
            entry_output([name(0, b"value", Missing::Error)]),
        ]),
        Some(setting(b"tags")),
        None,
    )
}
pub fn numeric_entry(length: u32) -> Entry {
    let mut operations = skein_lib::List::with_capacity(length.saturating_add(1));
    for value in 0..length {
        operations.push(op(0, Op::Integer(i64::from(value)))).expect("length");
    }
    operations.push(op(0, Op::List { length })).expect("length");
    entry(
        parts([entry_output([name(0, b"key", Missing::Error)])]),
        Some(Condition {
            at: at(3, 20),
            expression: crate::evaluate::Expression { operations: operations.into_boxed(), values: length },
        }),
        None,
    )
}
pub fn expanded_keys(expanded: &crate::entries::Expanded) -> Box<[&[u8]]> {
    let mut keys = skein_lib::List::with_capacity(crate::limits::SHIPPED.expansion);
    for group in &expanded.groups {
        for entry in &group.entries {
            keys.push(entry.key.as_ref()).expect("bounded entries");
        }
    }
    keys.into_boxed()
}
pub fn peers_with_absent() -> super::Roots {
    let mut roots = super::Roots::x1_prod();
    roots.settings = super::map(&[(
        b"peers",
        Some(super::map(&[
            (b"a", Some(crate::evaluate::Value::Int(1))),
            (b"absent", None),
            (b"b", Some(crate::evaluate::Value::Int(RANGE_COUNT))),
        ])),
    )]);
    roots
}

use crate::revision::declaration::{Field, Literal, Name};
pub const FIELD_AT: Position = Position { line: 3, column: 21 };
pub const EXPANDED_TEXT: crate::limits::Limits = crate::limits::Limits { expanded: 64, ..crate::limits::SHIPPED };
pub fn field(name: Name, value: Literal) -> Field {
    Field { name, at: FIELD_AT, value }
}
pub fn fields<const N: usize>(fields: [Field; N]) -> Declaration {
    let mut entry = entry(plain(b"ops"), None, None);
    entry.fields = Box::new(fields);
    entries([entry])
}
pub fn setting_tag(name: &[u8]) -> Interpolated {
    Interpolated::Parts { at: FIELD_AT, parts: Box::new([super::output(0, setting(name).expression)]) }
}
#[derive(Clone, Copy, Debug)]
pub enum StringPlace {
    Key,
    Field,
    Mixed,
    List,
}
pub fn boolean_string(place: StringPlace) -> Declaration {
    let tag = parts([entry_output([op(0, Op::Boolean(true))])]);
    match place {
        StringPlace::Key => entries([entry(tag, None, None)]),
        StringPlace::Field => fields([field(Name::From, Literal::Text(tag))]),
        StringPlace::Mixed => fields([field(
            Name::From,
            Literal::Text(parts([Does::Text(copy_of(b"x")), entry_output([op(0, Op::Boolean(true))])])),
        )]),
        StringPlace::List => fields([field(Name::Groups, Literal::Strings { at: FIELD_AT, each: Box::new([tag]) })]),
    }
}
pub fn mixed_optional() -> Interpolated {
    let Interpolated::Parts { at: _, parts: tagged } = setting_tag(b"wifi") else { panic!("tag") };
    let output = super::only(&tagged).clone();
    Interpolated::Parts { at: FIELD_AT, parts: Box::new([super::text(0, b"prefix"), output]) }
}

pub fn budget_text() -> Interpolated {
    let mut text = skein_lib::bytes::zeroed(usize::try_from(EXPANDED_TEXT.expanded).expect("small limit"));
    text.fill(b'x');
    Interpolated::Plain { at: FIELD_AT, text }
}
pub fn range_roots() -> super::Roots {
    let mut roots = super::Roots::x1_prod();
    roots.settings = super::map(&[(
        b"range",
        Some(super::map(&[
            (b"base", Some(crate::evaluate::Value::Int(RANGE_BASE))),
            (b"count", Some(crate::evaluate::Value::Int(RANGE_COUNT))),
        ])),
    )]);
    roots
}
pub fn range_field(count: Literal) -> Field {
    field(
        Name::Subordinate,
        Literal::Range {
            at: FIELD_AT,
            fields: Box::new([
                field(
                    Name::Base,
                    Literal::Int(crate::revision::declaration::Number { at: FIELD_AT, value: RANGE_BASE }),
                ),
                field(Name::Count, count),
            ]),
        },
    )
}

pub fn expansion(
    declared: &Declaration,
    limits: &crate::limits::Limits,
    left: u64,
) -> Result<crate::entries::Expanded, crate::entries::Fault> {
    expansion_budget(declared, limits, left).result
}
struct ExpansionBudget {
    result: Result<crate::entries::Expanded, crate::entries::Fault>,
    remaining: u64,
}
fn expansion_budget(declared: &Declaration, limits: &crate::limits::Limits, left: u64) -> ExpansionBudget {
    with_roots(declared, limits, left, &super::Roots::x1_prod())
}
fn with_roots(
    declared: &Declaration,
    limits: &crate::limits::Limits,
    mut left: u64,
    roots: &super::Roots,
) -> ExpansionBudget {
    let result = crate::entries::expand(declared, &roots.context(), limits, &mut left);
    ExpansionBudget { result, remaining: left }
}
pub fn expansion_remaining(declared: &Declaration, left: u64) -> Result<u64, crate::entries::Fault> {
    let budget = expansion_budget(declared, &crate::limits::SHIPPED, left);
    remaining(budget)
}
fn remaining(budget: ExpansionBudget) -> Result<u64, crate::entries::Fault> {
    match budget.result {
        Ok(_) => Ok(budget.remaining),
        Err(fault) => Err(fault),
    }
}
pub const UNDEFINED_FIELD: crate::entries::Fault = crate::entries::Fault {
    kind: crate::errors::Kind::Evaluation(crate::evaluate::FaultKind::Undefined),
    at: FIELD_AT,
    also: None,
};

/// Expand an ordinary declaration with the shipped target budget.
pub fn expanded(declared: &Declaration) -> crate::entries::Expanded {
    expansion(declared, &crate::limits::SHIPPED, crate::limits::SHIPPED.expanded).expect("expand")
}
pub fn evaluation_fault(kind: crate::evaluate::FaultKind, at: Position) -> crate::entries::Fault {
    crate::entries::Fault { kind: crate::errors::Kind::Evaluation(kind), at, also: None }
}
pub fn key_collision(key: &[u8], at: Position, first: Position) -> crate::entries::Fault {
    crate::entries::Fault {
        kind: crate::errors::Kind::Collision(crate::errors::Collision::Key { key: copy_of(key) }),
        at,
        also: Some(first),
    }
}

// The group's entry costs 32 and its key, "ops", costs three bytes.
pub const ENTRY_BYTES: u64 = 35;
// A kept "from" field adds 32 and its four-byte name to the entry.
pub const FIELD_BYTES: u64 = ENTRY_BYTES + crate::evaluate::ELEMENT + 4;
// A kept "groups" field adds 32 + six name bytes, and its empty string adds 32.
pub const EMPTY_LIST_BYTES: u64 = ENTRY_BYTES + crate::evaluate::ELEMENT * 2 + 6;
pub const KEY_LIMIT: crate::entries::Fault = crate::entries::Fault {
    kind: crate::errors::Kind::Evaluation(crate::evaluate::FaultKind::Limit),
    at: Position { line: 3, column: 5 },
    also: None,
};
pub const FIELD_LIMIT: crate::entries::Fault = crate::entries::Fault { at: FIELD_AT, ..KEY_LIMIT };
pub fn empty_string_field() -> Declaration {
    let mut from = field(Name::From, Literal::Plain { at: FIELD_AT, text: Box::new([]) });
    from.at = KEY_LIMIT.at;
    fields([from])
}
pub fn empty_list_field() -> Declaration {
    fields([field(Name::Groups, Literal::Strings { at: FIELD_AT, each: Box::new([plain(b"")]) })])
}

pub fn absent_fields() -> Declaration {
    fields([
        field(Name::From, Literal::Text(setting_tag(b"wifi"))),
        field(Name::Uid, Literal::Typed(setting_tag(b"wifi"))),
    ])
}

pub const RANGE_BASE: i64 = 42;
pub const RANGE_COUNT: i64 = 2;
// The entry (35), subordinate field (32 + 11), and nested fields (36 + 37).
pub const RANGE_BYTES: u64 = 151;
#[derive(Clone, Copy)]
pub enum RangeSource {
    Literal,
    Typed,
}
fn range_budget(source: RangeSource, left: u64) -> ExpansionBudget {
    let field = match source {
        RangeSource::Literal => {
            range_field(Literal::Int(crate::revision::declaration::Number { at: FIELD_AT, value: RANGE_COUNT }))
        }
        RangeSource::Typed => field(Name::Subordinate, Literal::Typed(setting_tag(b"range"))),
    };
    with_roots(&fields([field]), &crate::limits::SHIPPED, left, &range_roots())
}
pub fn range_remaining(source: RangeSource, left: u64) -> Result<u64, crate::entries::Fault> {
    remaining(range_budget(source, left))
}
pub fn expanded_range() -> crate::entries::Expanded {
    range_budget(RangeSource::Typed, crate::limits::SHIPPED.expanded).result.expect("expand")
}
