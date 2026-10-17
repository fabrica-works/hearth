use hearth_protocol_template::expression;
use hearth_template_world::{check, fixtures, generate, reference};
use skein_lib::Rng;
#[test]
fn generated_expressions_compile_as_the_reference_compiles_them() {
    for seed in 0..200 {
        let text = generate::text(&mut Rng::new(seed), fixtures::SMALL);
        let got = expression::compile(&text, &fixtures::LIMITS).expect("valid generated text");
        let want = reference::compile(&text, &fixtures::LIMITS).expect("valid reference text");
        assert_eq!(got, want, "seed {seed}");
        check::invariants(&got, &text);
    }
}
