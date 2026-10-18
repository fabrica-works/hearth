use hearth_builder_world::{fixtures, simulated};
use skein_heap::Counting;
#[global_allocator]
static HEAP: Counting = Counting;

#[test]
fn the_builder_holds_no_more_than_its_worst_case_at_every_iteration() {
    let outcome = simulated::checked(fixtures::SIMULATION_SEED);
    assert!(outcome.heap.is_some(), "the simulator checked every iteration's peak");
    assert!(
        fixtures::lists(&outcome.machine) > fixtures::STORE_PAGE,
        "store.md, 8.2: the ledger contains more than one page of list operations"
    );
    assert_eq!(outcome.machine.contents(fixtures::OUTPUT).as_deref(), fixtures::THIN_OUTPUT);
}
