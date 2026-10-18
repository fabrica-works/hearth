//! Session 30's shared oracle contexts and seed range.
use crate::expressions::{self, Roots};
use hearth_domain_render::evaluate::{Entry, Value};
use skein_lib::Rng;
/// The expression comparison draws 200 contexts from session 30.4's block.
pub const CASES: core::ops::Range<u64> = 0..200;
pub const SEED: u64 = super::properties::SEED + 0x4000;
pub struct Row<T> {
    pub name: &'static str,
    pub template: &'static str,
    pub expected: T,
}
/// Check literal rows against both engines without deriving expected values.
pub fn pinned<T: Clone + PartialEq + std::fmt::Debug, E>(
    rows: &[Row<T>],
    engine: &E,
    outcomes: impl Fn(&E, &str, &Roots) -> (T, T),
) {
    let roots = roots();
    for row in rows {
        assert_eq!(
            outcomes(engine, row.template, &roots),
            (row.expected.clone(), row.expected.clone()),
            "{}",
            row.name
        );
    }
}
#[must_use]
pub fn draw(seed: u64) -> (Rng, Roots) {
    let mut rng = Rng::new(seed);
    let roots = expressions::roots(&mut rng);
    (rng, roots)
}
#[must_use]
pub fn roots() -> Roots {
    let mut roots = expressions::roots(&mut Rng::new(SEED));
    roots.settings = Value::Map(
        [(b"a".as_slice(), Some(Value::Int(2))), (b"absent".as_slice(), None), (b"b".as_slice(), Some(Value::Int(2)))]
            .into_iter()
            .map(|(key, value)| Entry { key: key.into(), value })
            .collect(),
    );
    roots
}

/// Session 30.5 draws 200 templates and 3,750 sweep inputs from its own seed block.
pub const TEMPLATE_SEED: u64 = super::properties::SEED + 0x5000;
pub const SWEEP_CASES: core::ops::Range<u64> = 0..3_750;
pub const WIDE_EXPRESSIONS: expressions::Shape =
    expressions::Shape { style: expressions::Style::Oracle, ..expressions::WIDE };
pub const WIDE_TEMPLATES: crate::templates::Shape =
    crate::templates::Shape { style: expressions::Style::Oracle, ..crate::templates::WIDE };
/// Every statement delimiter and output has a place in this input, before dressing.
#[must_use]
pub fn marker_tree() -> crate::templates::Tree {
    use crate::templates::Tree;
    Tree::Sequence(vec![
        Tree::Set { value: expressions::Tree::Int(2) },
        Tree::Branch {
            arms: vec![
                (expressions::Tree::Bool(true), Tree::Output(expressions::Tree::Name(b"x"))),
                (expressions::Tree::Bool(false), Tree::Text(b"elif".to_vec())),
            ],
            otherwise: Box::new(Tree::Text(b"else".to_vec())),
        },
        Tree::For {
            pair: false,
            iterable: expressions::Tree::List(vec![expressions::Tree::Int(1)]),
            body: Box::new(Tree::Output(expressions::Tree::Name(b"x"))),
        },
    ])
}

/// Draw each seeded input once and apply the independent engine's comparison.
pub fn compare<T, E: std::fmt::Display>(
    cases: core::ops::Range<u64>,
    seed: u64,
    engine: &T,
    check: impl Fn(&T, &str, &Roots) -> Result<(), E>,
    mut template: impl FnMut(&mut Rng, &Roots) -> String,
) {
    for case in cases {
        let (mut rng, roots) = draw(seed + case);
        check(engine, &template(&mut rng, &roots), &roots).unwrap_or_else(|mismatch| panic!("seed {case}: {mismatch}"));
    }
}
