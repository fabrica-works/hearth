use super::fixtures;
use crate::{
    limits,
    output::{self, Inst, OutputError, Seen},
};
use skein_lib::{
    bytes,
    stream::{Delimiter, Down, Fault, Read, Up},
};
#[test]
fn an_inst_line_gives_its_name_version_and_architecture() {
    let (env, mut reader, mut above, mut below) = fixtures::opened();
    let line = bytes::copy_of(b"Inst libc6 (2.41-12 Debian:testing [amd64])\n");
    output::up(&mut reader, &env, Up::Bytes(line), &mut above, &mut below);
    let inst = Inst {
        name: fixtures::name(b"libc6"),
        version: bytes::copy_of(b"2.41-12"),
        architecture: fixtures::name(b"amd64"),
    };
    assert_eq!(above.pop(), Some(Seen::Inst(inst)));
    assert_eq!(below.pop(), Some(Down::Demand { read: Read::Scan { until: Delimiter::LF, max: 4_096 }, room: 0 }));
}
#[test]
fn an_inst_line_with_several_origins_gives_the_same_triple() {
    assert_eq!(
        output::inst(b"Inst libc6 (2.41-12 Debian:testing, Debian:unstable [amd64])", &limits::SHIPPED),
        output::inst(b"Inst libc6 (2.41-12 Debian:testing [amd64])", &limits::SHIPPED)
    );
}
#[test]
fn an_installed_version_in_brackets_is_dropped() {
    assert_eq!(
        output::inst(b"Inst libc6 [2.40-1] (2.41-12 Debian:testing [amd64])", &limits::SHIPPED),
        output::inst(b"Inst libc6 (2.41-12 Debian:testing [amd64])", &limits::SHIPPED)
    );
}
#[test]
fn an_inst_line_ignores_what_follows_its_closing_bracket() {
    let want = output::inst(b"Inst libc6 (2.41-12 Debian:testing [amd64])", &limits::SHIPPED);
    for line in [
        b"Inst libc6 (2.41-12 Debian:testing [amd64]) [libc6 :amd64]".as_slice(),
        b"Inst libc6 (2.41-12 Debian:testing [amd64]) []",
    ] {
        assert_eq!(output::inst(line, &limits::SHIPPED), want);
    }
}
const OTHER: &[&[u8]] = &[b"Conf libc6 (2.41-12 Debian:testing [amd64])\n", b"Remv libc6\n", b"I: apt-get update\n"];
#[test]
fn conf_remv_and_other_lines_give_no_inst() {
    for line in OTHER {
        let (env, mut reader, mut above, mut below) = fixtures::opened();
        output::up(&mut reader, &env, Up::Bytes(bytes::copy_of(line)), &mut above, &mut below);
        assert_eq!(above.pop(), None);
    }
}
const MALFORMED: &[&[u8]] = &[
    b"Inst bad\n",
    b"Inst a (1 Debian [amd64])\n",
    b"Inst libc6 ( Debian [amd64])\n",
    b"Inst libc6 (1  [amd64])\n",
    b"Inst libc6 (1 Debian [bad_])\n",
    b"Inst libc6 [x (1 Debian [amd64])\n",
];
#[test]
fn a_malformed_inst_line_fails_the_pipe_once_it_ends() {
    for line in MALFORMED {
        let (env, mut reader, mut above, mut below) = fixtures::opened();
        output::up(&mut reader, &env, Up::Bytes(bytes::copy_of(line)), &mut above, &mut below);
        assert_eq!(above.pop(), None);
        assert!(below.pop().is_some(), "malformed line keeps draining");
        output::up(
            &mut reader,
            &env,
            Up::Bytes(bytes::copy_of(b"Inst apt (1 Debian [amd64])\n")),
            &mut above,
            &mut below,
        );
        assert_eq!(above.pop(), None);
        drop(below.pop());
        output::up(&mut reader, &env, Up::End, &mut above, &mut below);
        assert_eq!(above.pop(), Some(Seen::Failed(OutputError::Inst)));
        assert!(!reader.waiting(), "terminal pipe does not wait");
    }
}
#[test]
fn inst_lines_past_the_limit_fail_the_pipe() {
    let (mut env, mut reader, mut above, mut below) = fixtures::opened();
    env.limits.insts = 2; // third line exceeds the limit
    for at in 0_u32..3 {
        output::up(
            &mut reader,
            &env,
            Up::Bytes(bytes::copy_of(b"Inst apt (1 Debian [amd64])\n")),
            &mut above,
            &mut below,
        );
        let seen = above.pop();
        assert_eq!(seen.is_some(), at < 2);
        drop(below.pop());
    }
    output::up(&mut reader, &env, Up::End, &mut above, &mut below);
    assert_eq!(above.pop(), Some(Seen::Failed(OutputError::TooMany)));
}
#[test]
fn a_line_past_its_bound_is_cut_and_never_read_as_two() {
    let (mut env, mut reader, mut above, mut below) = fixtures::opened();
    env.limits.line = 32; // the first piece of a long line is its only candidate
    output::up(
        &mut reader,
        &env,
        Up::Bytes(bytes::copy_of(b"Inst apt (1 Debian [amd64])      ")),
        &mut above,
        &mut below,
    );
    assert_eq!(
        above.pop(),
        Some(Seen::Inst(Inst {
            name: fixtures::name(b"apt"),
            version: bytes::copy_of(b"1"),
            architecture: fixtures::name(b"amd64")
        }))
    );
    drop(below.pop());
    output::up(&mut reader, &env, Up::Bytes(bytes::copy_of(b"Inst aa (2 Debian [amd64])\n")), &mut above, &mut below);
    assert_eq!(above.pop(), None);
}
const FETCHED: &[(&[u8], u64)] = &[
    (b"Fetched 836 B in 1s\n", 836),
    (b"Fetched 1234 kB in 1s\n", 1_234_000),
    (b"Fetched 12.3 MB in 1s\n", 12_300_000),
    (b"Fetched 0.9 B in 1s\n", 0),
    (b"Fetched 1 GB in 1s\n", 1_000_000_000),
    (b"Fetched 2 TB in 1s\n", 2_000_000_000_000),
];
#[test]
fn a_fetched_line_gives_its_bytes_in_apt_s_units() {
    for (line, want) in FETCHED {
        let (env, mut reader, mut above, mut below) = fixtures::opened();
        output::up(&mut reader, &env, Up::Bytes(bytes::copy_of(line)), &mut above, &mut below);
        assert_eq!(above.pop(), Some(Seen::Fetched(*want)));
    }
}
const NOT_FETCHED: &[&[u8]] = &[
    b"Fetched 1 MB\n",
    b"Fetched .1 MB in 1s\n",
    b"Fetched 1. MB in 1s\n",
    b"Fetched 1.2.3 MB in 1s\n",
    b"Fetched 1 KiB in 1s\n",
    b"Fetched 999999999999999999999999999 TB in 1s\n",
];
#[test]
fn a_line_that_is_not_a_fetched_line_gives_no_bytes() {
    for line in NOT_FETCHED {
        let (env, mut reader, mut above, mut below) = fixtures::opened();
        output::up(&mut reader, &env, Up::Bytes(bytes::copy_of(line)), &mut above, &mut below);
        assert_eq!(above.pop(), None);
    }
}
#[test]
fn a_broken_pipe_reports_its_stream_fault_once() {
    let (env, mut reader, mut above, mut below) = fixtures::opened();
    output::up(&mut reader, &env, Up::Failed(Fault::Reset), &mut above, &mut below);
    assert_eq!(above.pop(), Some(Seen::Failed(OutputError::Stream(Fault::Reset))));
    output::up(&mut reader, &env, Up::End, &mut above, &mut below);
    assert_eq!(above.pop(), None);
    assert_eq!(below.pop(), None);
}

