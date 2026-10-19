use hearth_builder_world::{fixtures, simulated};
use skein_heap::Counting;
#[path = "../src/fixtures/bounds.rs"]
mod bounds;
#[global_allocator]
static HEAP: Counting = Counting;

#[test]
fn the_builder_holds_no_more_than_its_worst_case_at_every_iteration() {
    let outcome = simulated::checked(fixtures::SIMULATION_SEED);
    assert!(outcome.heap.is_some(), "the simulator checked every iteration's peak");
    assert!(
        fixtures::lists(&outcome.machine) > fixtures::STORE_PAGE,
        "domain/store.md, 8.2: the ledger contains more than one page of list operations"
    );
    assert_eq!(outcome.machine.contents(fixtures::OUTPUT).as_deref(), fixtures::THIN_OUTPUT);
}

#[test]
fn queued_specs_hold_no_more_than_the_worst_case() {
    let outcome = simulated::checked_with(&bounds::queued(), bounds::SEED, bounds::limits());
    let output = outcome.machine.contents(fixtures::OUTPUT).expect("printed specs");
    assert_eq!(String::from_utf8_lossy(&output).matches("target       ").count(), 4);
}

#[test]
fn recording_resolutions_holds_no_more_than_the_worst_case() {
    let outcome = simulated::checked_with(&fixtures::resolving(fixtures::THIN), bounds::SEED, fixtures::small_limits());
    assert_eq!(fixtures::simulation_records(&outcome.machine).len(), 1);
}

#[test]
fn building_holds_no_more_than_the_worst_case() {
    use hearth_lib::encode::Digest;
    let laid = [1_u8, 2]
        .map(|byte| fixtures::Laid::Layer(Box::new(fixtures::build::config_over(Digest([byte; 32]), Digest([0; 32])))));
    let scenario = fixtures::Scenario { laid: &laid, ..fixtures::build::building() };
    let outcome = simulated::checked_with(&scenario, bounds::SEED, fixtures::small_limits());
    assert_eq!(fixtures::client_exits(&outcome), [skein_io::kernel::Exit::Code(69)], "the fetch is unavailable");
    let output = String::from_utf8(outcome.machine.contents(fixtures::OUTPUT).expect("printed layers")).expect("text");
    assert_eq!(output.matches("   to build\n").count(), 3, "{output}");
    assert!(output.ends_with("failed       fetch packages   unavailable\n"), "{output}");
    let error = String::from_utf8(outcome.machine.contents(fixtures::ERROR).expect("progress")).expect("text");
    assert!(error.starts_with("resolving ") && !error.contains("strays"), "the laid records are indexed: {error}");
}
