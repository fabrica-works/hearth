use hearth_protocol_template::template;
use hearth_template_world::{check, fixtures, generate, reference};
use skein_lib::Rng;
#[test]
fn generated_templates_compile_as_the_reference_compiles_them() {
    for seed in 0..200 {
        let text = generate::template::text(&mut Rng::new(seed), fixtures::SMALL_TEMPLATES);
        let compiled = template::compile(&text, &fixtures::LIMITS).expect("valid template");
        assert_eq!(Ok(&compiled), reference::template::compile(&text, &fixtures::LIMITS).as_ref(), "seed {seed}");
        check::template(&compiled, &text);
    }
}

#[test]
fn generated_strings_interpolate_as_the_reference_reads_them() {
    for seed in 0..200 {
        let text = generate::template::string(&mut Rng::new(seed), fixtures::SMALL);
        let compiled =
            hearth_protocol_template::interpolation::compile(&text, &fixtures::LIMITS).expect("valid decoded string");
        assert_eq!(Ok(&compiled), reference::template::interpolate(&text, &fixtures::LIMITS).as_ref(), "seed {seed}");
    }
}

// Keep the two debug compilers under the focused aim while retaining the boundary shapes.
const SCALED: hearth_protocol_template::limits::Limits =
    hearth_protocol_template::limits::Limits { template: 4096, ..hearth_protocol_template::limits::SHIPPED };
#[test]
fn templates_at_the_limits_compile_as_the_reference_does() {
    for (text, what, text_sized) in fixtures::templates_at_the_limits(&SCALED) {
        let limits = if text_sized { &SCALED } else { &fixtures::LIMITS };
        let got = template::compile(&text, limits);
        let want = reference::template::compile(&text, limits);
        assert_eq!(got.as_ref().map_err(|e| e.at), want.as_ref().map_err(|at| *at), "{what}");
        if let Ok(compiled) = got {
            check::template(&compiled, &text);
        }
    }
}
