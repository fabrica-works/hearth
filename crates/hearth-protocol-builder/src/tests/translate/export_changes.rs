use crate::{records::diff as records, render::parts::exports as render, tests::fixtures, translate::changes};
use skein_lib::List;
pub(super) fn change_back(change: records::ExportsChange) -> render::Change {
    let mut entries = List::with_capacity(u32::try_from(change.entries.len()).expect("fixture count"));
    for entry in change.entries {
        entries
            .push(render::ExportChange {
                task: entry.task,
                name: entry.name,
                before: optional_back(entry.before),
                after: optional_back(entry.after),
            })
            .expect("fixture count");
    }
    render::Change { entries: entries.into_boxed() }
}
#[test]
fn each_exports_change_translates_and_back() {
    for (row, input) in fixtures::export_changes().into_iter().enumerate() {
        let records::PartChange::Exports(change) =
            changes::part_change(crate::render::diff::PartChange::Exports(input.clone()))
        else {
            panic!("exports")
        };
        let back = change_back(change);
        assert_eq!(back, input, "row {row}: export change");
    }
}

fn optional_back(value: Option<crate::records::parts::exports::Value>) -> Option<crate::render::evaluate::Value> {
    Some(super::exports::value_back(value?))
}
