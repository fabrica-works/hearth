use hearth_git_world::{
    fixtures, generate,
    world::{self, DiffOutcome, Settings},
};
use skein_lib::Rng;
#[test]
fn generated_hunks_decode_to_their_lines_whatever_the_cuts() {
    for (input, expected) in fixtures::HUNKS {
        for seed in 0..16 {
            let stdout = generate::hunks(&mut Rng::new(seed), input);
            let DiffOutcome::Done(hunks) = world::diff(&stdout, &fixtures::SMALL, &Settings::cut(seed)) else {
                panic!("seed {seed}: valid hunks")
            };
            let lines: Vec<_> = hunks.lines.iter().map(|line| (line.kind, line.text.as_ref())).collect();
            assert_eq!(lines.as_slice(), *expected, "seed {seed}");
        }
    }
}
const EXIT_CASES: &[(hearth_protocol_git::invocation::Exit, bool)] = &[
    (hearth_protocol_git::invocation::Exit::Code(0), false),
    (hearth_protocol_git::invocation::Exit::Code(1), true),
    (hearth_protocol_git::invocation::Exit::Code(2), false),
    (hearth_protocol_git::invocation::Exit::Signal(9), false),
];
#[test]
fn only_diff_exit_one_permits_its_hunks() {
    use hearth_protocol_git::invocation::{self, Command};
    let command = Command::Diff { store: b".", before: &[0x11; 32], after: &[0x22; 32] };
    for (exit, expected) in EXIT_CASES {
        assert_eq!(invocation::succeeded(command, *exit), *expected);
    }
}
