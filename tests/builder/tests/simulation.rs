use hearth_builder_world::{fixtures, simulated};
use hearth_lib::encode::Item;
use skein_io::kernel::Exit;
use skein_sim::Config;

#[test]
fn hearth_render_through_the_simulator_prints_the_thin_spec() {
    let outcome = fixtures::simulated_thin();
    assert_eq!(
        outcome.machine.contents(fixtures::OUTPUT).as_deref(),
        Some(fixtures::THIN_TEXT.as_bytes()),
        "the shipped client, io and printer"
    );
}

#[test]
fn the_simulated_client_exits_zero_after_the_call() {
    assert_eq!(fixtures::client_exits(&fixtures::simulated_thin()), [Exit::Code(0)], "client/README.md, 3.3: done");
}

#[test]
fn diff_resolve_resolves_each_changed_key_once_in_turn_before_printing() {
    let outcome = simulated::run(&fixtures::diff_resolving(), fixtures::SIMULATION_SEED, Config::calm());
    assert_eq!(
        fixtures::tool_spans(&outcome.trace, b"/usr/bin/mmdebstrap").len(),
        fixtures::DIFF_RESOLUTION_RUNS,
        "the referee checked serial spawns before printing"
    );
    assert_eq!(
        fixtures::simulation_records(&outcome.machine)
            .into_iter()
            .map(|record| record.inputs.request.into_vec())
            .collect::<Vec<_>>(),
        [vec![Item::List(0)], vec![Item::List(1), Item::Bytes(b"pkg-23".as_slice().into())]],
        "exactly the changed pair's two keys are recorded"
    );
    let bytes = outcome.machine.contents(fixtures::OUTPUT).expect("client output");
    assert_eq!(fixtures::diff_record_kinds(&bytes), ["version", "change", "change", "outcome"]);
    assert_eq!(
        String::from_utf8(bytes).expect("JSON output").matches(r#""resolution":"known""#).count(),
        fixtures::KNOWN_REBUILDS,
        "the changed target's rebuild is known"
    );
}

#[test]
fn each_resolved_costs_equal_the_bytes_served_and_the_spawn_to_exit_span() {
    let outcome = fixtures::simulated_resolved();
    let records = fixtures::simulation_records(&outcome.machine);
    let costs = fixtures::only(&records).made.costs;
    assert_eq!(costs.fetched, outcome.machine.served_bytes(), "testing.md, 9.3: the mirror's own byte count");
    assert_eq!(
        costs.took,
        *fixtures::only(&fixtures::tool_spans(&outcome.trace, b"/usr/bin/mmdebstrap")),
        "testing.md, 9.3: spawn to exit"
    );
}

#[test]
fn git_and_mmdebstrap_take_the_time_their_work_takes() {
    let outcome = fixtures::simulated_resolved();
    let (minimum, maximum) = fixtures::pace_bounds(&outcome);
    assert!(outcome.end.as_nanos() >= minimum.as_nanos(), "each tool takes at least its minimum");
    assert!(outcome.end.as_nanos() <= maximum.as_nanos(), "each tool takes at most its maximum");
}

#[test]
fn a_stalled_mmdebstrap_meets_its_idle_timer() {
    let scenario = fixtures::Scenario {
        fault: Some(fixtures::Fault::Stalled { after: fixtures::STALL_AFTER }),
        ..fixtures::resolving(fixtures::THIN)
    };
    let outcome = simulated::run(&scenario, fixtures::SIMULATION_SEED, Config::calm());
    assert!(outcome.end.as_nanos() >= fixtures::RESOLVE_IDLE.as_nanos(), "the shipped sixty-second idle timer fired");
    assert_eq!(fixtures::client_exits(&outcome), [Exit::Code(75)], "the timeout is operational");
}

#[test]
fn two_builders_over_one_store_take_the_lock_in_turn() {
    let outcome = simulated::two(&fixtures::thin(), fixtures::SIMULATION_SEED, Config::calm());
    assert!(
        outcome.machine.borrow().ledger.iter().any(|entry| entry.refused == Some(hearth_builder_world::fixtures::BUSY)),
        "the second builder's lock was busy"
    );
    assert_eq!(outcome.machine.contents(fixtures::OUTPUT).as_deref(), Some(fixtures::THIN_TEXT.as_bytes()));
    assert_eq!(outcome.machine.contents(fixtures::SECOND_OUTPUT).as_deref(), Some(fixtures::THIN_TEXT.as_bytes()));
}

#[test]
fn a_lock_held_past_the_resolve_limit_is_waited_out_then_the_call_renders() {
    let outcome =
        simulated::with_lock(&fixtures::thin(), fixtures::SIMULATION_SEED, Config::calm(), fixtures::LONG_LOCK_RELEASE);
    assert!(outcome.end >= fixtures::LONG_LOCK_RELEASE, "store.md, 8.2 sets no lock wait limit");
    assert_eq!(
        outcome.machine.contents(fixtures::ERROR).as_deref(),
        Some(fixtures::WAITING_TEXT),
        "busy is reported once"
    );
    assert_eq!(outcome.machine.contents(fixtures::OUTPUT).as_deref(), Some(fixtures::THIN_TEXT.as_bytes()));
}

#[test]
fn the_protocol_and_simulated_worlds_leave_the_same_store() {
    let protocol = hearth_builder_world::protocol::run(&fixtures::thin(), fixtures::SIMULATION_SEED);
    let outcome = fixtures::simulated_thin();
    assert_eq!(
        fixtures::store_files(&outcome.machine),
        fixtures::store_files(&protocol.shared),
        "ruling 182: cache names, payloads and record identities; no temporary file"
    );
}
