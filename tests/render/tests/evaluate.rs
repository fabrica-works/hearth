use hearth_domain_render::limits;
use hearth_render_world::{expressions, reference};
use skein_lib::Rng;
#[test]
fn generated_expressions_evaluate_as_the_reference_evaluates_them() {
    for seed in 0..200_u64 {
        let mut rng = Rng::new(0x0014_0000 + seed);
        let roots = expressions::roots(&mut rng);
        let tree = expressions::tree(&mut rng, &roots, expressions::SMALL);
        assert_eq!(
            expressions::evaluate(&tree, &roots, &limits::SHIPPED),
            reference::evaluate(&tree, &roots, &limits::SHIPPED),
            "seed {seed}: {tree:?}"
        );
    }
}
