use hearth_render_world::schemas;
use skein_lib::Rng;
#[test]
fn generated_settings_validate_as_the_reference_validates_them() {
    for seed in 0..200_u64 {
        let mut rng = Rng::new(0x0013_0000 + seed);
        let declaration = schemas::declaration(&mut rng, schemas::SMALL);
        let settings = schemas::settings(&mut rng, &declaration);
        assert_eq!(
            schemas::validate(&declaration, &settings),
            schemas::reference(&declaration, &settings),
            "seed {seed}"
        );
    }
}
