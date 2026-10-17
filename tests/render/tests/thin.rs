use hearth_domain_render::{
    limits,
    revision::{Firmware, Position, Runs},
    vocabulary::{Answer, Decoded, Outcome, Request},
};
use hearth_render_world::{
    fixtures::{self, only},
    parent,
};

#[test]
fn a_targets_meta_is_its_names_and_the_base_envs_placement() {
    for seed in 0..8 {
        let run = parent::run(&fixtures::x1_prod(), &[], seed);
        let Answer::Spec(spec) = only(&run.answers) else { panic!("one spec: seed {seed}") };
        assert_eq!(spec.meta, fixtures::x1_prod_meta(), "render.md, 4.4: the target's meta");
        assert_eq!(
            spec.digest,
            fixtures::digest("e0d142ee5fb0ee79d5e273e4a7839d6201f9134a3abc48206432a3008aa6a009"),
            "seed {seed}"
        );
        assert_eq!(run.outcome, Outcome::Done, "seed {seed}");
    }
}

#[test]
fn a_base_env_without_its_environment_is_a_repository_error() {
    for seed in 0..8 {
        let mut repository = fixtures::x1_prod();
        repository.entries.remove(0);
        let run = parent::run(&repository, &[], seed);
        assert_eq!(
            only(&run.answers),
            &Answer::Error(hearth_domain_render::errors::Error {
                side: None,
                kind: hearth_domain_render::errors::Kind::Repository(
                    hearth_domain_render::errors::Repository::Environment { env: Box::from(b"prod".as_slice()) }
                ),
                class: hearth_domain_render::vocabulary::Class::Configuration,
                at: Some(hearth_domain_render::errors::Place {
                    path: Box::from(b"hosts/x1/x1-prod.json".as_slice()),
                    position: None
                }),
                also: None,
                target: None,
                task: None
            }),
            "seed {seed}"
        );
        assert_eq!(run.outcome, Outcome::Failed, "seed {seed}");
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
        fixtures::set_document(&mut repository, b'1', fixtures::document(Some(placement)));
        let run = parent::run(&repository, &[], seed);
        let Answer::Spec(spec) = only(&run.answers) else { panic!("one spec: seed {seed}") };
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

#[test]
fn a_refused_document_fails_its_targets_and_spares_the_rest() {
    for seed in 0..8 {
        let mut repository = fixtures::two_targets();
        fixtures::set_document(
            &mut repository,
            b'3',
            Decoded::Refused {
                at: Position { line: 2, column: 3 },
                fault: hearth_domain_render::errors::Decode::Unexpected,
                also: None,
            },
        );
        let run = parent::run(&repository, &[], seed);
        let [Answer::Spec(spec), Answer::Error(error)] = run.answers.as_slice() else {
            panic!("one surviving spec: seed {seed} and its error")
        };
        assert_eq!(error.target, None);
        assert_eq!(spec.meta.target.as_ref(), b"a-b-prod", "seed {seed}");
        assert_eq!(run.outcome, Outcome::Failed, "seed {seed}");
    }
}

#[test]
fn targets_are_answered_in_name_order_each_once() {
    for seed in 0..8 {
        let run = parent::run(&fixtures::two_targets(), &[b"x1-prod", b"a-b-prod", b"x1-prod"], seed);
        let names: Vec<&[u8]> = run
            .answers
            .iter()
            .map(|answer| match answer {
                Answer::Spec(spec) => spec.meta.target.as_ref(),
                Answer::Rendered { .. } => panic!("terminal is separate"),
                Answer::Error(_) | Answer::Change(_) => panic!("render returns specs"),
            })
            .collect();
        assert_eq!(names, [b"a-b-prod".as_slice(), b"x1-prod"], "prefix-based discovery, seed {seed}");
        assert_eq!(run.outcome, Outcome::Done, "seed {seed}");
        let Request::Read { blobs, .. } =
            run.requests.iter().find(|rq| matches!(rq, Request::Read { .. })).expect("read request")
        else {
            panic!("read follows list")
        };
        assert_eq!(blobs.len(), 4, "shared environment read once, seed {seed}");
    }
}

#[test]
fn a_document_above_its_limit_fails_only_its_targets_before_reading() {
    let mut repository = fixtures::two_targets();
    repository.entries.iter_mut().find(|entry| entry.oid == fixtures::oid(b'3')).expect("host entry").size =
        u64::from(limits::SHIPPED.document) + 1;
    let run = parent::run(&repository, &[], 0);
    let [Answer::Spec(spec), Answer::Error(error)] = run.answers.as_slice() else {
        panic!("one surviving target and its error")
    };
    assert_eq!(error.target, None);
    assert_eq!(spec.meta.target.as_ref(), b"a-b-prod");
    assert_eq!(run.outcome, Outcome::Failed);
    let Request::Read { blobs, .. } =
        run.requests.iter().find(|rq| matches!(rq, Request::Read { .. })).expect("read request")
    else {
        panic!("read follows list")
    };
    assert_eq!(
        blobs.iter().map(|b| b.oid.clone()).collect::<Vec<_>>(),
        [fixtures::oid(b'1'), fixtures::oid(b'4'), fixtures::oid(b'2')]
    );
}

#[test]
fn targets_above_the_configured_limit_fail_before_reading() {
    use hearth_domain_render::errors::{Decode, Error, Kind, Limit, Place};
    let run = parent::run_with(&fixtures::two_targets(), &[], 0, limits::Limits { targets: 1, ..limits::SHIPPED });
    assert_eq!(
        run.answers,
        [Answer::Error(Error {
            side: None,
            kind: Kind::Decode(Decode::TooLarge { limit: Limit::Targets, max: 1 }),
            class: hearth_domain_render::vocabulary::Class::Configuration,
            at: Some(Place { path: Box::from(b"hosts/x1/x1-prod.json".as_slice()), position: None }),
            also: None,
            target: None,
            task: None,
        })]
    );
    assert_eq!(run.outcome, Outcome::Failed);
    assert_eq!(run.requests.len(), 2);
}

#[test]
fn a_malformed_task_json_fails_the_targets_that_have_the_task() {
    let mut repository = fixtures::with_task();
    fixtures::set_document(
        &mut repository,
        b'4',
        Decoded::Refused {
            at: Position { line: 1, column: 2 },
            fault: hearth_domain_render::errors::Decode::Unexpected,
            also: None,
        },
    );
    repository.entries.push(fixtures::entry(b"hosts/a/a-prod.json", b'5'));
    repository.documents.push((fixtures::oid(b'5'), fixtures::document(Some(fixtures::placement()))));
    let run = parent::run(&repository, &[], 0);
    let [Answer::Spec(spec), Answer::Error(error)] = run.answers.as_slice() else {
        panic!("one unaffected target and its error")
    };
    assert_eq!(error.target, None);
    assert_eq!(spec.meta.target.as_ref(), b"a-prod");
    assert_eq!(run.outcome, Outcome::Failed);
}
#[test]
fn a_submodule_under_a_present_tasks_tree_fails_its_target() {
    let mut repository = fixtures::with_task();
    let mut submodule = fixtures::entry(b"tasks/motd/tree/etc/submodule", b'5');
    submodule.mode = hearth_domain_render::revision::Mode::Submodule;
    repository.entries.push(submodule);
    let run = parent::run(&repository, &[], 0);
    assert_eq!(
        only(&run.answers),
        &Answer::Error(hearth_domain_render::errors::Error {
            side: None,
            kind: hearth_domain_render::errors::Kind::Repository(hearth_domain_render::errors::Repository::Submodule),
            class: hearth_domain_render::vocabulary::Class::Configuration,
            at: Some(hearth_domain_render::errors::Place {
                path: Box::from(b"tasks/motd/tree/etc/submodule".as_slice()),
                position: None
            }),
            also: None,
            target: None,
            task: Some(Box::from(b"motd".as_slice()))
        })
    );
    assert_eq!(run.outcome, Outcome::Failed);
}
#[test]
fn a_template_and_a_link_are_left_out_of_the_tree() {
    let mut repository = fixtures::with_task();
    repository.entries.push(fixtures::entry(b"tasks/motd/tree/etc/template.j2", b'5'));
    repository.blobs.push((fixtures::oid(b'5'), Box::from(b"ok".as_slice())));
    let mut link = fixtures::entry(b"tasks/motd/tree/etc/link", b'6');
    link.mode = hearth_domain_render::revision::Mode::Link;
    repository.entries.push(link);
    let run = parent::run(&repository, &[], 0);
    let Answer::Spec(spec) = only(&run.answers) else { panic!("one target") };
    let part = only(&spec.parts);
    let hearth_domain_render::parts::PartRecord::Config(config) = &part.record;
    assert_eq!(
        config.tree.as_ref(),
        &[hearth_domain_render::parts::config::TreeEntry {
            path: b"/etc/template".as_slice().into(),
            node: hearth_domain_render::parts::config::Node::File {
                content_digest: fixtures::digest("2689367b205c16ce32ed4200942b8b8b1e262dfc70d9bc9fbc77c49699a4f1df"),
                mode: 0o644,
                owner: b"root".as_slice().into(),
                group: b"root".as_slice().into(),
            },
        }]
    );
    assert_eq!(run.requests.len(), 4, "the rendered template adds its Keep request");
}

#[test]
fn the_tree_orders_by_installed_path_when_the_suffix_reorders() {
    let repository = fixtures::tree_templates(&[
        (b"tasks/motd/tree/etc/a-b", hearth_domain_render::revision::Mode::File, b"ab"),
        (b"tasks/motd/tree/etc/a.j2", hearth_domain_render::revision::Mode::File, b"a"),
    ]);
    let run = parent::run(&repository, &[], 0);
    let Answer::Spec(spec) = only(&run.answers) else { panic!("one spec") };
    let hearth_domain_render::parts::PartRecord::Config(config) = &only(&spec.parts).record;
    assert_eq!(
        config.tree.iter().map(|entry| entry.path.as_ref()).collect::<Vec<_>>(),
        [b"/etc/a".as_slice(), b"/etc/a-b"]
    );
}

const FAULTING_TEMPLATES: &[(&[u8], hearth_domain_render::revision::Mode, &[u8])] = &[
    (b"tasks/motd/tree/etc/a.j2", hearth_domain_render::revision::Mode::File, b"{{ target.nam }}{{ target.no }}"),
    (
        b"tasks/motd/tree/etc/b.j2",
        hearth_domain_render::revision::Mode::File,
        b"header\n{{ target.zz }}{{ target.no }}",
    ),
];
const FIRST_FAULTS: &[(&[u8], Position, hearth_domain_render::errors::Kind)] = &[
    (
        b"tasks/motd/tree/etc/a.j2",
        Position { line: 1, column: 11 },
        hearth_domain_render::errors::Kind::Evaluation(hearth_domain_render::evaluate::FaultKind::NoKey),
    ),
    (
        b"tasks/motd/tree/etc/b.j2",
        Position { line: 2, column: 11 },
        hearth_domain_render::errors::Kind::Evaluation(hearth_domain_render::evaluate::FaultKind::NoKey),
    ),
];
#[test]
fn each_template_runs_after_a_fault_and_gives_its_first_fault() {
    let repository = fixtures::tree_templates(FAULTING_TEMPLATES);
    let run = parent::run(&repository, &[], 0);
    let errors: Vec<_> = run
        .answers
        .iter()
        .map(|answer| {
            let Answer::Error(error) = answer else { panic!("evaluation error") };
            let place = error.at.as_ref().expect("template place");
            (place.path.as_ref(), place.position.expect("fault position"), error.kind.clone())
        })
        .collect();
    assert_eq!(errors, FIRST_FAULTS);
}

#[test]
fn a_shared_templates_different_outputs_fit_for_each_target() {
    let run = parent::run(&fixtures::templates_for_many_targets(), &[], 0);
    assert_eq!(run.answers.len(), 9);
}
