use super::fixtures::{self, ERROR, KEY, Layer, OUTPUT, REQUEST};
use crate::{
    Asked,
    builder::vocabulary::Event as RootEvent,
    render::{
        errors::{Command, Ended, Limit, Tool},
        resolution::{Cause, Inst},
        vocabulary::{Event, Failure},
    },
};
use skein_io::{
    self as io, file,
    kernel::{Exit, Pipe, Way},
};
use skein_lib::{Duration, Time};
#[test]
fn a_resolve_makes_its_scratch_writes_sources_list_then_spawns_mmdebstrap() {
    let mut layer = Layer::resolution();
    layer.down(fixtures::resolve(KEY, &[]));
    let Some(Asked::File(file::Request::MakeDirectory { path, mode, .. })) = layer.asked() else { panic!("scratch") };
    assert_eq!(
        (path.as_ref(), mode),
        (b"0101010101010101010101010101010101010101010101010101010101010101".as_slice(), 0o700)
    );
    layer.file_up(fixtures::made());
    let Some(Asked::File(file::Request::Store { path, bytes, expected, no_follow, .. })) = layer.asked() else {
        panic!("sources")
    };
    assert_eq!(
        (path.as_ref(), bytes.as_ref(), expected, no_follow),
        (
            b"0101010101010101010101010101010101010101010101010101010101010101/sources.list".as_slice(),
            b"deb http://snapshot.debian.org/archive/debian/20261001T030405Z/ forky main\n".as_slice(),
            file::Expect::Absent,
            true
        )
    );
    layer.file_up(fixtures::stored());
    let Some(Asked::Io(io::Request::Spawn { spawn, .. })) = layer.asked() else { panic!("spawn") };
    assert_eq!(
        (spawn.program.as_ref(), spawn.dir.as_ref(), spawn.pipes.as_ref()),
        (
            b"/usr/bin/mmdebstrap".as_slice(),
            b"".as_slice(),
            [Pipe { child: 1, way: Way::Out, parent: None }, Pipe { child: 2, way: Way::Out, parent: None }].as_slice()
        )
    );
}
const PIPES: &[skein_lib::Token] = &[OUTPUT, ERROR];
#[test]
fn inst_lines_from_either_pipe_reach_render_as_they_come() {
    for (row, &pipe) in PIPES.iter().enumerate() {
        let mut layer = Layer::resolution();
        layer.start_resolve();
        layer.resolve_line(pipe, b"Inst apt (3.3.3 Debian:unstable [amd64])\n");
        assert_eq!(
            layer.told(),
            Some(RootEvent::Render(Event::Inst {
                request: REQUEST,
                inst: Inst {
                    name: b"apt".as_slice().into(),
                    version: b"3.3.3".as_slice().into(),
                    architecture: b"amd64".as_slice().into()
                }
            })),
            "row {row}"
        );
    }
}
#[test]
fn a_silent_run_is_closed_at_its_idle_timer_and_fails_timed_out() {
    let mut layer = Layer::resolution();
    layer.start_resolve();
    assert_eq!(layer.deadline(), Some(Time::ZERO.saturating_add(Duration::from_secs(60))));
    layer.advance_resolve(Duration::from_secs(60));
    let events = layer.resolve_answers(b"", b"", Exit::Code(0));
    let Failure::Run { tool, cause } = fixtures::resolution_failure(&events) else { panic!("run") };
    assert_eq!((&tool.ended, cause.as_ref()), (&Ended::Timeout, None));
}
#[test]
fn renders_deadline_ends_a_run_that_still_speaks() {
    let mut layer = Layer::resolution();
    layer.start_resolve();
    for _ in 0_u32..11 {
        layer.advance_resolve(Duration::from_secs(50));
        layer.resolve_line(OUTPUT, b"speaking\n");
        drop(layer.asked());
    }
    layer.advance_resolve(Duration::from_secs(50));
    let events = layer.resolve_answers(b"", b"", Exit::Code(0));
    let Failure::Run { tool, .. } = fixtures::resolution_failure(&events) else { panic!("run") };
    assert_eq!(tool.ended, Ended::Timeout);
}
#[test]
fn a_failed_run_answers_its_cause_and_stderrs_tail() {
    let mut layer = Layer::resolution();
    layer.down(fixtures::resolve(KEY, &[]));
    let events = layer.resolve_answers(b"E: stdout\n", b"E: Failed to fetch archive\n", Exit::Code(1));
    assert_eq!(
        fixtures::resolution_failure(&events),
        &Failure::Run {
            tool: Tool {
                command: Command::Mmdebstrap,
                ended: Ended::Exit(1),
                tail: Box::new([b"E: Failed to fetch archive".as_slice().into()])
            },
            cause: Some(Cause::Fetch),
        }
    );
}
const ABSENT: &[(&[u8], &[u8], &[u8])] = &[
    (b"Err:2 http://two/ forky-updates Release", b"http://two/", b"forky-updates"),
    (b"Err:1 http://nowhere/ unavailable", b"http://one/", b"forky"),
];
#[test]
fn an_absent_run_names_the_source_line_its_404_names() {
    for (row, &(tail, uri, suite)) in ABSENT.iter().enumerate() {
        assert_eq!(
            crate::translate::resolution::cause(
                hearth_protocol_mmdebstrap::tail::Cause::Absent,
                &fixtures::source_lines(),
                &[tail.into()]
            ),
            Cause::Absent { uri: uri.into(), suite: suite.into() },
            "row {row}"
        );
    }
}
const REFUSALS: &[u8] = &[0, 1, 2];
#[test]
fn a_refused_scratch_sources_list_or_spawn_fails_the_run_as_could_not_start() {
    for (row, &at) in REFUSALS.iter().enumerate() {
        let mut layer = Layer::resolution();
        layer.down(fixtures::resolve(KEY, &[]));
        drop(layer.asked());
        if at > 0 {
            layer.file_up(fixtures::made());
            drop(layer.asked());
        }
        if at > 1 {
            layer.file_up(fixtures::stored());
            drop(layer.asked());
            layer.up(io::Event::Failed { owner: crate::resolve::OWNER, error: io::Error::Other });
        } else {
            layer.file_up(fixtures::refused());
        }
        let events = layer.resolve_answers(b"", b"", Exit::Code(0));
        let Failure::Run { tool, cause } = fixtures::resolution_failure(&events) else { panic!("run") };
        assert_eq!((&tool.ended, cause.as_ref()), (&Ended::Start, None), "row {row}");
    }
}
#[test]
fn an_inst_flood_fails_the_run_too_large_at_the_closure_limit() {
    let mut layer = Layer::resolution_with(fixtures::TWO_INSTS);
    layer.down(fixtures::resolve(KEY, &[]));
    let events = layer.resolve_answers(
        b"Inst apt (1 Debian [amd64])\nInst apt (1 Debian [amd64])\nInst apt (1 Debian [amd64])\n",
        b"",
        Exit::Code(0),
    );
    let terminal = events.last().expect("terminal");
    assert_eq!(
        terminal,
        &RootEvent::Render(Event::Failed {
            request: REQUEST,
            class: crate::render::vocabulary::Class::Configuration,
            at: None,
            failure: Failure::TooLarge { limit: Limit::Closure, max: u64::from(fixtures::TWO_INSTS.mmdebstrap.insts) }
        })
    );
}
#[test]
fn the_scratch_is_walked_away_before_the_terminal() {
    let mut layer = Layer::resolution();
    layer.start_resolve();
    layer.resolve_end();
    layer.resolve_exit(Exit::Code(1));
    for _ in 0_u32..5 {
        match layer.asked() {
            Some(Asked::Io(io::Request::Close { entity })) => {
                let owner = if entity == skein_lib::Token::new(200) { crate::resolve::OWNER } else { entity };
                layer.up(io::Event::Closed { owner });
            }
            Some(Asked::Io(io::Request::Stream {
                down: skein_lib::stream::Down::Demand { read: skein_lib::stream::Read::Nothing, .. },
                ..
            })) => {}
            request => panic!("closing child resources: {request:?}"),
        }
    }
    let Some(Asked::File(request @ file::Request::OpenDirectory { .. })) = layer.asked() else {
        panic!("scratch walk")
    };
    assert_eq!(layer.told(), None);
    layer.resolve_file(request);
    for _ in 0_u32..3 {
        let Some(Asked::File(request)) = layer.asked() else { panic!("scratch walk step") };
        layer.resolve_file(request);
    }
    assert_eq!(layer.resolve_answers(b"", b"", Exit::Code(1)).len(), 1);
}
