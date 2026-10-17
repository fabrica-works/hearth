use hearth_config_world::{
    examples,
    world::{self, Settings},
};
use hearth_protocol_config::{
    Position,
    declaration::{Fragment, Script},
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
