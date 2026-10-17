use crate::render::evaluate::FaultKind;
const EVALUATIONS: &[FaultKind] = &[
    FaultKind::Unknown,
    FaultKind::NoKey,
    FaultKind::NoIndex,
    FaultKind::Undefined,
    FaultKind::Type,
    FaultKind::Overflow,
    FaultKind::Empty,
    FaultKind::Path,
    FaultKind::Output,
    FaultKind::Limit,
];
#[test]
fn each_evaluation_kind_translates_and_back() {
    for (row, &input) in EVALUATIONS.iter().enumerate() {
        let record = crate::translate::errors::evaluation(input);
        let back = crate::tests::fixtures::evaluation_back(record);
        assert_eq!(back, input, "row {row}: evaluation");
    }
}

#[test]
fn an_evaluation_error_translates_without_a_place() {
    use crate::{
        records::errors as records,
        render::{errors as render, vocabulary::Class},
    };
    assert_eq!(
        crate::translate::errors::error(render::Error {
            side: None,
            kind: render::Kind::Evaluation(FaultKind::Limit),
            class: Class::Configuration,
            at: None,
            also: None,
            target: Some(skein_lib::bytes::copy_of(b"x1-prod")),
            task: None,
        }),
        records::Error {
            side: None,
            kind: records::Kind::Evaluation(records::Evaluation::Limit),
            class: records::Class::Configuration,
            at: None,
            also: None,
            target: Some(skein_lib::bytes::copy_of(b"x1-prod")),
            task: None,
        }
    );
}
