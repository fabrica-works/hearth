use hearth_render_world::{declarations, expressions, fixtures, reference};
use skein_lib::Rng;
#[test]
fn generated_declarations_and_contexts_expand_as_the_reference_does() {
    for round in 0..5_000_u64 {
        let mut rng = Rng::new(0x0016_F000 + round);
        let limits = fixtures::limits(&mut rng);
        let roots = expressions::roots(&mut rng);
        let tree = declarations::tree(&mut rng, &roots);
        assert_eq!(
            declarations::expand(&tree, &roots, &limits),
            reference::expand(&tree, &roots, &limits),
            "round {round}: {tree:?}"
        );
    }
}
