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
