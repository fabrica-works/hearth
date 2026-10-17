use super::fixtures;
use crate::process::{Faults, Mmdebstrap, Progress};
use crate::{face, resolve::Variant};
use hearth_fake_machine::linux::{Exit, Op, Reply, Signal, Target};

const PROXIES: &[Option<u16>] = &[None, Some(3142)];
#[test]
fn hearth_s_vector_reads_into_its_simulate_request() {
    for proxy in PROXIES {
        let simulate = face::check(&fixtures::args(*proxy), &fixtures::env());
        assert_eq!(simulate.variant, Variant::Apt);
        assert_eq!(simulate.architecture.as_ref(), b"amd64");
        assert_eq!(simulate.include.as_ref(), &[]);
        assert_eq!(simulate.proxy, *proxy);
        assert_eq!(simulate.suite.as_ref(), b"forky");
        assert_eq!(simulate.scratch.as_ref(), b"/scratch");
    }
}

const INCLUDES: &[&[&[u8]]] = &[&[b"--include=git,mmdebstrap"], &[b"--include=git", b"--include=mmdebstrap"]];
#[test]
fn several_includes_keep_their_names_in_order() {
    for includes in INCLUDES {
        let mut args = fixtures::args(None).into_vec();
        args.splice(5..5, includes.iter().copied().map(Box::from));
        let simulate = face::check(&args, &fixtures::env());
        assert_eq!(simulate.include.iter().map(AsRef::as_ref).collect::<Vec<_>>(), [b"git".as_slice(), b"mmdebstrap"]);
    }
}

#[test]
#[should_panic(expected = "APT::Install-Recommends must be false")]
fn recommends_outside_the_fake_s_model_asserts_naming_the_option() {
    let mut args = fixtures::args(None).into_vec();
    args[6] = b"--aptopt=APT::Install-Recommends \"true\"".as_slice().into();
    let _ = face::check(&args, &fixtures::env());
}

const LISTS: &[(&[u8], &[u8])] = &[
    (b"InRelease", b"snapshot.debian.org_archive_debian_20261001T030405Z_dists_forky_InRelease"),
    (
        b"main/binary-amd64/Packages",
        b"snapshot.debian.org_archive_debian_20261001T030405Z_dists_forky_main_binary-amd64_Packages",
    ),
];
#[test]
fn source_relative_paths_take_apt_s_list_names() {
    let sources = face::sources(fixtures::SOURCES);
    let source = sources.first().expect("one fixture source");
    for (path, expected) in LISTS {
        assert_eq!(face::list_name(source, path).as_ref(), *expected);
    }
}

#[test]
fn a_spawned_mmdebstrap_leaves_the_lists_and_prints_each_inst_line() {
    let run = fixtures::run(&[]);
    assert!(fixtures::lines(&run.printed).any(|line| line == b"Inst mawk (1.3.4.20260302-2 Debian:testing [amd64])"));
    assert_eq!(run.machine.linux().contents(fixtures::PACKAGES), Some(fixtures::DEBIAN));
}

#[test]
fn an_unreachable_proxy_fails_with_no_index_in_the_lists() {
    let run = fixtures::run_on(fixtures::on_machine(Some(fixtures::PORT)), &[], Some(9), fixtures::SOURCES);
    assert_eq!(run.machine.linux().contents(fixtures::PACKAGES), None);
    assert!(fixtures::lines(&run.printed).any(|line| line == b"E: Failed to fetch http://snapshot.debian.org/archive/debian/20261001T030405Z/dists/forky/InRelease  Unable to connect to 127.0.0.1:9:"));
}

#[test]
fn an_absent_snapshot_s_inrelease_is_not_found_and_nothing_resolves() {
    let run = fixtures::run_on(
        fixtures::on_machine(Some(fixtures::PORT)),
        &[],
        Some(fixtures::PORT),
        b"deb http://snapshot.debian.org/archive/debian/20000101T000000Z/ forky main\n",
    );
    assert!(!fixtures::lines(&run.printed).any(|line| line.starts_with(b"Inst ")));
    assert!(fixtures::lines(&run.printed).any(|line| line == b"E: The repository 'http://snapshot.debian.org/archive/debian/20000101T000000Z forky Release' does not have a Release file."));
}

