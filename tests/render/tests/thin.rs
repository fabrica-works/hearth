use hearth_domain_render::{
    limits,
    revision::{Firmware, Position, Runs},
    vocabulary::{Answer, Decoded, Outcome},
};
use hearth_render_world::{
    fixtures::{self, only},
    parent,
};

#[test]
fn a_targets_meta_is_its_names_and_the_base_envs_placement() {
    for seed in 0..8 {
        let run = fixtures::render_seeded(&fixtures::x1_prod(), seed);
        let Answer::Spec(spec) = only(&run.answers) else { panic!("one spec: seed {seed}") };
        assert_eq!(spec.meta, fixtures::x1_prod_meta(), "render.md, 4.4: the target's meta");
        assert_eq!(
            spec.digest,
            fixtures::digest("f7796ed03afe370215abfbc0069f738892d01a5445a8c5a63f4d151c5ea977c2"),
            "seed {seed}"
        );
        assert_eq!(run.outcome, Outcome::Done, "seed {seed}");
    }
}

#[test]
fn a_base_env_without_its_environment_is_a_repository_error() {
    for seed in 0..8 {
        let mut repository = fixtures::x1_prod();
        repository.entries.retain(|entry| entry.path.as_ref() != b"envs/prod.json");
        let run = fixtures::render_seeded(&repository, seed);
        assert_eq!(
            only(&run.answers),
            &Answer::Error(fixtures::configuration_error(
                hearth_domain_render::errors::Kind::Repository(hearth_domain_render::errors::Repository::Environment {
                    env: Box::from(b"prod".as_slice())
                }),
                b"hosts/x1/x1-prod.json",
                None,
            )),
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
        let run = fixtures::render_seeded(&repository, seed);
        let Answer::Spec(spec) = only(&run.answers) else { panic!("one spec: seed {seed}") };
        assert_eq!(spec.meta.placement, fixtures::x1_prod_meta().placement, "seed {seed}");
    }
}

#[test]
fn a_named_target_not_discovered_ends_the_call_with_usage() {
    for seed in 0..8 {
        let run = fixtures::render_targets(&fixtures::x1_prod(), &[b"unknown"], seed);
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
        let run = fixtures::render_seeded(&repository, seed);
        let survivor = fixtures::survivor(&run.answers);
        let (spec, error) = (survivor.spec, survivor.error);
        assert_eq!(error.target, None);
        assert_eq!(spec.meta.target.as_ref(), b"a-b-prod", "seed {seed}");
        assert_eq!(run.outcome, Outcome::Failed, "seed {seed}");
    }
}

#[test]
fn targets_are_answered_in_name_order_each_once() {
    for seed in 0..8 {
        let run = fixtures::render_targets(&fixtures::two_targets(), &[b"x1-prod", b"a-b-prod", b"x1-prod"], seed);
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
        let blobs = fixtures::read_blobs(&run.requests);
        assert_eq!(blobs.len(), 4, "shared environment read once, seed {seed}");
    }
}

#[test]
fn a_document_above_its_limit_fails_only_its_targets_before_reading() {
    let mut repository = fixtures::two_targets();
    repository.entries.iter_mut().find(|entry| entry.oid == fixtures::oid(b'3')).expect("host entry").size =
        u64::from(limits::SHIPPED.document) + 1;
    let run = fixtures::render(&repository);
    let survivor = fixtures::survivor(&run.answers);
    let (spec, error) = (survivor.spec, survivor.error);
    assert_eq!(error.target, None);
    assert_eq!(spec.meta.target.as_ref(), b"a-b-prod");
    assert_eq!(run.outcome, Outcome::Failed);
    let blobs = fixtures::read_blobs(&run.requests);
    assert_eq!(
        blobs.iter().map(|b| b.oid.clone()).collect::<Vec<_>>(),
        [fixtures::oid(b'1'), fixtures::oid(b'4'), fixtures::oid(b'2')]
    );
}

#[test]
fn targets_above_the_configured_limit_fail_before_reading() {
    use hearth_domain_render::errors::{Decode, Kind, Limit};
    let run = parent::run_with(&fixtures::two_targets(), &[], 0, limits::Limits { targets: 1, ..limits::SHIPPED });
    assert_eq!(
        run.answers,
        [Answer::Error(fixtures::configuration_error(
            Kind::Decode(Decode::TooLarge { limit: Limit::Targets, max: 1 }),
            b"hosts/x1/x1-prod.json",
            None,
        ))]
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
    let run = fixtures::render(&repository);
    let survivor = fixtures::survivor(&run.answers);
    let (spec, error) = (survivor.spec, survivor.error);
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
    let run = fixtures::render(&repository);
    assert_eq!(
        only(&run.answers),
        &Answer::Error(fixtures::configuration_error(
            hearth_domain_render::errors::Kind::Repository(hearth_domain_render::errors::Repository::Submodule),
            b"tasks/motd/tree/etc/submodule",
            Some(b"motd"),
        ))
    );
    assert_eq!(run.outcome, Outcome::Failed);
}
#[test]
fn a_template_and_a_link_are_left_out_of_the_tree() {
    let repository = fixtures::template_and_link();
    let run = fixtures::render(&repository);
    let Answer::Spec(spec) = only(&run.answers) else { panic!("one target") };
    let config = fixtures::config_part(&spec.parts);
    assert_eq!(
        config.tree.as_ref(),
        &[
            hearth_domain_render::parts::config::TreeEntry {
                path: b"/etc/link".as_slice().into(),
                node: hearth_domain_render::parts::config::Node::Link {
                    target: b"/target".as_slice().into(),
                    owner: b"root".as_slice().into(),
                    group: b"root".as_slice().into()
                }
            },
            hearth_domain_render::parts::config::TreeEntry {
                path: b"/etc/template".as_slice().into(),
                node: hearth_domain_render::parts::config::Node::File {
                    content_digest: fixtures::digest(
                        "2689367b205c16ce32ed4200942b8b8b1e262dfc70d9bc9fbc77c49699a4f1df"
                    ),
                    mode: 0o644,
                    owner: b"root".as_slice().into(),
                    group: b"root".as_slice().into(),
                },
            }
        ]
    );
}

#[test]
fn the_tree_orders_by_installed_path_when_the_suffix_reorders() {
    let repository = fixtures::tree_templates(&[
        (b"tasks/motd/tree/etc/a-b", hearth_domain_render::revision::Mode::File, b"ab"),
        (b"tasks/motd/tree/etc/a.j2", hearth_domain_render::revision::Mode::File, b"a"),
    ]);
    let run = fixtures::render(&repository);
    let Answer::Spec(spec) = only(&run.answers) else { panic!("one spec") };
    let config = hearth_render_world::fixtures::config_part(&spec.parts);
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
    let run = fixtures::render(&repository);
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
    let run = fixtures::render(&fixtures::templates_for_many_targets());
    assert_eq!(run.answers.len(), 9);
}

#[test]
fn a_tree_link_shares_the_read_and_a_template_adds_a_keep() {
    assert_eq!(
        fixtures::configuration_requests(&fixtures::render(&fixtures::template_and_link()).requests).len(),
        5,
        "the rendered template and render-owned initramfs add their Keep requests"
    );
}
