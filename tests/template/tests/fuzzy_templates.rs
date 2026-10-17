use hearth_protocol_template::template;
use hearth_template_world::{check, fixtures, generate, reference};
use skein_lib::Rng;
use std::collections::BTreeSet;
#[test]
fn cut_and_mutated_templates_compile_as_the_reference_compiles_them() {
    let mut fell = BTreeSet::new();
    for round in 0..5_000_u64 {
        let mut rng = Rng::new(0x0006_F000 + round);
        let (limits, valid) =
            (fixtures::limits(&mut rng), generate::template::text(&mut rng, fixtures::WIDE_TEMPLATES));
        let text = match rng.below(4) {
            0 => valid,
            1 => generate::cut(&mut rng, &valid),
            _ => generate::template::mutate(&mut rng, &valid),
        };
        let (got, want) = (template::compile(&text, &limits), reference::template::compile(&text, &limits));
        assert_eq!(got.as_ref().map_err(|e| e.at), want.as_ref().map_err(|at| *at), "round {round}: {text:?}");
        match &got {
            Ok(e) => check::template(e, &text),
            Err(e) => {
                fell.insert(fixtures::kind(&e.kind));
            }
        }
    }
    fixtures::assert_fell(&fell, fixtures::TEMPLATE_KINDS);
}

#[test]
fn cut_and_mutated_strings_interpolate_as_the_reference_reads_them() {
    let mut fell = BTreeSet::new();
    for round in 0..2_000_u64 {
        let mut rng = Rng::new(0x0036_F000 + round);
        let (limits, valid) = (fixtures::limits(&mut rng), generate::template::string(&mut rng, fixtures::WIDE));
        let text = match rng.below(4) {
            0 => valid,
            1 => generate::cut(&mut rng, &valid),
            _ => generate::template::mutate(&mut rng, &valid),
        };
        let got = hearth_protocol_template::interpolation::compile(&text, &limits);
        let want = reference::template::interpolate(&text, &limits);
        assert_eq!(got.as_ref().map_err(|e| e.at), want.as_ref().map_err(|at| *at), "round {round}: {text:?}");
        if let Err(error) = got {
            fell.insert(fixtures::kind(&error.kind));
        }
    }
    fixtures::assert_fell(&fell, fixtures::STRING_KINDS);
}
