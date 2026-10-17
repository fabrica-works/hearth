use super::fixtures::{self, BUILDER};
use crate::{
    hearth::Tool,
    linux::{At, Counts, Exit, How, Item, Op, Pipe, Refusal, Reply, Signal, Spawn, Target, Way},
};

const UNCLAIMED: [Exit; 1] = [Exit::Code(127)];
#[test]
fn a_spawn_of_a_file_no_program_claims_exits_127() {
    let mut machine = fixtures::whole();
    machine.linux_mut().lay(&[Item::file(b"/usr/bin/stray", 0o755, b"#!/bin/sh\n")]);
    let parent = fixtures::builder(&mut machine);
    let (child, _) = fixtures::spawn(&mut machine, parent, b"/usr/bin/stray", &[], &[]);
    assert_eq!(machine.call(parent, Op::Wait { child }), Ok(Reply::Exited(UNCLAIMED[0])));
}
const NO_TOOL: [Option<Tool>; 1] = [None];
#[test]
fn no_tool_claims_an_uninstalled_executable() {
    let mut machine = fixtures::whole();
    machine.linux_mut().lay(&[Item::file(b"/usr/bin/stray", 0o755, b"")]);
    let parent = fixtures::builder(&mut machine);
    let (child, _) = fixtures::spawn(&mut machine, parent, b"/usr/bin/stray", &[], &[]);
    assert_eq!(machine.tool(child), NO_TOOL[0]);
}
const END: [Result<Reply, Refusal>; 1] = [Ok(Reply::Ended)];
#[test]
fn an_unclaimed_program_ends_its_stdout_before_spawn_returns() {
    let mut machine = fixtures::whole();
    machine.linux_mut().lay(&[Item::file(b"/usr/bin/stray", 0o755, b"")]);
    let parent = fixtures::builder(&mut machine);
    let (_, pipes) = fixtures::spawn(&mut machine, parent, b"/usr/bin/stray", &[], &[Pipe { child: 1, way: Way::Out }]);
    assert_eq!(&machine.call(parent, Op::Take { pipe: pipes[0], most: 64 }), &END[0]);
}
const REFUSED_PROGRAMS: [(&[u8], Refusal); 1] = [(b"/usr/bin/missing", Refusal::NotFound)];
#[test]
fn a_spawn_of_a_missing_program_is_refused_as_not_found() {
    let mut machine = fixtures::whole();
    let parent = fixtures::builder(&mut machine);
    let root = fixtures::open(machine.linux_mut(), parent, b"/", How::Directory);
    for &(program, expected) in &REFUSED_PROGRAMS {
        let spawn = Spawn {
            program: program.into(),
            args: Box::default(),
            env: Box::default(),
            root,
            dir: b"srv".as_slice().into(),
            pipes: Box::default(),
        };
        assert_eq!(machine.call(parent, Op::Spawn(spawn)), Err(expected));
    }
}
const UNEXECUTABLE: [Refusal; 1] = [Refusal::Permission];
#[test]
fn a_program_its_ids_cannot_execute_is_refused_as_permission() {
    let mut machine = fixtures::whole();
    machine.linux_mut().lay(&[Item::file(b"/usr/bin/private", 0o744, b"")]);
    let parent = fixtures::builder(&mut machine);
    let root = fixtures::open(machine.linux_mut(), parent, b"/", How::Directory);
    let spawn = Spawn {
        program: b"/usr/bin/private".as_slice().into(),
        args: Box::default(),
        env: Box::default(),
        root,
        dir: b"srv".as_slice().into(),
        pipes: Box::default(),
    };
    assert_eq!(machine.call(parent, Op::Spawn(spawn)), Err(UNEXECUTABLE[0]));
}
type InvocationCase<'a> = (&'a [&'a [u8]], &'a [&'a [u8]], &'a [u8]);
const INVOCATION: [InvocationCase<'_>; 1] =
    [(&[b"-C", b"/srv", b"rev-parse", b"--verify", b"HEAD"], &[b"LC_ALL=C", b"GIT_CONFIG_NOSYSTEM=1"], b"/srv")];
#[test]
fn a_child_gets_its_arguments_and_exactly_its_environment() {
    let mut machine = fixtures::whole();
    fixtures::install_git(&mut machine);
    let parent = fixtures::builder(&mut machine);
    let root = fixtures::open(machine.linux_mut(), parent, b"/", How::Directory);
    let (args, env, dir) = INVOCATION[0];
    let spawn = Spawn {
        program: b"/usr/bin/git".as_slice().into(),
        args: args.iter().map(|v| Box::from(*v)).collect(),
        env: env.iter().map(|v| Box::from(*v)).collect(),
        root,
        dir: b"srv".as_slice().into(),
        pipes: Box::default(),
    };
    let (child, _) = fixtures::spawned(machine.call(parent, Op::Spawn(spawn)).unwrap());
    let invocation = machine.linux().invocation(child);
    let observed_args: Vec<_> = invocation.args.iter().map(AsRef::as_ref).collect();
    let observed_env: Vec<_> = invocation.env.iter().map(AsRef::as_ref).collect();
    assert_eq!((observed_args.as_slice(), observed_env.as_slice(), invocation.dir.as_ref()), (args, env, dir));
}
const PIPE_CAPACITY: [u32; 3] = [4096, 0, 3];
#[test]
fn a_full_pipe_takes_no_more_until_its_reader_takes_some() {
    let mut machine = fixtures::whole();
    fixtures::install_git(&mut machine);
    let parent = fixtures::builder(&mut machine);
    let (child, pipes) =
        fixtures::spawn(&mut machine, parent, b"/usr/bin/git", &[], &[Pipe { child: 0, way: Way::In }]);
    assert_eq!(fixtures::sent(&mut machine, parent, pipes[0], &[b'x'; 4097]), Ok(PIPE_CAPACITY[0]));
    assert_eq!(fixtures::sent(&mut machine, parent, pipes[0], b"again"), Ok(PIPE_CAPACITY[1]));
    let input = machine.linux().descriptor(child, 0);
    fixtures::took(&mut machine, child, input, 3);
    assert_eq!(fixtures::sent(&mut machine, parent, pipes[0], b"again"), Ok(PIPE_CAPACITY[2]));
}
const KILLED: [Exit; 1] = [Exit::Signal(9)];
#[test]
fn a_kill_ends_the_child_with_signal_nine_and_ends_its_pipes() {
    let mut machine = fixtures::whole();
    fixtures::install_git(&mut machine);
    let parent = fixtures::builder(&mut machine);
    let (child, _) = fixtures::spawn(&mut machine, parent, b"/usr/bin/git", &[], &[Pipe { child: 1, way: Way::Out }]);
    machine.call(parent, Op::Signal { child, signal: Signal::Kill, to: Target::Group }).unwrap();
    assert_eq!(machine.call(parent, Op::Wait { child }), Ok(Reply::Exited(KILLED[0])));
}
const COUNTS: [Counts; 1] = [Counts { read: 3, written: 2, changed: 1, spawned: 1 }];
#[test]
fn counts_give_a_process_s_bytes_changes_and_spawns_after_its_exit() {
    let mut machine = fixtures::whole();
    fixtures::install_git(&mut machine);
    machine.linux_mut().lay(&[Item::file(b"/srv/file", 0o666, b"abc")]);
    let parent = fixtures::builder(&mut machine);
    let file = fixtures::open(machine.linux_mut(), parent, b"/srv/file", How::Write);
    fixtures::read(machine.linux_mut(), parent, file, 0, 3);
    machine.call(parent, Op::Write { file, at: 0, bytes: b"xy".as_slice().into() }).unwrap();
    fixtures::spawn(&mut machine, parent, b"/usr/bin/git", &[], &[]);
    machine.call(parent, Op::Exit { exit: Exit::Code(0) }).unwrap();
    assert_eq!(machine.linux().counts(parent), COUNTS[0]);
}
const PENDING: [Option<Signal>; 1] = [Some(Signal::Terminate)];
#[test]
fn a_terminate_stays_pending_for_its_program() {
    let mut machine = fixtures::whole();
    fixtures::install_git(&mut machine);
    let parent = fixtures::builder(&mut machine);
    let (child, _) = fixtures::spawn(&mut machine, parent, b"/usr/bin/git", &[], &[]);
    machine.call(parent, Op::Signal { child, signal: Signal::Terminate, to: Target::Child }).unwrap();
    assert_eq!(machine.linux().pending(child), PENDING[0]);
}
const NULL_IO: [Result<Reply, Refusal>; 2] = [Ok(Reply::Ended), Ok(Reply::Sent(7))];
#[test]
fn a_descriptor_given_no_pipe_acts_as_dev_null() {
    let mut machine = fixtures::whole();
    fixtures::install_git(&mut machine);
    let parent = fixtures::builder(&mut machine);
    let (child, _) = fixtures::spawn(&mut machine, parent, b"/usr/bin/git", &[], &[]);
    let input = machine.linux().descriptor(child, 0);
    let output = machine.linux().descriptor(child, 1);
    assert_eq!(&machine.call(child, Op::Take { pipe: input, most: 8 }), &NULL_IO[0]);
    assert_eq!(&machine.call(child, Op::Send { pipe: output, bytes: b"ignored".as_slice().into() }), &NULL_IO[1]);
}
const RELEASED: [Result<(), Refusal>; 1] = [Ok(())];
#[test]
fn an_exit_releases_the_process_s_lock() {
    let mut machine = fixtures::whole();
    let first = fixtures::builder(&mut machine);
    let second = machine.linux_mut().login(BUILDER, b"/srv");
    let a = fixtures::open(machine.linux_mut(), first, b"/srv", How::Directory);
    let b = fixtures::open(machine.linux_mut(), second, b"/srv", How::Directory);
    machine.call(first, Op::Lock { file: a }).unwrap();
    machine.call(first, Op::Exit { exit: Exit::Code(0) }).unwrap();
    assert_eq!(fixtures::done(machine.linux_mut(), second, Op::Lock { file: b }), RELEASED[0]);
}
#[test]
#[should_panic(expected = "a process it started")]
fn a_call_from_an_exited_process_asserts() {
    let mut machine = fixtures::whole();
    let parent = fixtures::builder(&mut machine);
    machine.call(parent, Op::Exit { exit: Exit::Code(0) }).unwrap();
    machine.call(parent, Op::Open { at: At::Root, path: b"/srv".as_slice().into(), how: How::Directory }).unwrap();
}
const SIGNALED: [Exit; 1] = [Exit::Signal(15)];
#[test]
fn a_program_can_end_itself_by_signal_fifteen() {
    let mut machine = fixtures::whole();
    fixtures::install_git(&mut machine);
    let parent = fixtures::builder(&mut machine);
    let (child, _) = fixtures::spawn(&mut machine, parent, b"/usr/bin/git", &[], &[]);
    machine.call(child, Op::Exit { exit: Exit::Signal(15) }).unwrap();
    assert_eq!(machine.call(parent, Op::Wait { child }), Ok(Reply::Exited(SIGNALED[0])));
}
const BROKEN: [Refusal; 1] = [Refusal::BrokenPipe];
#[test]
fn a_pipe_without_a_reader_refuses_a_send() {
    let mut machine = fixtures::whole();
    fixtures::install_git(&mut machine);
    let parent = fixtures::builder(&mut machine);
    let (child, pipes) =
        fixtures::spawn(&mut machine, parent, b"/usr/bin/git", &[], &[Pipe { child: 0, way: Way::In }]);
    let input = machine.linux().descriptor(child, 0);
    machine.call(child, Op::Close { handle: input }).unwrap();
    assert_eq!(fixtures::sent(&mut machine, parent, pipes[0], b"bytes"), Err(BROKEN[0]));
}
const EMPTY: [Result<Reply, Refusal>; 1] = [Ok(Reply::Empty)];
#[test]
fn an_empty_pipe_with_a_writer_is_still_live() {
    let mut machine = fixtures::whole();
    fixtures::install_git(&mut machine);
    let parent = fixtures::builder(&mut machine);
    let (_, pipes) = fixtures::spawn(&mut machine, parent, b"/usr/bin/git", &[], &[Pipe { child: 1, way: Way::Out }]);
    assert_eq!(&machine.call(parent, Op::Take { pipe: pipes[0], most: 64 }), &EMPTY[0]);
}
#[test]
fn killing_a_child_ends_its_stdout_pipe() {
    let mut machine = fixtures::whole();
    fixtures::install_git(&mut machine);
    let parent = fixtures::builder(&mut machine);
    let (child, pipes) =
        fixtures::spawn(&mut machine, parent, b"/usr/bin/git", &[], &[Pipe { child: 1, way: Way::Out }]);
    machine.call(parent, Op::Signal { child, signal: Signal::Kill, to: Target::Group }).unwrap();
    assert_eq!(&machine.call(parent, Op::Take { pipe: pipes[0], most: 64 }), &END[0]);
}
const DRAINED: [&[u8]; 1] = [b"written"];
#[test]
fn an_exited_writer_s_pipe_drains_before_it_ends() {
    let mut machine = fixtures::whole();
    fixtures::install_git(&mut machine);
    let parent = fixtures::builder(&mut machine);
    let (child, pipes) =
        fixtures::spawn(&mut machine, parent, b"/usr/bin/git", &[], &[Pipe { child: 1, way: Way::Out }]);
    let output = machine.linux().descriptor(child, 1);
    fixtures::sent(&mut machine, child, output, b"written").unwrap();
    machine.call(child, Op::Exit { exit: Exit::Code(0) }).unwrap();
    assert_eq!(fixtures::took(&mut machine, parent, pipes[0], 64).as_ref(), DRAINED[0]);
    assert_eq!(&machine.call(parent, Op::Take { pipe: pipes[0], most: 64 }), &END[0]);
}
const INHERITED: [(u32, u32, u32); 1] = [(1000, 1000, 0o755)];
#[test]
fn a_child_creates_files_with_its_parent_s_ids_and_umask() {
    let mut machine = fixtures::whole();
    fixtures::install_git(&mut machine);
    let parent = fixtures::builder(&mut machine);
    let (child, _) = fixtures::spawn(&mut machine, parent, b"/usr/bin/git", &[], &[]);
    fixtures::open(machine.linux_mut(), child, b"/srv/child-file", How::Create { mode: Some(0o777) });
    let facts = machine.linux().facts(b"/srv/child-file").unwrap();
    assert_eq!((facts.user, facts.group, facts.mode), INHERITED[0]);
}
const SPAWN_ORDER: [(&[u8], Refusal); 2] = [(b"../escape", Refusal::Escape), (b"missing", Refusal::NotFound)];
#[test]
fn a_spawn_checks_its_working_directory_before_its_program() {
    let mut machine = fixtures::whole();
    let parent = fixtures::builder(&mut machine);
    let root = fixtures::open(machine.linux_mut(), parent, b"/srv", How::Directory);
    for &(dir, expected) in &SPAWN_ORDER {
        let spawn = Spawn {
            program: b"/missing".as_slice().into(),
            args: Box::default(),
            env: Box::default(),
            root,
            dir: dir.into(),
            pipes: Box::default(),
        };
        assert_eq!(machine.call(parent, Op::Spawn(spawn)), Err(expected), "{dir:?}");
    }
}
