use hearth_builder_world::fixtures::inputs;
use hearth_protocol_builder::store_shape;
#[test]
fn store_names_mutated_by_a_byte_shape_as_the_table_says() {
    inputs::each(|seed, rng| {
        for row in inputs::NAMES {
            let name = inputs::name(row);
            assert_eq!(store_shape(row.directory, &name, row.kind), row.expected, "seed {seed}: {name:?}");
            let name = inputs::damaged(rng, &name);
            assert_eq!(store_shape(row.directory, &name, row.kind), row.damaged, "seed {seed}: {name:?}");
        }
    });
}
#[test]
fn mutated_argv_gives_the_startup_its_form_asks() {
    inputs::each(|seed, rng| {
        let input = inputs::invocation(rng);
        for row in inputs::MUTATIONS {
            let args = inputs::mutated(&input, row);
            assert_eq!(
                (hearth::argv::form(&args), inputs::parse(&args)),
                (row.form, Err(inputs::refusal(row))),
                "seed {seed}: {:?}",
                row.word
            );
        }
    });
}

#[test]
fn drawn_invocations_encode_and_back() {
    inputs::each(|seed, rng| {
        for (row, input) in inputs::invocations(rng).into_iter().enumerate() {
            assert_eq!(inputs::parse(&inputs::arguments(&input)), Ok(input), "seed {seed}, row {row}");
        }
    });
}
