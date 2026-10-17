use crate::{records::errors as target, render::errors as source};
const STEPS: &[source::Step] = &[
    source::Step::List,
    source::Step::Remove,
    source::Step::Load,
    source::Step::Empty,
    source::Step::Keep,
    source::Step::Write,
];
#[test]
fn each_store_step_translates_and_back() {
    for (row, &input) in STEPS.iter().enumerate() {
        let error = crate::translate::errors::error(source::Error {
            side: None,
            kind: source::Kind::Store(source::StoreError {
                store: b"/store".as_slice().into(),
                fault: source::StoreFault::Failed { step: input },
            }),
            class: crate::render::vocabulary::Class::Operational,
            at: None,
            also: None,
            target: None,
            task: None,
        });
        let target::Kind::Store(target::StoreError { fault: target::StoreFault::Failed { step }, .. }) = error.kind
        else {
            panic!("store failure")
        };
        let back = match step {
            target::Step::List => source::Step::List,
            target::Step::Remove => source::Step::Remove,
            target::Step::Load => source::Step::Load,
            target::Step::Empty => source::Step::Empty,
            target::Step::Keep => source::Step::Keep,
            target::Step::Write => source::Step::Write,
        };
        assert_eq!(back, input, "row {row}: store step");
    }
}

const RESOLUTION_PROGRESS: &[(crate::builder::vocabulary::Progress, crate::records::records::Record)] = &[
    (
        crate::builder::vocabulary::Progress::Resolving { key: crate::tests::fixtures::KEY },
        crate::records::records::Record::Resolving(crate::records::progress::Resolving { key: [1; 32] }),
    ),
    (
        crate::builder::vocabulary::Progress::Resolved {
            key: crate::tests::fixtures::KEY,
            took: skein_lib::Duration::from_millis(7),
        },
        crate::records::records::Record::Resolved(crate::records::progress::Resolved {
            key: [1; 32],
            took: skein_lib::Duration::from_millis(7),
        }),
    ),
    (
        crate::builder::vocabulary::Progress::Unresolved { key: crate::tests::fixtures::KEY },
        crate::records::records::Record::Unresolved(crate::records::progress::Unresolved { key: [1; 32] }),
    ),
];
#[test]
fn resolution_progress_keeps_its_key_and_duration() {
    for (row, (input, expected)) in RESOLUTION_PROGRESS.iter().enumerate() {
        assert_eq!(&crate::translate::store::progress(input.clone(), b"/store"), expected, "row {row}");
    }
}
