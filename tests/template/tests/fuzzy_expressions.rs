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

#[test]
fn texts_at_the_limits_cut_and_mutated_compile_as_the_reference_does() {
    let cases = fixtures::at_the_limits();
    for round in 0..500_u64 {
        let mut rng = Rng::new(0x0037_F000 + round);
        let index = usize::try_from(rng.below(cases.len() as u64)).expect("fixture index");
        let (valid, what, _) = &cases[index];
        let text = if rng.below(2) == 0 { generate::cut(&mut rng, valid) } else { generate::mutate(&mut rng, valid) };
        let got = expression::compile(&text, &fixtures::LIMITS);
        let want = reference::compile(&text, &fixtures::LIMITS);
        assert_eq!(
            got.as_ref().map_err(|error| error.at),
            want.as_ref().map_err(|at| *at),
            "round {round}: {what}: {text:?}"
        );
        if let Ok(expression) = got {
            check::invariants(&expression, &text);
        }
    }
}
