use hearth_protocol_template::expression;
use hearth_template_world::{check, fixtures, generate, reference};
use skein_lib::Rng;
use std::collections::BTreeSet;
#[test]
fn cut_and_mutated_texts_compile_as_the_reference_compiles_them() {
    let mut fell = BTreeSet::new();
    for round in 0..10_000_u64 {
        let mut rng = Rng::new(0x0005_F000 + round);
        let (limits, valid) = (fixtures::limits(&mut rng), generate::text(&mut rng, fixtures::WIDE));
        let text = match rng.below(4) {
            0 => valid,
            1 => generate::cut(&mut rng, &valid),
            _ => generate::mutate(&mut rng, &valid),
        };
        let (got, want) = (expression::compile(&text, &limits), reference::compile(&text, &limits));
        assert_eq!(got.as_ref().map_err(|e| e.at), want.as_ref().map_err(|at| *at), "round {round}: {text:?}");
        match &got {
            Ok(e) => check::invariants(e, &text),
            Err(e) => {
                fell.insert(fixtures::kind(&e.kind));
            }
        }
    }
    fixtures::assert_fell(&fell);
}
