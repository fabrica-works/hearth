use hearth_builder_world::{fixtures, protocol};

#[test]
fn random_cuts_kills_and_vanished_files_settle_and_each_fault_falls() {
    let mut fell = std::collections::BTreeSet::new();
    for seed in fixtures::FUZZY_PROTOCOL_SEEDS {
        let (case, scenario) = fixtures::swept_fault(seed);
        let run = protocol::run(&scenario, seed);
        assert_eq!(run.fault_fell, case.fell, "seed {seed}, {}: the drawn fault fell", case.name);
        fell.insert(case.name);
        assert_eq!(run.spawned, run.reaped, "seed {seed}: every child reaped");
        assert_eq!(run.opened_pipes, run.ended_pipes, "seed {seed}: every output read to its end");
        assert_eq!(run.code, case.code, "seed {seed}, {}: terminal outcome", case.name);
    }
    assert_eq!(
        fell.into_iter().collect::<Vec<_>>(),
        ["absent", "cache gone", "killed", "normal", "not executable"],
        "the sweep tried every fault and normal pipes"
    );
}
