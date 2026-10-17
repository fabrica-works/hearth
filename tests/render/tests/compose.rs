use hearth_domain_render::{
    limits,
    parts::PartRecord,
    vocabulary::{Answer, Outcome, Request},
};
use hearth_render_world::{
    fixtures::{self, only},
    parent,
};
#[test]
fn render_lists_once_then_reads_every_configuration_document_once() {
    for seed in 0..8 {
        let mut repository = fixtures::two_targets();
        repository.entries.push(fixtures::entry(b"fleet.json", b'5'));
        repository.documents.push((fixtures::oid(b'5'), fixtures::document(None)));
        repository.entries.sort_by(|a, b| a.path.cmp(&b.path));
        let run = parent::run(&repository, &[b"x1-prod"], seed);
        let [Request::List { .. }, Request::Read { blobs, .. }] = run.requests.as_slice() else {
            panic!("list then documents")
        };
        assert_eq!(
            blobs.iter().map(|blob| blob.oid.0.as_ref()).collect::<Vec<_>>(),
            [
                b"1111111111111111111111111111111111111111",
                b"5555555555555555555555555555555555555555",
                b"4444444444444444444444444444444444444444",
                b"3333333333333333333333333333333333333333",
                b"2222222222222222222222222222222222222222"
            ],
            "seed {seed}"
        );
    }
}
#[test]
fn fleet_and_environment_keys_never_add_a_task() {
    for seed in 0..8 {
        let mut repository = fixtures::x1_prod();
        repository.entries.push(fixtures::entry(b"fleet.json", b'5'));
        repository.documents.push((fixtures::oid(b'5'), fixtures::tasks(&[(b"motd", false)], None)));
        fixtures::set_document(&mut repository, b'1', fixtures::tasks(&[(b"issue", false)], None));
        repository.entries.sort_by(|a, b| a.path.cmp(&b.path));
        let run = parent::run(&repository, &[], seed);
        let Answer::Spec(spec) = only(&run.answers) else { panic!("spec") };
        let PartRecord::Config(config) = &only(&spec.parts).record;
        assert_eq!(config.tree.as_ref(), [], "seed {seed}");
        assert_eq!(run.requests.len(), 2, "absent tasks are never read, seed {seed}");
    }
}
#[test]
fn documents_past_the_revision_s_budget_fail_the_call() {
    for seed in 0..8 {
        let mut repository = fixtures::x1_prod();
        for entry in &mut repository.entries {
            entry.size = 24;
        }
        let run = parent::run_with(&repository, &[], seed, limits::Limits { decoded: 64, ..limits::SHIPPED });
        assert_eq!(run.answers, [], "seed {seed}");
        assert_eq!(run.outcome, Outcome::Failed, "seed {seed}");
        assert_eq!(run.requests.len(), 1, "budget checked before reading, seed {seed}");
    }
}
