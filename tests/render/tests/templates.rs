use hearth_domain_render::limits;
use hearth_render_world::{expressions, reference, templates};
use skein_lib::Rng;
#[test]
fn generated_templates_render_as_the_reference_renders_them() {
    for seed in 0..200_u64 {
        let mut rng = Rng::new(0x0015_0000 + seed);
        let roots = expressions::roots(&mut rng);
        let tree = templates::tree(&mut rng, &roots, templates::SMALL);
        assert_eq!(
            templates::run(&tree, &roots, &limits::SHIPPED),
            reference::run(&tree, &roots, &limits::SHIPPED),
            "seed {seed}: {tree:?}"
        );
    }
}
