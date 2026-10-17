use hearth_domain_render::limits;
use hearth_render_world::{declarations, expressions, reference};
use skein_lib::Rng;
#[test]
fn generated_declarations_expand_as_the_reference_expands_them() {
    for seed in 0..200_u64 {
        let mut rng = Rng::new(0x0016_0000 + seed);
        let roots = expressions::roots(&mut rng);
        let tree = declarations::tree(&mut rng, &roots);
        assert_eq!(
            declarations::expand(&tree, &roots, &limits::SHIPPED),
            reference::expand(&tree, &roots, &limits::SHIPPED),
            "seed {seed}: {tree:?}"
        );
    }
}
