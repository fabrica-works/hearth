//! Generated template trees, independent of the compiler's flat instructions.
use crate::expressions::{self, Roots, Style, Tree as Expression};
use hearth_domain_render::{evaluate::Value, limits::Limits};
use skein_lib::Rng;
#[derive(Clone, Copy, Debug)]
pub struct Shape {
    pub style: Style,
    pub depth: u32,
    pub width: u32,
}
pub const SMALL: Shape = Shape { style: Style::Hearth, depth: 3, width: 3 };
pub const WIDE: Shape = Shape { depth: 4, width: 4, ..SMALL };
pub const ORACLE: Shape = Shape { style: Style::Oracle, ..SMALL };
#[derive(Clone, Debug)]
pub enum Tree {
    Text(Vec<u8>),
    Output(Expression),
    Sequence(Vec<Tree>),
    Branch { arms: Vec<(Expression, Tree)>, otherwise: Box<Tree> },
    For { pair: bool, iterable: Expression, body: Box<Tree> },
    Set { value: Expression },
}
#[must_use]
pub fn tree(rng: &mut Rng, roots: &Roots, shape: Shape) -> Tree {
    if shape.depth == 0 {
        return leaf(rng, roots, shape.style);
    }
    let sub = Shape { depth: shape.depth - 1, ..shape };
    match rng.below(7) {
        0 => leaf(rng, roots, shape.style),
        1 => Tree::Sequence((0..rng.between(1, u64::from(shape.width))).map(|_| tree(rng, roots, sub)).collect()),
        2 => Tree::Branch {
            arms: vec![
                (Expression::Bool(rng.chance(500)), tree(rng, roots, sub)),
                (Expression::Bool(rng.chance(500)), tree(rng, roots, sub)),
            ],
            otherwise: Box::new(tree(rng, roots, sub)),
        },
        3 => Tree::For {
            pair: false,
            iterable: Expression::List(
                (0..rng.below(4)).map(|i| Expression::Int(i64::try_from(i).expect("small"))).collect(),
            ),
            body: Box::new(Tree::Sequence(vec![
                Tree::Output(Expression::Name(b"x")),
                loop_output(rng, shape.style),
                tree(rng, roots, sub),
            ])),
        },
        4 => Tree::For {
            pair: false,
            iterable: Expression::Name(b"settings"),
            body: Box::new(Tree::Sequence(vec![loop_output(rng, shape.style), tree(rng, roots, sub)])),
        },
        5 => Tree::For {
            pair: true,
            iterable: Expression::Filter(
                Box::new(Expression::Name(b"settings")),
                hearth_domain_render::evaluate::Filter::Items,
                vec![],
            ),
            body: Box::new(Tree::Sequence(vec![
                Tree::Output(if shape.style == Style::Oracle {
                    Expression::Filter(
                        Box::new(Expression::Name(b"y")),
                        hearth_domain_render::evaluate::Filter::Tojson,
                        vec![],
                    )
                } else {
                    Expression::Name(b"y")
                }),
                loop_output(rng, shape.style),
                tree(rng, roots, sub),
            ])),
        },
        _ => Tree::Sequence(vec![
            Tree::Set { value: Expression::Int(i64::try_from(rng.below(10)).expect("small")) },
            Tree::Output(Expression::Name(b"x")),
        ]),
    }
}
// Wide statement trees already explore nesting; the separate expression sweep draws depth six.
const ORACLE_EXPRESSION: expressions::Shape = expressions::Shape { depth: 2, ..expressions::ORACLE };
const LOOP_FIELDS: &[&[u8]] = &[b"index", b"index0", b"length", b"first", b"last"];
fn loop_output(rng: &mut Rng, style: Style) -> Tree {
    if style == Style::Hearth {
        return Tree::Text(Vec::new());
    }
    Tree::Output(Expression::Key(
        Box::new(Expression::Name(b"loop")),
        LOOP_FIELDS[usize::try_from(rng.below(LOOP_FIELDS.len() as u64)).expect("field")],
        hearth_domain_render::evaluate::Missing::Error,
    ))
}
fn leaf(rng: &mut Rng, roots: &Roots, style: Style) -> Tree {
    if style == Style::Oracle {
        return if rng.chance(250) {
            Tree::Text(b"text".to_vec())
        } else {
            Tree::Output(expressions::tree(rng, roots, ORACLE_EXPRESSION))
        };
    }
    match rng.below(4) {
        0 => Tree::Text(b"text".to_vec()),
        1 => Tree::Output(Expression::Key(
            Box::new(Expression::Name(b"loop")),
            LOOP_FIELDS[usize::try_from(rng.below(5)).expect("field")],
            hearth_domain_render::evaluate::Missing::Error,
        )),
        _ => {
            let mut roots = roots.clone();
            roots.bindings.clear();
            let expression = expressions::tree(rng, &roots, expressions::SMALL);
            let maximum =
                usize::try_from(hearth_protocol_template::limits::SHIPPED.expression).expect("expression bound");
            if expressions::text(&expression, expressions::Style::Hearth).len() > maximum {
                Tree::Output(Expression::Int(1))
            } else {
                Tree::Output(expression)
            }
        }
    }
}
/// The printer hands complete tags to the whitespace generator, never bytes to mutate.
pub enum Text {
    Literal(Vec<u8>),
    Output(String),
    Statement(String),
}
#[must_use]
pub fn tokens(tree: &Tree, style: Style) -> Vec<Text> {
    let expression = |tree| expressions::text(tree, style);
    match tree {
        Tree::Text(bytes) => vec![Text::Literal(bytes.clone())],
        Tree::Output(value) => vec![Text::Output(expression(value))],
        Tree::Sequence(children) => children.iter().flat_map(|child| tokens(child, style)).collect(),
        Tree::Branch { arms, otherwise } => {
            let mut out = Vec::new();
            for (index, (condition, body)) in arms.iter().enumerate() {
                let keyword = if index == 0 { "if" } else { "elif" };
                out.push(Text::Statement(format!("{keyword} {}", expression(condition))));
                out.extend(tokens(body, style));
            }
            out.push(Text::Statement("else".into()));
            out.extend(tokens(otherwise, style));
            out.push(Text::Statement("endif".into()));
            out
        }
        Tree::For { pair, iterable, body } => {
            let names = if *pair { "x, y" } else { "x" };
            let mut out = vec![Text::Statement(format!("for {names} in {}", expression(iterable)))];
            out.extend(tokens(body, style));
            out.push(Text::Statement("endfor".into()));
            out
        }
        Tree::Set { value } => vec![Text::Statement(format!("set x = {}", expression(value)))],
    }
}
#[must_use]
pub fn text(tree: &Tree) -> Vec<u8> {
    tokens(tree, Style::Hearth)
        .into_iter()
        .flat_map(|token| match token {
            Text::Literal(bytes) => bytes,
            Text::Output(value) => format!("{{{{ {value} }}}}").into_bytes(),
            Text::Statement(value) => format!("{{% {value} %}}").into_bytes(),
        })
        .collect()
}
/// Constructs present in the generated tree; dressing adds its comments and raw envelope.
#[must_use]
pub fn constructs(tree: &Tree) -> std::collections::BTreeSet<&'static str> {
    let mut found = std::collections::BTreeSet::new();
    for token in tokens(tree, Style::Oracle) {
        match token {
            Text::Literal(_) => {
                found.insert("text");
            }
            Text::Output(_) => {
                found.insert("output");
            }
            Text::Statement(ref body) => {
                for keyword in ["if", "elif", "else", "endif", "for", "endfor", "set"] {
                    if body.split_whitespace().next() == Some(keyword) {
                        found.insert(keyword);
                    }
                }
            }
        }
        let body = match &token {
            Text::Output(body) | Text::Statement(body) => body.as_str(),
            Text::Literal(_) => "",
        };
        for filter in FILTER_CONSTRUCTS {
            if body.contains(&format!("| {filter}")) {
                found.insert(*filter);
            }
        }
    }
    found
}
const FILTER_CONSTRUCTS: &[&str] = &[
    "default",
    "join",
    "split",
    "first",
    "last",
    "length",
    "replace",
    "items",
    "keys",
    "sort",
    "tojson",
    "shell_quote",
    "systemd_escape",
    "systemd_escape_path",
    "basename",
    "dirname",
];
pub const CONSTRUCTS: &[&str] = &[
    "output",
    "if",
    "elif",
    "else",
    "endif",
    "for",
    "endfor",
    "set",
    "default",
    "join",
    "split",
    "first",
    "last",
    "length",
    "replace",
    "items",
    "keys",
    "sort",
    "tojson",
    "shell_quote",
    "systemd_escape",
    "systemd_escape_path",
    "basename",
    "dirname",
];
#[must_use]
pub fn output_of(expression: &str) -> String {
    format!("{{{{ {expression} }}}}")
}
/// # Errors
/// Returns the first rendered fault, by kind.
pub fn run(tree: &Tree, roots: &Roots, limits: &Limits) -> Result<Vec<u8>, &'static str> {
    let bytes = text(tree);
    let compiled = hearth_protocol_template::template::compile(&bytes, &hearth_protocol_template::limits::SHIPPED)
        .unwrap_or_else(|error| panic!("generated template {}: {error:?}", String::from_utf8_lossy(&bytes)));
    let template = hearth_protocol_builder::translate::instructions::template(compiled);
    hearth_domain_render::evaluate::template::run(
        &template,
        &roots.context(),
        None,
        limits,
        u64::from(limits.rendered_file),
    )
    .map(<[u8]>::into_vec)
    .map_err(|fault| crate::fixtures::kind(fault.kind))
}
pub(super) fn loop_value(index: usize, length: usize) -> Value {
    use hearth_domain_render::evaluate::Entry;
    Value::Map(
        [
            (b"first".as_slice(), Value::Bool(index == 0)),
            (b"index".as_slice(), Value::Int(i64::try_from(index + 1).expect("small"))),
            (b"index0".as_slice(), Value::Int(i64::try_from(index).expect("small"))),
            (b"last".as_slice(), Value::Bool(index + 1 == length)),
            (b"length".as_slice(), Value::Int(i64::try_from(length).expect("small"))),
        ]
        .into_iter()
        .map(|(key, value)| Entry { key: Box::from(key), value: Some(value) })
        .collect(),
    )
}
