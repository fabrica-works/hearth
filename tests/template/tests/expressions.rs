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

#[test]
fn texts_at_the_limits_compile_as_the_reference_does() {
    for (text, what, outcome) in fixtures::at_the_limits() {
        let got = expression::compile(&text, &fixtures::LIMITS);
        let want = reference::compile(&text, &fixtures::LIMITS);
        assert_eq!(got.as_ref().map(|_| ()).map_err(|error| error.at), outcome, "{what}");
        assert_eq!(got.as_ref().map_err(|error| error.at), want.as_ref().map_err(|at| *at), "{what}");
    }
}

#[test]
fn a_filter_after_a_test_is_refused_at_its_pipe() {
    const CASES: &[(&[u8], u32)] = &[(b"not (\t5892802747607251180  is defined | replace('a', 'b') | length())", 38)];
    for (text, at) in CASES {
        assert_eq!(expression::compile(text, &fixtures::LIMITS).map_err(|error| error.at), Err(*at));
        assert_eq!(reference::compile(text, &fixtures::LIMITS), Err(*at));
    }
}

#[test]
fn a_second_comparison_is_refused_before_reading_its_in() {
    const CASES: &[(&[u8], u32)] = &[(b"not ((not (name_1\n.and  < s not defined)))", 28)];
    for (text, at) in CASES {
        assert_eq!(expression::compile(text, &fixtures::LIMITS).map_err(|error| error.at), Err(*at));
        assert_eq!(reference::compile(text, &fixtures::LIMITS), Err(*at));
    }
}
