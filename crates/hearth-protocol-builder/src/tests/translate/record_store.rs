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