use crate::tail::{self, Cause, Class, Kind, Tail};
fn selected(lines: &[&[u8]]) -> Tail {
    let mut tail = Tail::new(&limits::SHIPPED);
    for line in lines {
        tail::admit(&mut tail, line, &limits::SHIPPED);
    }
    tail
}
#[test]
fn a_failed_fetch_keeps_its_err_and_e_lines_and_is_operational() {
    let lines: [&[u8]; 4] = [b"I: running apt-get update...", b"Err:1 http://snapshot.debian.org/archive/debian/20261001T030405Z forky InRelease", b"E: Failed to fetch http://snapshot.debian.org/archive/debian/20261001T030405Z/dists/forky/InRelease  Could not connect to snapshot.debian.org:80", b"E: mmdebstrap failed to run"];
    let tail = selected(&lines);
    assert_eq!(tail.lines().len(), 2);
    for (line, want) in tail.lines().iter().zip([lines[1], lines[2]]) {
        assert_eq!(&**line, want);
    }
    assert_eq!(
        (tail.cause(), tail.cause().class(), tail.cause().kind()),
        (&Cause::Fetch, Class::Operational, Kind::Tool)
    );
}
#[test]
fn mmdebstrap_s_own_framing_is_never_in_the_tail() {
    let tail = selected(&[
        b"E: setup failed: E: Failed to fetch",
        b"E: apt-get update failed: process exited with 100",
        b"W: hooklistener errored out",
        b"I: PID 123 exited with exit code 1",
        b"E: mmdebstrap failed to run",
    ]);
    assert_eq!(tail.lines().len(), 0);
    assert_eq!(tail.cause(), &Cause::Other);
}
#[test]
fn the_unmet_dependencies_and_replace_blocks_are_kept_whole() {
    let lines: [&[u8]; 4] = [
        b"The following packages have unmet dependencies:",
        b" apt : Depends: libc6",
        b"However the following packages replace it:",
        b"  apt-new",
    ];
    let tail = selected(&lines);
    assert_eq!(tail.lines().len(), 4);
    for (line, want) in tail.lines().iter().zip(lines) {
        assert_eq!(&**line, want);
    }
}
const ABSENT: &[&[u8]] = &[
    b"Err:1 URI 404  Not Found",
    b"E: Failed to fetch URI 404  Not Found",
    b"E: Failed to fetch URI NOT FOUND",
    b"E: The repository 'URI' does not have a Release file",
];
#[test]
fn a_404_or_a_missing_release_file_is_absent_and_configuration() {
    for line in ABSENT {
        let tail = selected(&[line]);
        assert_eq!(tail.cause(), &Cause::Absent);
        assert_eq!((tail.cause().class(), tail.cause().kind()), (Class::Configuration, Kind::Resolution));
    }
}
const FETCH: &[&[u8]] = &[
    b"Err:1 URI 503 Service Unavailable",
    b"E: Failed to fetch URI Hash Sum mismatch",
    b"E: Failed to fetch URI File has unexpected size",
    b"E: Failed to fetch URI Connection timed out",
];
#[test]
fn a_5xx_a_mismatch_or_a_timeout_is_a_fetch_and_operational() {
    for line in FETCH {
        let tail = selected(&[line]);
        assert_eq!((tail.cause(), tail.cause().class()), (&Cause::Fetch, Class::Operational));
    }
}
#[test]
fn no_space_left_is_operational() {
    let tail = selected(&[b"dpkg: error: No space left on device"]);
    assert_eq!((tail.cause(), tail.cause().class()), (&Cause::Space, Class::Operational));
}
#[test]
fn an_indented_line_after_an_e_line_stays_with_it() {
    let tail = selected(&[
        b"E: invalid package",
        b"\tcontinued\rdiagnostic",
        b"not an error",
        b" ignored",
        b"dpkg: error invalid",
        b" continuation",
    ]);
    let want: [&[u8]; 4] = [b"E: invalid package", b" continued diagnostic", b"dpkg: error invalid", b" continuation"];
    assert_eq!(tail.lines().len(), 4);
    for (line, want) in tail.lines().iter().zip(want) {
        assert_eq!(&**line, want);
    }
}
#[test]
fn only_the_last_twenty_lines_are_kept_each_cut_at_160_bytes() {
    let mut line = [b'x'; 200];
    for (byte, prefix) in line.iter_mut().zip(b"E: ") {
        *byte = *prefix;
    }
    let mut tail = Tail::new(&limits::SHIPPED);
    for at in b'a'..=b'y' {
        line[3] = at;
        tail::admit(&mut tail, &line, &limits::SHIPPED);
    }
    assert_eq!(tail.lines().len(), 20);
    for (line, at) in tail.lines().iter().zip(b'f'..=b'y') {
        assert_eq!(line.len(), 160);
        assert_eq!(line[3], at);
    }
}
#[test]
fn unable_to_locate_package_is_configuration_naming_the_package() {
    let tail = selected(&[b"E: Unable to locate package missing-package"]);
    assert_eq!(tail.cause(), &Cause::Unknown(fixtures::name(b"missing-package")));
    assert_eq!((tail.cause().class(), tail.cause().kind()), (Class::Configuration, Kind::Resolution));
}
#[test]
fn no_installation_candidate_is_configuration_naming_the_package() {
    let tail = selected(&[b"E: Package 'virtual-package' has no installation candidate"]);
    assert_eq!(tail.cause(), &Cause::Unknown(fixtures::name(b"virtual-package")));
    assert_eq!(tail.cause().class(), Class::Configuration);
}
#[test]
fn unmet_dependencies_name_the_first_package_apt_lists() {
    let tail = selected(&[
        b"The following packages have unmet dependencies:",
        b" apt : Depends: libc6",
        b" libc6 : Depends: something",
        b"E: Unable to correct problems, you have held broken packages.",
    ]);
    assert_eq!(tail.cause(), &Cause::Unsatisfiable(Some(fixtures::name(b"apt"))));
    assert_eq!((tail.cause().class(), tail.cause().kind()), (Class::Configuration, Kind::Resolution));
}
#[test]
fn an_operational_cause_anywhere_outranks_a_configuration_one() {
    let mut tail = selected(&[
        b"E: Unable to locate package missing-package",
        b"E: Failed to fetch URI Temporary failure resolving host",
    ]);
    for _ in 0_u32..25 {
        tail::admit(&mut tail, b"E: Unable to locate package other-package", &limits::SHIPPED);
    }
    assert_eq!(tail.cause(), &Cause::Resolve);
    assert_eq!(tail.lines().len(), 20);
}
#[test]
fn stderr_s_tail_is_taken_unless_it_is_empty() {
    let (env, mut stdout, mut above, mut below) = fixtures::opened();
    output::up(
        &mut stdout,
        &env,
        Up::Bytes(bytes::copy_of(b"E: Unable to locate package stdout-package\n")),
        &mut above,
        &mut below,
    );
    drop(below.pop());
    let (_, mut stderr, _, _) = fixtures::opened();
    let stdout_failure = tail::failure(&stderr, &stdout);
    assert_eq!(stdout_failure.cause, Cause::Unknown(fixtures::name(b"stdout-package")));
    output::up(&mut stderr, &env, Up::Bytes(bytes::copy_of(b"E: No space left on device\n")), &mut above, &mut below);
    let failure = tail::failure(&stderr, &stdout);
    assert_eq!(failure.cause, Cause::Space);
    assert_eq!(failure.tail.as_ref(), &[bytes::copy_of(b"E: No space left on device")]);
}
#[test]
fn a_200_byte_package_name_is_named_whole_though_its_line_is_cut() {
    let mut line = [b'a'; 228];
    let prefix = b"E: Unable to locate package ";
    for (byte, prefix) in line.iter_mut().zip(prefix) {
        *byte = *prefix;
    }
    let tail = selected(&[&line]);
    assert_eq!(tail.cause(), &Cause::Unknown(fixtures::name(&[b'a'; 200])));
    assert_eq!(tail.lines().iter().next().unwrap().len(), 160);
}
