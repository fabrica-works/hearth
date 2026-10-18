//! Typed expression trees that avoid session 30's listed engine differences.
use super::{Binary, Filter, Missing, Roots, Shape, Tree};
use skein_lib::Rng;
#[derive(Clone, Copy)]
enum Type {
    Int,
    Text,
    Bool,
}
pub(super) fn tree(rng: &mut Rng, roots: &Roots, shape: Shape) -> Tree {
    let ty = match rng.below(3) {
        0 => Type::Int,
        1 => Type::Text,
        _ => Type::Bool,
    };
    scalar(rng, roots, shape, ty)
}
fn filter(input: Tree, filter: Filter, arguments: Vec<Tree>) -> Tree {
    Tree::Filter(Box::new(input), filter, arguments)
}
fn text(value: &str) -> Tree {
    Tree::String(value.as_bytes().to_vec())
}
fn binary(left: Tree, op: Binary, right: Tree) -> Tree {
    Tree::Binary(Box::new(left), op, Box::new(right))
}
fn lookup(key: &'static [u8]) -> Tree {
    Tree::Key(Box::new(Tree::Name(b"settings")), key, Missing::Undefined)
}
fn leaf(rng: &mut Rng, roots: &Roots, ty: Type) -> Tree {
    match ty {
        Type::Int if !roots.bindings.is_empty() && rng.chance(250) => Tree::Name(b"key"),
        Type::Int => Tree::Int(i64::try_from(rng.below(1024)).expect("small integer") - 512),
        Type::Text => text(&format!("word{}", rng.below(16))),
        Type::Bool => Tree::Bool(rng.chance(500)),
    }
}
fn list(rng: &mut Rng, roots: &Roots, shape: Shape, ty: Type) -> Tree {
    Tree::List((0..rng.between(1, u64::from(shape.width))).map(|_| scalar(rng, roots, shape, ty)).collect())
}
fn scalar(rng: &mut Rng, roots: &Roots, shape: Shape, ty: Type) -> Tree {
    if shape.depth == 0 {
        return leaf(rng, roots, ty);
    }
    let sub = Shape { depth: shape.depth - 1, ..shape };
    match rng.below(8) {
        0 => leaf(rng, roots, ty),
        1 => Tree::Conditional {
            condition: Box::new(scalar(rng, roots, sub, Type::Bool)),
            yes: Box::new(scalar(rng, roots, sub, ty)),
            no: Box::new(scalar(rng, roots, sub, ty)),
        },
        2 => filter(lookup(b"missing"), Filter::Default, vec![scalar(rng, roots, sub, ty)]),
        _ => match ty {
            Type::Int => integer(rng, roots, sub),
            Type::Text => string(rng, roots, sub),
            Type::Bool => boolean(rng, roots, sub),
        },
    }
}
fn integer(rng: &mut Rng, roots: &Roots, sub: Shape) -> Tree {
    match rng.below(4) {
        0 => binary(scalar(rng, roots, sub, Type::Int), Binary::Add, scalar(rng, roots, sub, Type::Int)),
        1 => binary(scalar(rng, roots, sub, Type::Int), Binary::Subtract, scalar(rng, roots, sub, Type::Int)),
        2 => filter(Tree::Name(b"settings"), Filter::Length, vec![]),
        _ => filter(list(rng, roots, sub, Type::Int), Filter::First, vec![]),
    }
}
fn string(rng: &mut Rng, roots: &Roots, sub: Shape) -> Tree {
    match rng.below(12) {
        0 => binary(scalar(rng, roots, sub, Type::Text), Binary::Concat, scalar(rng, roots, sub, Type::Int)),
        1 => filter(list(rng, roots, sub, Type::Int), Filter::Join, vec![text(":")]),
        2 => filter(Tree::Name(if roots.bindings.is_empty() { b"settings" } else { b"value" }), Filter::Tojson, vec![]),
        3 => filter(filter(Tree::Name(b"settings"), Filter::Keys, vec![]), Filter::Join, vec![text(":")]),
        4 => filter(filter(list(rng, roots, sub, Type::Text), Filter::Sort, vec![]), Filter::Join, vec![text(":")]),
        5 => filter(filter(text("a:b::a"), Filter::Split, vec![text(":")]), Filter::Join, vec![text("-")]),
        6 => filter(scalar(rng, roots, sub, Type::Text), Filter::Replace, vec![text("word"), text("x")]),
        7 => filter(scalar(rng, roots, sub, Type::Text), Filter::ShellQuote, vec![]),
        8 => filter(text(".a/b-c"), Filter::SystemdEscape, vec![]),
        9 => filter(text("/a/b-c"), Filter::SystemdEscapePath, vec![]),
        10 => filter(text("/a/b"), if rng.chance(500) { Filter::Basename } else { Filter::Dirname }, vec![]),
        _ => filter(list(rng, roots, sub, Type::Text), Filter::Last, vec![]),
    }
}
fn boolean(rng: &mut Rng, roots: &Roots, sub: Shape) -> Tree {
    match rng.below(7) {
        0 => Tree::Defined(Box::new(lookup(if rng.chance(500) { b"a" } else { b"missing" })), rng.chance(500)),
        1 => Tree::Not(Box::new(scalar(rng, roots, sub, Type::Bool))),
        2 => Tree::And(Box::new(scalar(rng, roots, sub, Type::Bool)), Box::new(scalar(rng, roots, sub, Type::Bool))),
        3 => Tree::Or(Box::new(scalar(rng, roots, sub, Type::Bool)), Box::new(scalar(rng, roots, sub, Type::Bool))),
        4 => binary(scalar(rng, roots, sub, Type::Int), Binary::Equal, scalar(rng, roots, sub, Type::Int)),
        5 => binary(scalar(rng, roots, sub, Type::Text), Binary::Less, scalar(rng, roots, sub, Type::Text)),
        _ => binary(scalar(rng, roots, sub, Type::Int), Binary::In, list(rng, roots, sub, Type::Int)),
    }
}
