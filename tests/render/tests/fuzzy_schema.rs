use hearth_render_world::{fixtures, schemas};
use skein_lib::Rng;
use std::collections::BTreeSet;
#[test]
fn generated_schemas_and_settings_validate_as_the_reference_does() {
    let mut fell = BTreeSet::new();
    for round in 0..10_000_u64 {
        let mut rng = Rng::new(0x0013_F000 + round);
        let declaration = schemas::declaration(&mut rng, schemas::WIDE);
        let settings = schemas::settings(&mut rng, &declaration);
        let value = schemas::validate(&declaration, &settings);
        assert_eq!(value, schemas::reference(&declaration, &settings), "round {round}");
        if let Err(faults) = value {
            for fault in &faults {
                fell.insert(schemas::kind(&fault.schema));
            }
        }
    }
    fixtures::assert_fell(&fell, schemas::KINDS);
}
