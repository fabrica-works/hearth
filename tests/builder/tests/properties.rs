use hearth_builder_world::fixtures;
use hearth_render_world::fixtures::{self as render, properties as inputs};
#[test]
fn a_generated_repository_renders_the_worlds_digests_through_real_git() {
    for case in inputs::PROCESS_CASES {
        let generated = inputs::generated(case);
        let files = fixtures::files_of(&generated.leaves);
        let repository = fixtures::Repository::make("generated-property", &files);
        let output = fixtures::render(&repository, &[], hearth::Form::JsonLines);
        let digests = render::digests(&inputs::run(&generated, case));
        let digests: Vec<_> =
            digests.into_iter().map(|(name, digest)| (name, fixtures::hex_digest(digest.0))).collect();
        assert_eq!(
            fixtures::digests_of(&String::from_utf8(output.stdout).expect("JSON lines")),
            digests,
            "case {case}"
        );
    }
}
