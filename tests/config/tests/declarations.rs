use hearth_config_world::{
    examples,
    world::{self, Settings},
};
use hearth_protocol_config::{
    Position,
    declaration::{Fragment, Script},
    entries::Map,
    limits::SHIPPED,
};
use skein_lib::Rng;

type ReadShape = (&'static [u8], Position, Position, usize, bool, bool);
const READS: &[ReadShape] = &[
    (
        b"core",
        Position { offset: 19, line: 3, column: 5 },
        Position { offset: 27, line: 3, column: 13 },
        1,
        false,
        false,
    ),
    (
        b"mail",
        Position { offset: 115, line: 5, column: 5 },
        Position { offset: 123, line: 5, column: 13 },
        0,
        true,
        false,
    ),
    (
        b"wifi",
        Position { offset: 59, line: 4, column: 5 },
        Position { offset: 67, line: 4, column: 13 },
        1,
        false,
        true,
    ),
];
#[test]
fn the_reads_example_decodes_to_its_three_reads_cut_anywhere() {
    for seed in 0..8 {
        let settings = Settings::calm(&mut Rng::new(seed), SHIPPED);
        let declaration = world::declare(examples::task("reads").bytes, &settings, seed).expect("format.md, 6.4");
        assert_eq!(declaration.reads.len(), 3);
        for (read, expected) in declaration.reads.iter().zip(READS) {
            assert_eq!(
                (&*read.task.text, read.task.at, read.at, read.settings.len(), read.exports, read.optional),
                *expected,
                "seed {seed}"
            );
        }
    }
}
const READ_SETTINGS: &[(&[u8], &[u8], Position)] = &[
    (b"core", b"domain", Position { offset: 42, line: 3, column: 28 }),
    (b"wifi", b"ssid", Position { offset: 82, line: 4, column: 28 }),
];
#[test]
fn the_reads_example_keeps_each_setting_at_its_string() {
    for seed in 0..8 {
        let settings = Settings::calm(&mut Rng::new(seed), SHIPPED);
        let declaration = world::declare(examples::task("reads").bytes, &settings, seed).expect("format.md, 6.4");
        let observed: Vec<_> = declaration
            .reads
            .iter()
            .flat_map(|read| read.settings.iter().map(|setting| (&*read.task.text, &*setting.text, setting.at)))
            .collect();
        assert_eq!(observed, READ_SETTINGS, "seed {seed}");
    }
}
const SCRIPTS: &[Script] = &[
    Script { fragment: Fragment::Customize, at: Position { offset: 15, line: 1, column: 16 }, order: 300 },
    Script { fragment: Fragment::Software, at: Position { offset: 49, line: 1, column: 50 }, order: 500 },
];
#[test]
fn the_scripts_example_decodes_cut_anywhere() {
    for seed in 0..8 {
        let settings = Settings::calm(&mut Rng::new(seed), SHIPPED);
        let declaration = world::declare(examples::task("scripts").bytes, &settings, seed).expect("format.md, 5.8");
        assert_eq!(&*declaration.scripts, SCRIPTS, "seed {seed}");
    }
}
#[test]
fn the_bless_example_decodes_cut_anywhere() {
    for seed in 0..8 {
        let settings = Settings::calm(&mut Rng::new(seed), SHIPPED);
        let declaration = world::declare(examples::task("bless").bytes, &settings, seed).expect("format.md, 5.13");
        let bless = declaration.bless.expect("bless");
        let [unit] = &bless.ignore_failed[..] else { panic!("one unit") };
        assert_eq!(bless.at, Position { offset: 11, line: 1, column: 12 });
        assert_eq!((&*unit.text, unit.at), (b"foo.service".as_slice(), Position { offset: 31, line: 1, column: 32 }));
    }
}

const ENTRY_EXAMPLES: &[(&str, usize, Map, usize)] = &[
    ("users-accounts", 1, Map::Users, 1),
    ("ssh", 1, Map::Files, 2),
    ("mail-accounts", 2, Map::Groups, 1),
    ("mail-accounts", 2, Map::Users, 1),
    ("core-journal", 2, Map::Data, 1),
    ("core-journal", 2, Map::Groups, 1),
    ("wifi", 2, Map::Data, 1),
    ("wifi", 2, Map::Packages, 1),
    ("mail-data", 2, Map::Data, 2),
    ("mail-data", 2, Map::Groups, 1),
    ("firmware", 1, Map::Packages, 4),
    ("core-gettys", 1, Map::Files, 1),
    ("mail-live", 1, Map::Live, 1),
    ("stoa", 4, Map::Files, 2),
    ("stoa", 4, Map::Live, 1),
    ("stoa", 4, Map::Presets, 1),
    ("stoa", 4, Map::Software, 1),
    ("ripgrep", 1, Map::Live, 1),
    ("dkim", 1, Map::Secrets, 1),
    ("ssh-identity", 1, Map::Identity, 1),
    ("software", 1, Map::Software, 2),
];
#[test]
fn every_entry_example_decodes_to_its_entries_cut_anywhere() {
    for &(name, maps, map, entries) in ENTRY_EXAMPLES {
        for seed in 0..8 {
            let settings = Settings::calm(&mut Rng::new(seed), SHIPPED);
            let declaration =
                world::declare(examples::task(name).bytes, &settings, seed).expect("format.md entry example");
            assert_eq!(declaration.entries.len(), maps, "{name}, seed {seed}");
            let actual = declaration.entries.iter().find(|actual| actual.map == map).expect("expected map");
            assert_eq!(actual.entries.len(), entries, "{name}, {map:?}, seed {seed}");
        }
    }
}
const LATER_EXAMPLES: &[(&str, Position)] = &[
    ("checks-ssh", Position { offset: 2, line: 1, column: 3 }),
    ("artifact-stoa", Position { offset: 38, line: 4, column: 7 }),
];
#[test]
fn the_m5_and_m6_examples_are_refused_at_their_keys() {
    for &(name, at) in LATER_EXAMPLES {
        for seed in 0..8 {
            let settings = Settings::calm(&mut Rng::new(seed), SHIPPED);
            assert_eq!(
                world::declare(examples::task(name).bytes, &settings, seed),
                Err(hearth_protocol_config::Error {
                    at,
                    fault: hearth_protocol_config::Fault::Declaration(hearth_protocol_config::Declared::Later)
                }),
                "{name}, seed {seed}"
            );
        }
    }
}

const SCHEMAS: &[(&str, &[&[u8]], usize)] = &[
    ("boot", &[b"kernel", b"modules", b"params"], 8),
    ("hearth", &[b"alarms", b"backups", b"snapshots"], 14),
    ("locale", &[b"locale", b"timezone"], 4),
    ("users", &[b"accounts"], 7),
];
#[test]
fn every_schema_example_decodes_to_its_settings_cut_anywhere() {
    for &(name, names, types) in SCHEMAS {
        for seed in 0..8 {
            let settings = Settings::calm(&mut Rng::new(seed), SHIPPED);
            let declaration =
                world::declare(examples::task(name).bytes, &settings, seed).expect("format.md schema example");
            let actual: Vec<_> = declaration.settings.iter().map(|spec| &*spec.name.text).collect();
            assert_eq!(actual, names, "{name}, seed {seed}");
            assert_eq!(declaration.types.len(), types, "{name}, seed {seed}");
        }
    }
}
