#[path = "../../render/src/fixtures/decoders/golden.rs"]
mod golden;
use hearth_protocol_template::{limits::SHIPPED, template};
use hearth_template_world::{check, generate, reference};
use skein_lib::Rng;
#[test]
fn golden_templates_cut_and_mutated_compile_as_the_reference_compiles_them() {
    let corpus = golden::corpus("j2");
    for seed in golden::CASES {
        let mut rng = Rng::new(golden::SEED + seed);
        for (path, input) in &corpus {
            for text in [input.clone(), generate::cut(&mut rng, input), generate::template::mutate(&mut rng, input)] {
                let compiled = template::compile(&text, &SHIPPED);
                assert_eq!(
                    compiled.as_ref().map_err(|error| error.at),
                    reference::template::compile(&text, &SHIPPED).as_ref().map_err(|at| *at),
                    "{}: seed {seed}",
                    path.display()
                );
                if let Ok(compiled) = compiled {
                    check::template(&compiled, &text);
                }
            }
        }
    }
}
