use hearth_git_world::{
    expect::{self},
    fixtures,
    world::{self, Settings},
};
use hearth_protocol_git::{
    invocation::{self, Command},
    oid::Oid,
};
const SEEDS: &[u64] = &[0, 1, 2, 3];
#[test]
fn every_recording_decodes_to_its_expectation_whatever_the_cuts() {
    let recordings = hearth_recordings::all("git");
    assert!(recordings.len() >= 18, "at least eighteen cases");
    for (case, recording) in recordings {
        let expected = expect::decode(recording.expect.as_ref().expect("recorded expectation"));
        for seed in SEEDS {
            assert_eq!(
                world::replay(&recording, &expected, &fixtures::RECORDED, &Settings::cut(*seed)),
                Some(expected.clone()),
                "{case}, seed {seed}"
            );
        }
    }
}
#[test]
fn every_recording_ran_as_hearth_runs_git() {
    for (case, recording) in hearth_recordings::all("git") {
        assert_eq!(recording.argv[0], invocation::GIT, "{case}");
        assert_eq!(recording.argv[1..], arguments(&recording), "{case}");
        assert_eq!(recording.env, invocation::environment().iter().map(|b| b.to_vec()).collect::<Vec<_>>(), "{case}");
        assert_eq!(recording.dir, b"/scratch/store", "{case}");
    }
}
fn arguments(recording: &hearth_recordings::Recording) -> Vec<Vec<u8>> {
    let repository = recording.argv[1].strip_prefix(b"--git-dir=").expect("repository option");
    let command = recording.argv[2].as_slice();
    let revision = invocation::revision(
        recording.argv.last().expect("last argument").strip_suffix(b"^{commit}").unwrap_or(b"main"),
    )
    .expect("recorded revision");
    let commit = Oid::parse(recording.argv.last().expect("commit"))
        .unwrap_or_else(|| Oid::parse(b"0000000000000000000000000000000000000000").expect("literal"));
    let command = match command {
        b"rev-parse" => Command::RevParse { repository, revision: &revision },
        b"ls-tree" => Command::LsTree { repository, commit },
        b"cat-file" => Command::CatFile { repository },
        b"diff" => Command::Diff { store: b".", before: &[0x11; 32], after: &[0x22; 32] },
        other => panic!("command {other:?}"),
    };
    invocation::arguments(command).iter().map(|b| b.to_vec()).collect()
}
const EARLY_CUTS: &[u64] = &[0, 1];
const LATE_CUTS: &[u64] = &[2, 3];
#[test]
fn cut_and_raised_recordings_fail_or_decode_as_recorded() {
    fixtures::check_recording_mutations(EARLY_CUTS);
}
#[test]
fn cut_and_raised_recordings_fail_or_decode_as_recorded_at_other_cuts() {
    fixtures::check_recording_mutations(LATE_CUTS);
}
#[test]
fn every_recording_directory_is_a_case() {
    for (case, _) in hearth_recordings::all("git") {
        assert!(expect::CASE_NAMES.contains(&case.as_ref()), "unknown case {case}");
    }
}
