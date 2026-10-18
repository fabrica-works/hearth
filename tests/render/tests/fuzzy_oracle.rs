#[path = "oracle/engine.rs"]
mod engine;
use engine::Oracle;
use hearth_render_world::{
    expressions::{self, Style},
    fixtures::{self, oracle as inputs},
    spacing, templates,
};
use std::collections::BTreeSet;
#[test]
fn generated_expressions_at_wide_shapes_render_as_the_oracle_renders_them() {
    let oracle = Oracle::new();
    oracle.check_differences().expect("listed differences keep their pinned outcomes");
    for seed in inputs::SWEEP_CASES {
        let (mut rng, roots) = fixtures::draw(inputs::TEMPLATE_SEED + seed);
        let tree = expressions::tree(&mut rng, &roots, inputs::WIDE_EXPRESSIONS);
        let template = templates::output_of(&expressions::text(&tree, Style::Oracle));
        oracle.compare(&template, &roots).unwrap_or_else(|mismatch| panic!("seed {seed}: {mismatch}"));
    }
}
#[test]
fn generated_templates_at_wide_shapes_render_as_the_oracle_renders_them() {
    let oracle = Oracle::new();
    oracle.check_differences().expect("listed differences keep their pinned outcomes");
    let mut drawn = BTreeSet::new();
    let mut dressed = BTreeSet::new();
    for seed in inputs::SWEEP_CASES {
        let (mut rng, roots) = fixtures::draw(inputs::TEMPLATE_SEED + seed);
        let tree = templates::tree(&mut rng, &roots, inputs::WIDE_TEMPLATES);
        drawn.extend(templates::constructs(&tree));
        let template = spacing::dressed(&mut rng, &tree);
        dressed.extend(spacing::constructs(&template));
        oracle.compare(&template, &roots).unwrap_or_else(|mismatch| panic!("seed {seed}: {mismatch}"));
    }
    fixtures::assert_fell(&drawn, templates::CONSTRUCTS);
    fixtures::assert_fell(&dressed, spacing::CONSTRUCTS);
}
