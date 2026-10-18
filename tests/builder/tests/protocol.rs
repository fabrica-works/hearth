use hearth::Form;
use hearth_builder_world::{fixtures, protocol};

#[test]
fn render_prints_thin_whatever_the_cuts_of_every_pipe() {
    fixtures::each_protocol(&fixtures::thin(), |run, seed| {
        assert_eq!(run.stdout, fixtures::THIN_TEXT.as_bytes(), "seed {seed}: the real protocol and printer");
        assert_eq!(run.code, Some(0), "seed {seed}: done");
    });
}

#[test]
fn a_full_pipe_holds_its_writer_until_read() {
    let files = fixtures::many_blobs();
    let scenario = fixtures::Scenario { files: &files, ..fixtures::thin() };
    for seed in fixtures::WORLD_SEEDS {
        let run = protocol::with_pipe(&scenario, seed, fixtures::SMALL_PIPE);
        assert!(run.blocked_writes > 0, "seed {seed}: ls-tree waited on its full output pipe");
        assert_eq!(run.code, Some(0), "seed {seed}: reading let its writer finish");
        assert_eq!(
            String::from_utf8_lossy(&run.stdout).matches("/etc/blob-").count(),
            fixtures::MANY_BLOBS,
            "seed {seed}"
        );
    }
}

#[test]
fn a_tool_killed_mid_output_fails_the_call_with_75() {
    fixtures::each_protocol(
        &fixtures::faulty(fixtures::Fault::KilledAt { stdout_bytes: fixtures::KILLED_BYTES }),
        |run, seed| {
            assert!(run.fault_fell, "seed {seed}: the child's stdout was cut");
            assert_eq!(run.code, Some(75), "seed {seed}: operational failure");
        },
    );
}

#[test]
fn a_busy_lock_waits_and_says_so_once() {
    for seed in fixtures::WORLD_SEEDS {
        let run = protocol::with_lock(&fixtures::thin(), seed, fixtures::PROTOCOL_LOCK_RELEASE);
        assert!(run.now >= fixtures::PROTOCOL_LOCK_RELEASE, "seed {seed}: the contender held the lock");
        assert!(run.lock_times.len() > 1, "seed {seed}: lock acquisition retried");
        assert_eq!(run.stderr, b"waiting for the store's lock: /srv/store\n", "seed {seed}: busy reported once");
        assert_eq!(run.stdout, fixtures::THIN_TEXT.as_bytes(), "seed {seed}: render after the lock was released");
    }
}

#[test]
fn a_resolve_over_cut_pipes_records_its_closure() {
    fixtures::each_protocol(&fixtures::resolving(fixtures::THIN), |run, seed| {
        assert_eq!(run.code, Some(0), "seed {seed}: resolved");
        let records = fixtures::protocol_records(&run);
        assert_eq!(records.len(), 1, "seed {seed}: the closure was recorded");
        assert!(!fixtures::only(&records).closure.is_empty(), "seed {seed}: its selected packages are present");
        assert_eq!(
            fixtures::protocol_index_moves(&run),
            [
                b"snapshot.debian.org_archive_debian_20261001T030405Z_dists_forky_InRelease".as_slice(),
                b"snapshot.debian.org_archive_debian_20261001T030405Z_dists_forky_main_binary-amd64_Packages"
                    .as_slice(),
            ],
            "seed {seed}: each apt index renamed into cache"
        );
    });
}

const NOT_INSTALLED: &[(&str, fixtures::Fault, Option<u8>)] = &[
    ("absent", fixtures::Fault::NotInstalled { absent: true }, Some(75)),
    ("not executable", fixtures::Fault::NotInstalled { absent: false }, Some(75)),
];
#[test]
fn a_tool_not_installed_fails_the_call_with_75() {
    for (name, fault, code) in NOT_INSTALLED {
        fixtures::each_protocol(&fixtures::faulty(*fault), |run, seed| {
            assert!(run.fault_fell, "seed {seed}, {fault:?}: exec refused");
            assert_eq!(run.code, *code, "seed {seed}, {name}: could not start");
        });
    }
}

#[test]
fn a_settled_run_leaves_no_child_unreaped_pipe_unread_or_handle_open() {
    fixtures::each_protocol(&fixtures::thin(), |run, seed| {
        assert!(run.spawned > 0, "seed {seed}: real children ran");
        assert_eq!(run.spawned, run.reaped, "seed {seed}: every child reaped");
        assert_eq!(run.opened_pipes, run.ended_pipes, "seed {seed}: every stdout and stderr read to its end");
        assert!(run.shared.borrow().locks.is_empty(), "seed {seed}: dropping the world closed its root handles");
    });
}

#[test]
fn a_diff_whose_git_fails_prints_the_change_bare_then_its_tool_error_and_exits_75() {
    let scenario = fixtures::Scenario {
        fault: Some(fixtures::Fault::CacheFileGone { sha256: fixtures::MOTD_KIND_SHA256 }),
        ..fixtures::diff_motd(Form::JsonLines)
    };
    fixtures::each_protocol(&scenario, |run, seed| {
        assert!(run.fault_fell, "seed {seed}: the cached side vanished");
        assert_eq!(
            fixtures::diff_record_kinds(&run.stdout),
            ["version", "change", "change", "error", "outcome"],
            "seed {seed}: changes before the tool error"
        );
        let lines = fixtures::protocol_lines(&run.stdout);
        assert!(!lines.iter().any(|line| line.contains("\"hunks\"")), "seed {seed}: changes printed bare");
        assert!(
            lines.iter().any(|line| line.starts_with(r#"{"record":"error","side":"to","kind":"tool""#)),
            "seed {seed}: operational tool error"
        );
        assert_eq!(run.code, Some(75), "seed {seed}: failed Hunks");
    });
}
