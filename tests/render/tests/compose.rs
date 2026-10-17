use hearth_domain_render::{
    limits,
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
        let requests = fixtures::configuration_requests(&run.requests);
        let [Request::List { .. }, Request::Read { blobs, .. }] = requests.as_slice() else {
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
        let mut repository = fixtures::with_task();
        fixtures::set_document(&mut repository, b'2', fixtures::document(None));
        repository.entries.push(fixtures::entry(b"fleet.json", b'5'));
        repository.documents.push((fixtures::oid(b'5'), fixtures::tasks(&[(b"motd", false)], None)));
        fixtures::set_document(&mut repository, b'1', fixtures::tasks(&[(b"motd", false)], None));
        repository.entries.sort_by(|a, b| a.path.cmp(&b.path));
        let run = parent::run(&repository, &[], seed);
        let Answer::Spec(spec) = only(&run.answers) else { panic!("spec") };
        let config = hearth_render_world::fixtures::config_part(&spec.parts);
        assert_eq!(config.tree.as_ref(), [], "seed {seed}");
        let requests = fixtures::configuration_requests(&run.requests);
        assert_eq!(requests.len(), 2, "absent tasks are never read, seed {seed}");
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
        assert_eq!(
            run.answers,
            [Answer::Error(hearth_domain_render::errors::Error {
                side: None,
                kind: hearth_domain_render::errors::Kind::Decode(hearth_domain_render::errors::Decode::TooLarge {
                    limit: hearth_domain_render::errors::Limit::Decoded,
                    max: 64
                }),
                class: hearth_domain_render::vocabulary::Class::Configuration,
                at: Some(hearth_domain_render::errors::Place {
                    path: Box::from(b"hosts/x1/x1.json".as_slice()),
                    position: None
                }),
                also: None,
                target: None,
                task: None,
            })],
            "seed {seed}"
        );
        assert_eq!(run.outcome, Outcome::Failed, "seed {seed}");
        assert_eq!(run.requests.len(), 1, "budget checked before reading, seed {seed}");
    }
}

#[test]
fn targets_are_discovered_from_every_base_s_members() {
    let mut repository = fixtures::cluster();
    repository.entries.push(fixtures::entry(b"hosts/stoa/stoa-prod.json", b'4'));
    repository.entries.push(fixtures::entry(b"hosts/stoa/stoa.json", b'5'));
    repository.documents.push((fixtures::oid(b'4'), fixtures::document(Some(fixtures::placement()))));
    repository.documents.push((fixtures::oid(b'5'), fixtures::members(&[(b"a", false), (b"b", false)])));
    for seed in 0..8 {
        let run = parent::run(&repository, &[], seed);
        let names: Vec<_> = run
            .answers
            .iter()
            .map(|answer| {
                let Answer::Spec(spec) = answer else { panic!("spec") };
                spec.meta.target.as_ref()
            })
            .collect();
        assert_eq!(names, [b"stoa-a-prod".as_slice(), b"stoa-b-prod", b"x1-1-prod", b"x1-2-prod"], "seed {seed}");
    }
}
#[test]
fn the_target_past_the_limit_errs_and_fails_the_call() {
    use hearth_domain_render::errors::{Decode, Kind, Limit};
    let mut repository = fixtures::cluster();
    fixtures::set_document(&mut repository, b'2', fixtures::members(&[(b"1", false), (b"2", false), (b"3", false)]));
    let run = parent::run_with(&repository, &[], 0, limits::Limits { targets: 2, ..limits::SHIPPED });
    let Answer::Error(error) = only(&run.answers) else { panic!("limit error") };
    assert_eq!(error.kind, Kind::Decode(Decode::TooLarge { limit: Limit::Targets, max: 2 }));
    assert_eq!(run.outcome, Outcome::Failed);
}
#[test]
fn a_named_target_a_refused_base_hides_is_no_usage() {
    use hearth_domain_render::{errors::Decode, revision::Position, vocabulary::Decoded};
    let mut repository = fixtures::cluster();
    fixtures::set_document(
        &mut repository,
        b'2',
        Decoded::Refused { at: Position { line: 1, column: 2 }, fault: Decode::Unexpected, also: None },
    );
    assert_eq!(parent::run(&repository, &[b"x1-1-prod"], 0).outcome, Outcome::Failed);
}
#[test]
fn a_container_whose_host_failed_is_left_out_without_an_error() {
    use hearth_domain_render::{errors::Decode, revision::Position, vocabulary::Decoded};
    let mut repository = fixtures::containers();
    fixtures::set_document(
        &mut repository,
        b'4',
        Decoded::Refused { at: Position { line: 1, column: 2 }, fault: Decode::Unexpected, also: None },
    );
    for seed in 0..8 {
        let run = parent::run(&repository, &[], seed);
        let Answer::Error(error) = only(&run.answers) else { panic!("host error") };
        assert_eq!(error.kind, hearth_domain_render::errors::Kind::Decode(Decode::Unexpected), "seed {seed}");
        assert_eq!(run.outcome, Outcome::Failed, "seed {seed}");
    }
}
#[test]
fn a_selected_container_uses_its_unselected_host() {
    let repository = fixtures::containers();
    for seed in 0..8 {
        let run = parent::run(&repository, &[b"x1-prod"], seed);
        let Answer::Spec(spec) = only(&run.answers) else { panic!("selected container") };
        assert_eq!(spec.meta.target.as_ref(), b"x1-prod", "seed {seed}");
        assert_eq!(spec.meta.placement.on.as_deref(), Some(b"a-b-prod".as_slice()), "seed {seed}");
        assert_eq!(run.outcome, Outcome::Done, "seed {seed}");
    }
}
#[test]
fn a_container_whose_unselected_host_s_settings_failed_is_left_out_without_an_error() {
    let repository = fixtures::container_with_unsound_host();
    for seed in 0..8 {
        let run = parent::run(&repository, &[b"x1-prod"], seed);
        assert_eq!(run.answers, [], "seed {seed}");
        assert_eq!(run.outcome, Outcome::Failed, "seed {seed}");
    }
}
