//! Generated template trees, independent of the compiler's flat instructions.
use crate::expressions::{self, Roots, Tree as Expression};
use hearth_domain_render::{evaluate::Value, limits::Limits};
use skein_lib::Rng;
#[derive(Clone, Copy, Debug)]
pub struct Shape {
    pub depth: u32,
    pub width: u32,
}
pub const SMALL: Shape = Shape { depth: 3, width: 3 };
pub const WIDE: Shape = Shape { depth: 4, width: 4 };
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
        return leaf(rng, roots);
    }
    let sub = Shape { depth: shape.depth - 1, ..shape };
    match rng.below(7) {
        0 => leaf(rng, roots),
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
            body: Box::new(Tree::Sequence(vec![Tree::Output(Expression::Name(b"x")), tree(rng, roots, sub)])),
        },
        4 => Tree::For { pair: false, iterable: Expression::Name(b"settings"), body: Box::new(tree(rng, roots, sub)) },
        5 => Tree::For {
            pair: true,
            iterable: Expression::Filter(
                Box::new(Expression::Name(b"settings")),
                hearth_domain_render::evaluate::Filter::Items,
                vec![],
            ),
            body: Box::new(Tree::Sequence(vec![Tree::Output(Expression::Name(b"y")), tree(rng, roots, sub)])),
        },
        _ => Tree::Sequence(vec![
            Tree::Set { value: Expression::Int(i64::try_from(rng.below(10)).expect("small")) },
            Tree::Output(Expression::Name(b"x")),
        ]),
    }
}
const LOOP_FIELDS: &[&[u8]] = &[b"index", b"index0", b"length", b"first", b"last"];
fn leaf(rng: &mut Rng, roots: &Roots) -> Tree {
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
            if expressions::text(&expression).len() > maximum {
                Tree::Output(Expression::Int(1))
            } else {
                Tree::Output(expression)
            }
        }
    }
}
#[must_use]
pub fn text(tree: &Tree) -> Vec<u8> {
    match tree {
        Tree::Text(bytes) => bytes.clone(),
        Tree::Output(value) => [b"{{ ".as_slice(), &expressions::text(value), b" }}"].concat(),
        Tree::Sequence(children) => children.iter().flat_map(text).collect(),
        Tree::Branch { arms, otherwise } => {
            let mut bytes = vec![];
            for (index, (condition, body)) in arms.iter().enumerate() {
                bytes.extend_from_slice(if index == 0 { b"{% if " } else { b"{% elif " });
                bytes.extend(expressions::text(condition));
                bytes.extend_from_slice(b" %}");
                bytes.extend(text(body));
            }
            bytes.extend_from_slice(b"{% else %}");
            bytes.extend(text(otherwise));
            bytes.extend_from_slice(b"{% endif %}");
            bytes
        }
        Tree::For { pair, iterable, body } => [
            if *pair { b"{% for x, y in ".as_slice() } else { b"{% for x in ".as_slice() },
            &expressions::text(iterable),
            b" %}",
            &text(body),
            b"{% endfor %}",
        ]
        .concat(),
        Tree::Set { value } => [b"{% set x = ".as_slice(), &expressions::text(value), b" %}"].concat(),
    }
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
