use hearth_render_world::{expressions, fixtures, reference};
use skein_lib::Rng;
use std::collections::BTreeSet;
#[test]
fn generated_expressions_and_contexts_evaluate_as_the_reference_does() {
    let mut fell = BTreeSet::new();
    for round in 0..10_000_u64 {
        let mut rng = Rng::new(0x0014_F000 + round);
        let limits = fixtures::limits(&mut rng);
        let roots = expressions::roots(&mut rng);
        let tree = expressions::tree(&mut rng, &roots, expressions::WIDE);
        let got = expressions::evaluate(&tree, &roots, &limits);
        assert_eq!(got, reference::evaluate(&tree, &roots, &limits), "round {round}: {tree:?}");
        if let Err(kind) = got {
            fell.insert(kind);
        }
    }
    fixtures::assert_fell(&fell, expressions::KINDS);
}
