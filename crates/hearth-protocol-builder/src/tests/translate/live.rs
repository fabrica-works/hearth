//! Live records translated exhaustively (render.md, 4.4).
use crate::{records::parts::live as source, render::parts::live as target};
#[must_use]
pub(super) fn back(live: source::Live) -> target::Live {
    let source::Live { paths } = live;
    let mut covered = skein_lib::List::with_capacity(u32::try_from(paths.len()).expect("path count"));
    for source::Covered { path, consumers } in paths {
        covered
            .push(target::Covered { path, consumers: crate::tests::fixtures::consumers_back(consumers) })
            .expect("path count");
    }
    target::Live { paths: covered.into_boxed() }
}

#[test]
fn each_live_field_translates_to_the_records_and_back() {
    for (row, input) in super::super::fixtures::live_records().into_iter().enumerate() {
        let translated = crate::translate::parts::live::live(input.clone());
        assert_eq!(back(translated), input, "row {row}: every live field");
    }
}
