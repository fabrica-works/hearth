use hearth_render_world::schemas;
use skein_lib::Rng;
#[test]
fn generated_settings_validate_as_the_reference_validates_them() {
    for seed in 0..200_u64 {
        let mut rng = Rng::new(0x0013_0000 + seed);
        let declaration = schemas::declaration(&mut rng, schemas::SMALL);
        let settings = schemas::settings(&mut rng, &declaration);
        assert_eq!(
            schemas::validate(&declaration, &settings),
            schemas::reference(&declaration, &settings),
            "seed {seed}"
        );
    }
}

use hearth_domain_render::{
    errors::{Error, Kind, Place, Schema},
    revision::{Kind as Written, Position},
    vocabulary::{Answer, Class},
};
use hearth_render_world::{fixtures, parent};
#[test]
fn a_typo_in_a_default_for_a_task_no_target_has_is_refused() {
    for seed in 0..8 {
        let run = parent::run(&fixtures::unused_object_typo(), &[], seed);
        let [Answer::Spec(_), Answer::Error(error)] = run.answers.as_slice() else { panic!("spec and revision fault") };
        assert_eq!(
            error,
            &Error {
                side: None,
                kind: Kind::Schema(Schema::UndeclaredField {
                    field: Box::from(b"typo".as_slice()),
                    declared: Box::new([Box::from(b"x".as_slice())])
                }),
                class: Class::Configuration,
                at: Some(Place {
                    path: Box::from(b"tasks/motd/task.json".as_slice()),
                    position: Some(Position { line: 2, column: 7 })
                }),
                also: None,
                target: None,
                task: Some(Box::from(b"motd".as_slice()))
            },
            "seed {seed}"
        );
    }
}
#[test]
fn an_environment_s_setting_of_a_task_only_other_targets_have_is_checked_alone() {
    for seed in 0..8 {
        let mut repository = fixtures::with_int_setting();
        repository.entries.push(fixtures::entry(b"envs/lab.json", b'5'));
        repository.documents.push((
            fixtures::oid(b'5'),
            fixtures::task_patch(b"motd", b"x", Written::String(Box::from(b"wrong".as_slice()))),
        ));
        let run = parent::run(&repository, &[], seed);
        let [Answer::Spec(_), Answer::Error(error)] = run.answers.as_slice() else {
            panic!("spec and unclaimed fault")
        };
        assert_eq!(
            error,
            &Error {
                side: None,
                kind: Kind::Schema(Schema::Wrong { expected: hearth_domain_render::errors::TypeName::Int }),
                class: Class::Configuration,
                at: Some(Place {
                    path: Box::from(b"envs/lab.json".as_slice()),
                    position: Some(Position { line: 2, column: 7 })
                }),
                also: None,
                target: None,
                task: Some(Box::from(b"motd".as_slice()))
            },
            "seed {seed}"
        );
    }
}
#[test]
fn one_target_s_schema_fault_spares_the_other_targets() {
    for seed in 0..8 {
        let mut repository = fixtures::with_int_setting();
        repository.entries.push(fixtures::entry(b"hosts/a-b/a-b-prod.json", b'5'));
        repository.documents.push((fixtures::oid(b'5'), fixtures::document(Some(fixtures::placement()))));
        fixtures::set_document(
            &mut repository,
            b'2',
            fixtures::task_patch(b"motd", b"x", Written::String(Box::from(b"wrong".as_slice()))),
        );
        let run = parent::run(&repository, &[], seed);
        let [Answer::Spec(spec), Answer::Error(error)] = run.answers.as_slice() else {
            panic!("unaffected spec and target fault")
        };
        assert_eq!(
            (spec.meta.target.as_ref(), error.target.as_deref()),
            (b"a-b-prod".as_slice(), Some(b"x1-prod".as_slice())),
            "seed {seed}"
        );
    }
}
#[test]
fn validated_bytes_past_the_budget_answer_an_unpositioned_target_limit() {
    let mut repository = fixtures::with_setting(hearth_domain_render::revision::declaration::Form::String);
    fixtures::set_document(
        &mut repository,
        b'2',
        fixtures::task_patch(b"motd", b"x", Written::String(Box::from(b"abcdefghijklmnopqrstuvwxyz012345".as_slice()))),
    );
    let run = parent::run_with(
        &repository,
        &[],
        0,
        hearth_domain_render::limits::Limits { validated: 64, ..hearth_domain_render::limits::SHIPPED },
    );
    assert_eq!(
        run.answers.as_slice(),
        &[Answer::Error(Error {
            side: None,
            kind: Kind::Evaluation(hearth_domain_render::evaluate::FaultKind::Limit),
            class: Class::Configuration,
            at: None,
            also: None,
            target: Some(Box::from(b"x1-prod".as_slice())),
            task: None
        })]
    );
}

#[test]
fn validated_bytes_are_returned_when_a_target_is_answered() {
    let mut repository = fixtures::with_int_setting();
    repository.entries.push(fixtures::entry(b"envs/lab.json", b'5'));
    repository.entries.push(fixtures::entry(b"hosts/x1/x1-lab.json", b'6'));
    repository.documents.push((fixtures::oid(b'5'), fixtures::document(None)));
    repository.documents.push((fixtures::oid(b'6'), fixtures::document(Some(fixtures::placement()))));
    let run = parent::run_with(
        &repository,
        &[],
        0,
        hearth_domain_render::limits::Limits { validated: 64, ..hearth_domain_render::limits::SHIPPED },
    );
    let names: Vec<_> = run
        .answers
        .iter()
        .map(|answer| {
            let Answer::Spec(spec) = answer else { panic!("spec") };
            spec.meta.target.as_ref()
        })
        .collect();
    assert_eq!(names, &[b"x1-lab".as_slice(), b"x1-prod".as_slice()]);
}
