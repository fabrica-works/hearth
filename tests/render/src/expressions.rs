//! Generated trees and contexts, independent of the postfix evaluator's implementation.
use hearth_domain_render::{
    evaluate::{self, Binding, Bindings, Budget, Context, Entry, Evaluator, Filter, Missing, Operand, Value},
    limits::Limits,
};
use skein_lib::Rng;
#[derive(Clone, Copy, Debug)]
pub struct Shape {
    pub depth: u32,
    pub width: u32,
}
pub const SMALL: Shape = Shape { depth: 3, width: 3 };
pub const WIDE: Shape = Shape { depth: 6, width: 6 };
pub const KINDS: &[&str] =
    &["unknown", "key", "index", "undefined", "type", "limit", "overflow", "output", "empty", "path"];
#[derive(Clone, Debug)]
pub struct Roots {
    pub target: Value,
    pub debian: Value,
    pub settings: Value,
    pub tasks: Value,
    pub exports: Value,
    pub bindings: Vec<(&'static [u8], Value)>,
}
impl Roots {
    #[must_use]
    pub const fn context(&self) -> Context<'_> {
        Context {
            target: &self.target,
            debian: &self.debian,
            settings: &self.settings,
            tasks: &self.tasks,
            exports: &self.exports,
        }
    }
}
#[derive(Clone, Debug)]
pub enum Tree {
    String(Vec<u8>),
    Int(i64),
    Bool(bool),
    Name(&'static [u8]),
    Key(Box<Tree>, &'static [u8], Missing),
    Item(Box<Tree>, Box<Tree>, Missing),
    List(Vec<Tree>),
    Defined(Box<Tree>, bool),
    Filter(Box<Tree>, Filter, Vec<Tree>),
    Not(Box<Tree>),
    Binary(Box<Tree>, Binary, Box<Tree>),
    And(Box<Tree>, Box<Tree>),
    Or(Box<Tree>, Box<Tree>),
    Conditional { condition: Box<Tree>, yes: Box<Tree>, no: Box<Tree> },
    Output(Box<Tree>),
}
#[derive(Clone, Copy, Debug)]
pub enum Binary {
    Equal,
    NotEqual,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
    In,
    NotIn,
    Concat,
    Add,
    Subtract,
}
const BINARIES: &[Binary] = &[
    Binary::Equal,
    Binary::NotEqual,
    Binary::Less,
    Binary::LessOrEqual,
    Binary::Greater,
    Binary::GreaterOrEqual,
    Binary::In,
    Binary::NotIn,
    Binary::Concat,
    Binary::Add,
    Binary::Subtract,
];

pub type Plain = Value;
#[must_use]
pub fn roots(rng: &mut Rng) -> Roots {
    fn root(rng: &mut Rng) -> Value {
        Value::Map(
            [b"a".as_slice(), b"b", b"c"]
                .into_iter()
                .map(|key| Entry {
                    key: Box::from(key),
                    value: if rng.chance(250) { None } else { Some(context_value(rng, 3)) },
                })
                .collect(),
        )
    }
    let bindings = if rng.chance(500) {
        vec![(b"key".as_slice(), Value::Int(2)), (b"value".as_slice(), context_value(rng, 3))]
    } else {
        vec![]
    };
    Roots { target: root(rng), debian: root(rng), settings: root(rng), tasks: root(rng), exports: root(rng), bindings }
}
fn context_value(rng: &mut Rng, depth: u32) -> Value {
    if depth == 0 {
        return Value::Int(i64::try_from(rng.below(5)).expect("small"));
    }
    match rng.below(5) {
        0 => Value::Bool(rng.chance(500)),
        1 => Value::String(vec![b'x'; usize::try_from(rng.below(50)).expect("small")].into_boxed_slice()),
        2 => Value::List((0..rng.below(4)).map(|_| context_value(rng, depth - 1)).collect()),
        3 => Value::Int(i64::try_from(rng.below(20)).expect("small") - 10),
        _ => Value::Map(
            [b"a".as_slice(), b"b", b"c"]
                .into_iter()
                .map(|key| Entry {
                    key: Box::from(key),
                    value: if rng.chance(250) { None } else { Some(context_value(rng, depth - 1)) },
                })
                .collect(),
        ),
    }
}
#[must_use]
pub fn tree(rng: &mut Rng, roots: &Roots, shape: Shape) -> Tree {
    let tree = grow(rng, roots, shape);
    if rng.chance(500) { Tree::Output(Box::new(tree)) } else { tree }
}
fn grow(rng: &mut Rng, roots: &Roots, shape: Shape) -> Tree {
    if shape.depth == 0 {
        return leaf(rng);
    }
    let sub = Shape { depth: shape.depth - 1, ..shape };
    match rng.below(18) {
        0 => leaf(rng),
        1 => {
            if roots.bindings.is_empty() {
                Tree::Name(b"settings")
            } else {
                Tree::Name(b"value")
            }
        }
        2 => Tree::Key(Box::new(grow(rng, roots, sub)), b"a", Missing::Error),
        3 => Tree::Item(
            Box::new(grow(rng, roots, sub)),
            Box::new(Tree::Int(i64::try_from(rng.below(6)).expect("small") - 1)),
            Missing::Error,
        ),
        4 => Tree::List((0..rng.below(u64::from(shape.width) + 1)).map(|_| grow(rng, roots, sub)).collect()),
        5 => Tree::Defined(
            Box::new(Tree::Key(
                Box::new(Tree::Key(Box::new(Tree::Name(b"settings")), b"a", Missing::Undefined)),
                b"b",
                Missing::Undefined,
            )),
            rng.chance(500),
        ),
        6 => Tree::Key(Box::new(Tree::Name(b"settings")), b"missing", Missing::Error),
        7 => Tree::Defined(Box::new(grow(rng, roots, sub)), rng.chance(500)),
        8 => Tree::List(vec![
            Tree::String(vec![b'x'; usize::try_from(rng.between(16, 4096)).expect("small")]);
            usize::try_from(shape.width).expect("small")
        ]),
        9 => Tree::Not(Box::new(grow(rng, roots, sub))),
        10 => Tree::Binary(
            Box::new(grow(rng, roots, sub)),
            BINARIES[usize::try_from(rng.below(11)).expect("small")],
            Box::new(grow(rng, roots, sub)),
        ),
        11 => Tree::And(Box::new(grow(rng, roots, sub)), Box::new(grow(rng, roots, sub))),
        12 => Tree::Or(Box::new(grow(rng, roots, sub)), Box::new(grow(rng, roots, sub))),
        13 => Tree::Conditional {
            condition: Box::new(grow(rng, roots, sub)),
            yes: Box::new(grow(rng, roots, sub)),
            no: Box::new(grow(rng, roots, sub)),
        },
        14 => Tree::Binary(Box::new(Tree::Int(i64::MAX)), Binary::Add, Box::new(Tree::Int(1))),
        15 => Tree::Binary(Box::new(Tree::String(b"x".to_vec())), Binary::Concat, Box::new(Tree::Int(-12))),
        _ => filtered(rng, roots, sub),
    }
}
fn leaf(rng: &mut Rng) -> Tree {
    match rng.below(5) {
        0 => Tree::String(vec![b'x'; usize::try_from(rng.below(128)).expect("small")]),
        1 => Tree::Int(i64::try_from(rng.below(5)).expect("small")),
        2 => Tree::Bool(rng.chance(500)),
        _ => Tree::Name(
            [b"target".as_slice(), b"debian", b"settings", b"tasks", b"exports", b"key", b"value", b"nope"]
                [usize::try_from(rng.below(8)).expect("small")],
        ),
    }
}
/// A generated tree as source for the real expression compiler.
#[must_use]
pub fn text(tree: &Tree) -> Vec<u8> {
    printed(tree).into_bytes()
}
fn wrapped(tree: &Tree) -> String {
    match tree {
        Tree::String(_) | Tree::Int(_) | Tree::Bool(_) | Tree::List(_) => printed(tree),
        Tree::Name(_)
        | Tree::Key(..)
        | Tree::Item(..)
        | Tree::Defined(..)
        | Tree::Filter(..)
        | Tree::Not(_)
        | Tree::Binary(..)
        | Tree::And(..)
        | Tree::Or(..)
        | Tree::Conditional { .. }
        | Tree::Output(_) => format!("({})", printed(tree)),
    }
}
pub(super) fn chain(tree: &Tree) -> bool {
    match tree {
        Tree::Name(_) => true,
        Tree::Key(input, _, _) | Tree::Item(input, _, _) => chain(input),
        Tree::String(_)
        | Tree::Int(_)
        | Tree::Bool(_)
        | Tree::List(_)
        | Tree::Defined(..)
        | Tree::Filter(..)
        | Tree::Not(_)
        | Tree::Binary(..)
        | Tree::And(..)
        | Tree::Or(..)
        | Tree::Conditional { .. }
        | Tree::Output(_) => false,
    }
}
fn lookup(tree: &Tree) -> String {
    if chain(tree) { printed(tree) } else { wrapped(tree) }
}
fn printed(tree: &Tree) -> String {
    match tree {
        Tree::String(value) => {
            let mut quoted = String::from("'");
            for character in core::str::from_utf8(value).expect("generated UTF-8").chars() {
                match character {
                    '\\' => quoted.push_str("\\\\"),
                    '\'' => quoted.push_str("\\'"),
                    '\n' => quoted.push_str("\\n"),
                    '\r' => quoted.push_str("\\r"),
                    '\t' => quoted.push_str("\\t"),
                    character => quoted.push(character),
                }
            }
            quoted.push('\'');
            quoted
        }
        Tree::Int(value) => value.to_string(),
        Tree::Bool(value) => value.to_string(),
        Tree::Name(name) => core::str::from_utf8(name).expect("generated name").to_owned(),
        Tree::Key(input, key, _) => format!("{}.{}", lookup(input), core::str::from_utf8(key).expect("key")),
        Tree::Item(input, index, _) => format!("{}[{}]", lookup(input), printed(index)),
        Tree::List(items) => format!("[{}]", items.iter().map(printed).collect::<Vec<_>>().join(",")),
        Tree::Defined(input, negated) => {
            format!("{} is {}defined", lookup(input), if *negated { "not " } else { "" })
        }
        Tree::Filter(input, filter, arguments) => {
            let name = match filter {
                Filter::Default => "default",
                Filter::Join => "join",
                Filter::Split => "split",
                Filter::First => "first",
                Filter::Last => "last",
                Filter::Length => "length",
                Filter::Replace => "replace",
                Filter::Items => "items",
                Filter::Keys => "keys",
                Filter::Values => "values",
                Filter::Sort => "sort",
                Filter::Tojson => "tojson",
                Filter::ShellQuote => "shell_quote",
                Filter::SystemdEscape => "systemd_escape",
                Filter::SystemdEscapePath => "systemd_escape_path",
                Filter::Basename => "basename",
                Filter::Dirname => "dirname",
            };
            let input = if *filter == Filter::Default { lookup(input) } else { wrapped(input) };
            format!("{input} | {name}({})", arguments.iter().map(printed).collect::<Vec<_>>().join(","))
        }
        Tree::Not(input) => format!("not {}", wrapped(input)),
        Tree::Binary(left, binary, right) => {
            let symbol = match binary {
                Binary::Equal => "==",
                Binary::NotEqual => "!=",
                Binary::Less => "<",
                Binary::LessOrEqual => "<=",
                Binary::Greater => ">",
                Binary::GreaterOrEqual => ">=",
                Binary::In => "in",
                Binary::NotIn => "not in",
                Binary::Concat => "~",
                Binary::Add => "+",
                Binary::Subtract => "-",
            };
            format!("{} {symbol} {}", wrapped(left), wrapped(right))
        }
        Tree::And(left, right) => format!("{} and {}", wrapped(left), wrapped(right)),
        Tree::Or(left, right) => format!("{} or {}", wrapped(left), wrapped(right)),
        Tree::Conditional { condition, yes, no } => {
            format!("{} if {} else {}", wrapped(yes), wrapped(condition), wrapped(no))
        }
        Tree::Output(input) => printed(input),
    }
}
/// # Errors
/// Returns the first evaluation fault kind.
pub fn evaluate(tree: &Tree, roots: &Roots, limits: &Limits) -> Result<Option<Plain>, &'static str> {
    let text = text(tree);
    let compiled = hearth_protocol_template::expression::compile(
        &text,
        &hearth_protocol_template::limits::Limits {
            expression: u32::try_from(text.len()).expect("generated text fits"),
            ..hearth_protocol_template::limits::SHIPPED
        },
    )
    .unwrap_or_else(|error| panic!("generated expression {}: {error:?}", String::from_utf8_lossy(&text)));
    let expression = hearth_protocol_builder::translate::operations::expression(compiled);
    let mut bindings = Bindings::with_capacity(u32::try_from(roots.bindings.len()).expect("small"));
    for (name, value) in &roots.bindings {
        bindings.push(Binding { name, value: Operand::Value(value) }).expect("room");
    }
    let result = evaluate::expression(
        &mut Evaluator::new(limits),
        &expression,
        &roots.context(),
        &bindings,
        &mut Budget::new(limits),
    )
    .map_err(|fault| super::fixtures::kind(fault.kind))?;
    if matches!(tree, Tree::Output(_)) {
        let written = evaluate::output(&result).map_err(super::fixtures::kind)?;
        let text = match written {
            evaluate::Written::Bytes(v) => v.to_vec(),
            evaluate::Written::Integer { negative, digits } => {
                let mut v = vec![];
                if negative {
                    v.push(b'-');
                }
                v.extend_from_slice(digits.as_bytes());
                v
            }
        };
        Ok(Some(Value::String(text.into_boxed_slice())))
    } else {
        Ok(owned(result))
    }
}
fn owned(operand: Operand<'_>) -> Option<Plain> {
    Some(match operand {
        Operand::Loop { .. } => panic!("loop needs a template context"),
        Operand::Undefined => return None,
        Operand::Value(v) => v.clone(),
        Operand::Bool(v) => Value::Bool(v),
        Operand::Int(v) => Value::Int(v),
        Operand::String(v) => Value::String(v),
        Operand::List { items, depth: _ } => {
            Value::List(items.into_iter().map(|v| owned(v).expect("defined list")).collect())
        }
    })
}

pub const FILTERS: &[Filter] = &[
    Filter::Default,
    Filter::Join,
    Filter::Split,
    Filter::First,
    Filter::Last,
    Filter::Length,
    Filter::Replace,
    Filter::Items,
    Filter::Keys,
    Filter::Values,
    Filter::Sort,
    Filter::Tojson,
    Filter::ShellQuote,
    Filter::SystemdEscape,
    Filter::SystemdEscapePath,
    Filter::Basename,
    Filter::Dirname,
];
fn filtered(rng: &mut Rng, roots: &Roots, shape: Shape) -> Tree {
    let filter = FILTERS[usize::try_from(rng.below(17)).expect("small")];
    let text = || Tree::String(b"a:b::a".to_vec());
    let (input, arguments) = match filter {
        Filter::Default => (Tree::Key(Box::new(Tree::Name(b"settings")), b"a", Missing::Undefined), vec![Tree::Int(9)]),
        Filter::Join => (Tree::List(vec![text(), Tree::Int(-12)]), vec![Tree::String(b",".to_vec())]),
        Filter::Split => (text(), vec![Tree::String(if rng.chance(200) { vec![] } else { b":".to_vec() })]),
        Filter::Replace => (text(), vec![Tree::String(if rng.chance(200) { vec![] } else { b"a".to_vec() }), text()]),
        Filter::First | Filter::Last => {
            (Tree::List(if rng.chance(200) { vec![] } else { vec![text(), Tree::Int(2)] }), vec![])
        }
        Filter::Items | Filter::Keys | Filter::Values => (Tree::Name(b"settings"), vec![]),
        Filter::Sort => (
            Tree::List(vec![Tree::String(b"b".to_vec()), Tree::String(b"a".to_vec()), Tree::String(b"b".to_vec())]),
            vec![],
        ),
        Filter::SystemdEscapePath | Filter::Basename | Filter::Dirname => {
            (Tree::String(if rng.chance(200) { b"/a/../b".to_vec() } else { b"/a/b".to_vec() }), vec![])
        }
        Filter::Length | Filter::ShellQuote | Filter::SystemdEscape => (text(), vec![]),
        Filter::Tojson => (grow(rng, roots, shape), vec![]),
    };
    let input = if rng.chance(200) { grow(rng, roots, shape) } else { input };
    Tree::Filter(Box::new(input), filter, arguments)
}
