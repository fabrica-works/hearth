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
