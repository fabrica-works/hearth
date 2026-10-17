use hearth_render_world::{expressions, fixtures, reference, templates};
use skein_lib::Rng;
#[test]
fn generated_templates_and_contexts_render_as_the_reference_does() {
    for round in 0..2_000_u64 {
        let mut rng = Rng::new(0x0015_F000 + round);
        let limits = fixtures::limits(&mut rng);
        let roots = expressions::roots(&mut rng);
        let tree = templates::tree(&mut rng, &roots, templates::WIDE);
        assert_eq!(
            templates::run(&tree, &roots, &limits),
            reference::run(&tree, &roots, &limits),
            "round {round}: {tree:?}"
        );
    }
}
