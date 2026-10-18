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
