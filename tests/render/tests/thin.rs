use hearth_domain_render::{
    limits,
    revision::{Firmware, Position, Runs},
    vocabulary::{Answer, Decoded, Outcome, Request},
};
use hearth_render_world::{
    fixtures::{self, Repository},
    parent,
};

#[test]
fn a_targets_meta_is_its_names_and_the_base_envs_placement() {
    for seed in 0..8 {
        let run = parent::run(&fixtures::x1_prod(), &[], seed);
        let [Answer::Spec(spec)] = &run.answers[..] else { panic!("one spec: seed {seed}") };
        assert_eq!(spec.meta, fixtures::x1_prod_meta(), "render.md, 4.4: the target's meta");
        assert_eq!(
            spec.digest,
            fixtures::digest("d5942fa1a047fd65c5731c85cfba607e81d0fd16cb00f154aacfa79c44ed118c"),
            "seed {seed}"
        );
        assert_eq!(run.outcome, Outcome::Done, "seed {seed}");
    }
}

#[test]
fn render_lists_once_then_reads_each_targets_documents_once() {
    for seed in 0..8 {
        let run = parent::run(&fixtures::x1_prod(), &[], seed);
        let [Request::List { .. }, Request::Read { blobs, .. }] = &run.requests[..] else {
            panic!("list then read, seed {seed}")
        };
        assert_eq!(
            blobs.iter().map(|b| b.oid.clone()).collect::<Vec<_>>(),
            [fixtures::oid(b'1'), fixtures::oid(b'3')],
            "no base read, seed {seed}"
        );
    }
}

#[test]
fn a_base_env_without_its_environment_makes_no_target() {
    for seed in 0..8 {
        let mut repository = fixtures::x1_prod();
        repository.entries.remove(0);
        let run = parent::run(&repository, &[], seed);
        assert_eq!(run.answers, [], "seed {seed}");
        assert_eq!(run.outcome, Outcome::Done, "seed {seed}");
    }
}

#[test]
fn the_base_envs_placement_replaces_the_environments_whole() {
    for seed in 0..8 {
        let mut repository = fixtures::x1_prod();
        let mut placement = fixtures::placement();
        placement.kind = Runs::Vm;
        placement.firmware = Some(Firmware::Bios);
        placement.hardware = Some(Box::from(b"elsewhere".as_slice()));
        repository.documents[0].1 = fixtures::document(Some(placement));
        let run = parent::run(&repository, &[], seed);
        let [Answer::Spec(spec)] = &run.answers[..] else { panic!("one spec: seed {seed}") };
        assert_eq!(spec.meta.placement, fixtures::x1_prod_meta().placement, "seed {seed}");
    }
}

#[test]
fn a_named_target_not_discovered_ends_the_call_with_usage() {
    for seed in 0..8 {
        let run = parent::run(&fixtures::x1_prod(), &[b"unknown"], seed);
        assert_eq!(run.answers, [], "seed {seed}");
        assert_eq!(run.outcome, Outcome::Usage, "seed {seed}");
    }
}

fn two_targets() -> Repository {
    let mut repository = fixtures::x1_prod();
    repository.entries.push(fixtures::entry(b"hosts/a-b/a-b-prod.json", b'4'));
    repository.documents.push((fixtures::oid(b'4'), fixtures::document(Some(fixtures::placement()))));
    repository
}

#[test]
fn a_refused_document_fails_its_targets_and_spares_the_rest() {
    for seed in 0..8 {
        let mut repository = two_targets();
        repository.documents[2].1 = Decoded::Refused { at: Position { line: 2, column: 3 } };
        let run = parent::run(&repository, &[], seed);
        let [Answer::Spec(spec)] = &run.answers[..] else { panic!("one surviving spec: seed {seed}") };
        assert_eq!(spec.meta.target.as_ref(), b"a-b-prod", "seed {seed}");
        assert_eq!(run.outcome, Outcome::Failed, "seed {seed}");
    }
}

#[test]
fn targets_are_answered_in_name_order_each_once() {
    for seed in 0..8 {
        let run = parent::run(&two_targets(), &[b"x1-prod", b"a-b-prod", b"x1-prod"], seed);
        let names: Vec<&[u8]> = run
            .answers
            .iter()
            .map(|answer| match answer {
                Answer::Spec(spec) => spec.meta.target.as_ref(),
                Answer::Rendered { .. } => panic!("terminal is separate"),
            })
            .collect();
        assert_eq!(names, [b"a-b-prod".as_slice(), b"x1-prod"], "prefix-based discovery, seed {seed}");
        assert_eq!(run.outcome, Outcome::Done, "seed {seed}");
        let Request::Read { blobs, .. } = &run.requests[1] else { panic!("read follows list") };
        assert_eq!(blobs.len(), 3, "shared environment read once, seed {seed}");
    }
}

#[test]
fn a_document_above_its_limit_fails_only_its_targets_before_reading() {
    let mut repository = two_targets();
    repository.entries[2].size = u64::from(limits::SHIPPED.document) + 1;
    let run = parent::run(&repository, &[], 0);
    let [Answer::Spec(spec)] = &run.answers[..] else { panic!("one surviving target") };
    assert_eq!(spec.meta.target.as_ref(), b"a-b-prod");
    assert_eq!(run.outcome, Outcome::Failed);
    let Request::Read { blobs, .. } = &run.requests[1] else { panic!("read follows list") };
    assert_eq!(blobs.iter().map(|b| b.oid.clone()).collect::<Vec<_>>(), [fixtures::oid(b'1'), fixtures::oid(b'4')]);
}

#[test]
fn targets_above_the_configured_limit_fail_before_reading() {
    let run = parent::run_with(&two_targets(), &[], 0, limits::Limits { targets: 1, ..limits::SHIPPED });
    assert_eq!(run.answers, []);
    assert_eq!(run.outcome, Outcome::Failed);
    assert_eq!(run.requests.len(), 1);
}