#[test]
fn an_unknown_package_fails_after_the_lists_are_written() {
    let run = fixtures::run(&[b"hearth-absent"]);
    assert_eq!(run.machine.linux().contents(fixtures::PACKAGES), Some(fixtures::DEBIAN));
    assert!(fixtures::lines(&run.printed).any(|line| line == b"E: Unable to locate package hearth-absent"));
    assert!(fixtures::lines(&run.printed).any(|line| line == b"Inst mawk (1.3.4.20260302-2 Debian:testing [amd64])"));
}

#[test]
fn a_virtual_request_prints_its_providers_and_no_candidate() {
    let run = fixtures::run(&[b"mail-transport-agent"]);
    assert!(run.printed.windows(b"Package mail-transport-agent is a virtual package provided by:\n  postfix 3.10.4-1\n  exim4-daemon-light 4.98.2-1\n\nE: Package 'mail-transport-agent' has no installation candidate\n".len()).any(|part| part == b"Package mail-transport-agent is a virtual package provided by:\n  postfix 3.10.4-1\n  exim4-daemon-light 4.98.2-1\n\nE: Package 'mail-transport-agent' has no installation candidate\n"));
}

#[test]
fn a_conflict_prints_the_unmet_dependencies_block() {
    let run = fixtures::run(&[b"exim4-daemon-light", b"postfix"]);
    assert!(run.printed.windows(b"The following packages have unmet dependencies:\n exim4-daemon-light : Conflicts: mail-transport-agent\n postfix : Conflicts: mail-transport-agent\nE: Unable to correct problems, you have held broken packages.\n".len()).any(|part| part == b"The following packages have unmet dependencies:\n exim4-daemon-light : Conflicts: mail-transport-agent\n postfix : Conflicts: mail-transport-agent\nE: Unable to correct problems, you have held broken packages.\n"));
}

const EXITS: &[(&[&[u8]], u8)] = &[
    (&[], 0),
    (&[b"hearth-absent"], 25),
    (&[b"mail-transport-agent"], 25),
    (&[b"exim4-daemon-light", b"postfix"], 25),
];
#[test]
fn a_resolved_run_exits_zero_and_an_apt_failure_exits_twenty_five() {
    for (names, exit) in EXITS {
        let mut run = fixtures::run(names);
        assert_eq!(
            run.machine.call(run.parent, Op::Wait { child: run.child }),
            Ok(Reply::Exited(Exit::Code(*exit))),
            "{names:?}"
        );
    }
}

#[test]
fn a_stalled_mmdebstrap_sends_nothing_more_until_terminated() {
    let mut machine = fixtures::on_machine(Some(fixtures::PORT));
    let parent = fixtures::builder(&mut machine);
    let (child, [stdout, stderr]) =
        fixtures::spawn_mmdebstrap(&mut machine, parent, &fixtures::args(Some(fixtures::PORT)));
    let mut process = Mmdebstrap::start(&mut machine, child, Faults { stall_after: Some(7) });
    while process.step(&mut machine) == Progress::Working {}
    assert_eq!(fixtures::drain_both(&mut machine, parent, stdout, stderr), b"I: skip");
    assert_eq!(process.step(&mut machine), Progress::Waiting);
    assert_eq!(fixtures::drain_both(&mut machine, parent, stdout, stderr), b"");
    assert_eq!(
        machine.call(parent, Op::Signal { child, signal: Signal::Terminate, to: Target::Child }),
        Ok(Reply::Done)
    );
    assert_eq!(process.step(&mut machine), Progress::Exited);
    assert_eq!(machine.call(parent, Op::Wait { child }), Ok(Reply::Exited(Exit::Signal(15))));
}
