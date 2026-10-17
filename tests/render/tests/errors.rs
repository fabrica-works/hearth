use hearth_domain_render::{
    errors::{Decode, Error, Kind, Place},
    revision::Position,
    vocabulary::{Answer, Class, Decoded, Outcome},
};
use hearth_render_world::{fixtures, parent};
#[test]
fn a_broken_document_answers_one_error_and_spares_other_targets() {
    for seed in 0..8 {
        let mut repository = fixtures::two_targets();
        repository.entries.push(fixtures::entry(b"hosts/x1/x1-other.json", b'5'));
        repository.entries.push(fixtures::entry(b"envs/other.json", b'6'));
        repository.documents.push((fixtures::oid(b'5'), fixtures::document(Some(fixtures::placement()))));
        repository.documents.push((fixtures::oid(b'6'), fixtures::document(None)));
        fixtures::set_document(
            &mut repository,
            b'2',
            Decoded::Refused { at: Position { line: 2, column: 3 }, fault: Decode::Unexpected, also: None },
        );
        let run = parent::run(&repository, &[], seed);
        let [Answer::Spec(spec), Answer::Error(error)] = run.answers.as_slice() else {
            panic!("one spec then one error")
        };
        assert_eq!(spec.meta.target.as_ref(), b"a-b-prod");
        assert_eq!(
            error,
            &Error {
                kind: Kind::Decode(Decode::Unexpected),
                class: Class::Configuration,
                at: Some(Place {
                    path: Box::from(b"hosts/x1/x1.json".as_slice()),
                    position: Some(Position { line: 2, column: 3 })
                }),
                also: None,
                target: None,
                task: None
            }
        );
        assert_eq!(run.outcome, Outcome::Failed, "seed {seed}");
    }
}

#[test]
fn a_broken_task_json_no_target_has_prints_once_and_fails_nothing() {
    for seed in 0..8 {
        let mut repository = fixtures::x1_prod();
        repository.entries.push(fixtures::entry(b"tasks/unused/task.json", b'7'));
        repository.documents.push((
            fixtures::oid(b'7'),
            Decoded::Refused {
                at: Position { line: 1, column: 2 },
                fault: Decode::Declaration(hearth_domain_render::errors::Declared::Unknown {
                    of: hearth_domain_render::errors::Object::Declaration,
                }),
                also: None,
            },
        ));
        let run = parent::run(&repository, &[], seed);
        let [Answer::Spec(spec), Answer::Error(error)] = run.answers.as_slice() else {
            panic!("one spec then one error")
        };
        assert_eq!(spec.meta.target.as_ref(), b"x1-prod", "seed {seed}");
        assert_eq!(
            error,
            &Error {
                kind: Kind::Decode(Decode::Declaration(hearth_domain_render::errors::Declared::Unknown {
                    of: hearth_domain_render::errors::Object::Declaration
                })),
                class: Class::Configuration,
                at: Some(Place {
                    path: Box::from(b"tasks/unused/task.json".as_slice()),
                    position: Some(Position { line: 1, column: 2 })
                }),
                also: None,
                target: None,
                task: Some(Box::from(b"unused".as_slice())),
            },
            "seed {seed}"
        );
    }
}
