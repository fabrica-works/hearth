use hearth_builder_world::fixtures;
use hearth_render_world::fixtures::{self as render, properties as inputs};
#[test]
fn a_generated_repository_renders_the_worlds_digests_through_real_git() {
    for case in inputs::PROCESS_CASES {
        let generated = inputs::generated(case);
        let digests = render::digests(&inputs::run(&generated, case));
        let digests: Vec<_> =
            digests.into_iter().map(|(name, digest)| (name, fixtures::hex_digest(digest.0))).collect();
        assert_eq!(fixtures::properties::printed(&generated), digests, "case {case}");
    }
}

#[test]
fn a_production_edit_leaves_the_lab_specs_through_real_git() {
    for case in inputs::PROCESS_CASES {
        let generated = inputs::generated(case);
        let before = inputs::environment(&fixtures::properties::printed(&generated), inputs::LAB);
        for edit in inputs::production_edits() {
            let edited = inputs::edited(&generated, &edit, case);
            assert_eq!(
                inputs::environment(&fixtures::properties::printed(&edited), inputs::LAB),
                before,
                "case {case}: {edit:?}"
            );
        }
    }
}
