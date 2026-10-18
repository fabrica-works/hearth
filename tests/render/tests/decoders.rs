use hearth_render_world::fixtures::decoders as inputs;
struct Row {
    decoder: &'static str,
    world: &'static str,
    file: &'static str,
    corpus: Option<&'static str>,
    session: &'static str,
}
const ROWS: &[Row] = &[
    Row { decoder: "JSON documents", world: "config", file: "fuzzy_documents.rs", corpus: None, session: "03.2" },
    Row { decoder: "declarations", world: "config", file: "fuzzy_declarations.rs", corpus: None, session: "35.2" },
    Row {
        decoder: "golden JSON",
        world: "config",
        file: "fuzzy_goldens.rs",
        corpus: Some("golden::"),
        session: "30.6",
    },
    Row { decoder: "expressions", world: "template", file: "fuzzy_expressions.rs", corpus: None, session: "05, 37" },
    Row {
        decoder: "templates and interpolated strings",
        world: "template",
        file: "fuzzy_templates.rs",
        corpus: None,
        session: "06, 36",
    },
    Row {
        decoder: "golden templates",
        world: "template",
        file: "fuzzy_goldens.rs",
        corpus: Some("golden::"),
        session: "30.6",
    },
    Row {
        decoder: "ls-tree, cat-file and hunks",
        world: "git",
        file: "fuzzy_git.rs",
        corpus: Some("recordings::"),
        session: "02, 34",
    },
    Row { decoder: "rev-parse and revision", world: "git", file: "fuzzy_revisions.rs", corpus: None, session: "30.6" },
    Row {
        decoder: "Inst, Packages and tails",
        world: "mmdebstrap",
        file: "fuzzy_mmdebstrap.rs",
        corpus: Some("recordings::"),
        session: "07, 38",
    },
    Row {
        decoder: "store, cache and resolution frames (kinds 1, 4, 5)",
        world: "tree",
        file: "fuzzy_tree.rs",
        corpus: None,
        session: "39",
    },
    Row { decoder: "store names", world: "builder", file: "fuzzy_inputs.rs", corpus: None, session: "30.6" },
    Row {
        decoder: "argument form and invocation",
        world: "builder",
        file: "fuzzy_inputs.rs",
        corpus: None,
        session: "30.6",
    },
];
const DOMAIN_REFERENCES: &[(&str, &str)] = &[("builder", "fuzzy_protocol.rs"), ("builder", "fuzzy_simulation.rs")];
#[test]
fn every_decoder_has_a_seeded_sweep_that_exists() {
    for row in ROWS {
        let source = inputs::source(row.world, row.file);
        assert!(
            row.file.starts_with("fuzzy_") && source.contains("#[test]") && !row.session.is_empty(),
            "{}: {} has no seeded sweep",
            row.decoder,
            row.file
        );
    }
}
#[test]
fn every_fuzzy_file_is_a_row_or_a_domain_reference() {
    let files = inputs::fuzzy_files();
    assert!(!files.is_empty(), "the worlds have seeded sweeps");
    for (world, file) in files {
        assert!(
            world == "render"
                || ROWS.iter().any(|row| (row.world, row.file) == (world.as_str(), file.as_str()))
                || DOMAIN_REFERENCES.contains(&(world.as_str(), file.as_str())),
            "{world}/{file} is absent from the inventory"
        );
    }
}
#[test]
fn every_machine_crate_has_a_world_with_a_sweep() {
    // The records crate only writes records; it decodes no input (hearth 10).
    let crates = inputs::machine_crates();
    assert!(!crates.is_empty(), "the workspace contains machine crates");
    for name in crates {
        assert!(ROWS.iter().any(|row| row.world == name), "hearth-protocol-{name} has no decoder world");
    }
}
#[test]
fn every_sweep_names_the_corpus_its_row_says() {
    for row in ROWS {
        if let Some(corpus) = row.corpus {
            assert!(
                inputs::source(row.world, row.file).contains(corpus),
                "{}: {} does not name {corpus}",
                row.decoder,
                row.file
            );
        }
    }
}
