use hearth_builder_world::{
    domain,
    fixtures::{self, budgets},
    simulated,
};
use skein_sim::Config;

#[test]
fn rendering_the_fleet_lists_once_and_reads_each_blob_once() {
    let run = budgets::fleet_run(false);
    let reads = budgets::reads(&run);
    assert_eq!(reads.lists, budgets::RENDER_LISTS, "domain/render.md, 3.10: one listing");
    assert_eq!(budgets::distinct(&reads.blobs), reads.blobs.len(), "each oid read once");
    assert_eq!(reads.blobs.len(), budgets::FLEET_BLOBS);
    assert!(reads.blobs.len() < budgets::FLEET_ENTRIES, "the duplicate content is shared");
}

#[test]
fn a_diff_reads_a_blob_the_two_revisions_share_once() {
    let run = budgets::fleet_run(true);
    let reads = budgets::reads(&run);
    assert_eq!(reads.lists, budgets::DIFF_LISTS);
    assert_eq!(reads.blobs.len(), budgets::DIFF_BLOBS);
    assert_eq!(budgets::distinct(&reads.blobs), reads.blobs.len(), "domain/render.md, 8.4: shared oids reused");
}

#[test]
fn a_diffs_second_revision_reads_only_the_blobs_that_changed() {
    let reads = budgets::reads(&budgets::fleet_run(true));
    assert_eq!(reads.lists, budgets::DIFF_LISTS);
    assert_eq!(reads.second.iter().map(|oid| oid.0.as_ref()).collect::<Vec<_>>(), [budgets::FLEET_EDIT_OID]);
}

#[test]
fn a_render_spawns_git_once_per_read_and_twice_more_and_a_known_resolution_no_mmdebstrap() {
    for row in budgets::PROGRAMS {
        let files = budgets::files(row.repository);
        let laid = [fixtures::Laid::Resolution(fixtures::test_min_recorded())];
        let scenario = budgets::scenario(row.repository, &files, &laid);
        let reads = budgets::reads(&domain::Run::new(&scenario, fixtures::SIMULATION_SEED).finish());
        let outcome = simulated::run(&scenario, fixtures::SIMULATION_SEED, Config::calm());
        assert_eq!(budgets::programs(&outcome.trace), (row.git, row.mmdebstrap), "{:?}", row.repository);
        assert_eq!(reads.batches + budgets::DISCOVERY_SPAWNS, row.git, "{:?}", row.repository);
    }
}
