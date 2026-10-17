use crate::{records::errors as records, render::errors as render, tests::fixtures};
fn cause_back(cause: records::Cause) -> render::Cause {
    match cause {
        records::Cause::Decode(records::Decode::Unexpected) => render::Cause::Decode(render::Decode::Unexpected),
        records::Cause::Repository(records::Repository::Stray) => render::Cause::Repository(render::Repository::Stray),
        records::Cause::Composition(records::Composition::Null) => {
            render::Cause::Composition(render::Composition::Null)
        }
        records::Cause::Schema(records::Schema::Main) => render::Cause::Schema(render::Schema::Main),
        records::Cause::Evaluation(value) => render::Cause::Evaluation(fixtures::evaluation_back(value)),
        value @ (records::Cause::Decode(_)
        | records::Cause::Repository(_)
        | records::Cause::Composition(_)
        | records::Cause::Schema(_)) => panic!("unlisted fixture cause: {value:?}"),
    }
}
#[test]
fn each_exported_cause_translates_to_the_records_and_back() {
    for (row, cause) in fixtures::EXPORTED_CAUSES.iter().enumerate() {
        let input = fixtures::exported_error(cause.clone());
        let records::Error { side, kind, class: _, at: _, also: _, target, task } =
            crate::translate::errors::error(input.clone());
        let records::Kind::Exported(records::Exported { exporter, task: exporting, cause }) = kind else {
            panic!("exported")
        };
        let mut translated_side = None;
        if let Some(side) = side {
            translated_side = Some(super::side_back(side));
        }
        let back = render::Error {
            side: translated_side,
            kind: render::Kind::Exported(render::Exported { exporter, task: exporting, cause: cause_back(cause) }),
            class: crate::render::vocabulary::Class::Configuration,
            at: None,
            also: None,
            target,
            task,
        };
        assert_eq!(back, input, "row {row}: exported cause");
    }
}
