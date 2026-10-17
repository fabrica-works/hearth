use crate::{records::parts::exports as records, render, tests::fixtures, translate::parts::exports as translate};
use skein_lib::{List, bytes::copy_of};
pub(super) fn value_back(value: records::Value) -> render::evaluate::Value {
    use render::evaluate::{Entry, Value};
    match value {
        records::Value::Bool(value) => Value::Bool(value),
        records::Value::Int(value) => Value::Int(value),
        records::Value::String(value) => Value::String(value),
        records::Value::List(values) => {
            let mut items = List::with_capacity(u32::try_from(values.len()).expect("fixture count"));
            for value in values {
                items.push(value_back(value)).expect("fixture count");
            }
            Value::List(items.into_boxed())
        }
        records::Value::Map(values) => {
            let mut entries = List::with_capacity(u32::try_from(values.len()).expect("fixture count"));
            for (key, value) in values {
                entries.push(Entry { key, value: Some(value_back(value)) }).expect("fixture count");
            }
            Value::Map(entries.into_boxed())
        }
    }
}
pub(super) fn exports_back(exports: records::Exports) -> render::parts::exports::Exports {
    let mut tasks = List::with_capacity(u32::try_from(exports.tasks.len()).expect("fixture count"));
    for task in exports.tasks {
        let mut values = List::with_capacity(u32::try_from(task.values.len()).expect("fixture count"));
        for named in task.values {
            values
                .push(render::evaluate::Entry { key: named.name, value: Some(value_back(named.value)) })
                .expect("fixture count");
        }
        tasks.push(render::exports::Exported { task: task.task, values: values.into_boxed() }).expect("fixture count");
    }
    render::parts::exports::Exports { tasks: tasks.into_boxed() }
}
pub(super) fn fault_back(fault: crate::records::errors::ExportFault) -> render::errors::ExportFault {
    use crate::records::errors::{ExportFault, Root};
    match fault {
        ExportFault::Form => render::errors::ExportFault::Form,
        ExportFault::Root { root } => render::errors::ExportFault::Root {
            root: match root {
                Root::Tasks => render::errors::Root::Tasks,
                Root::Exports => render::errors::Root::Exports,
                Root::Debian => render::errors::Root::Debian,
            },
        },
    }
}
#[test]
fn each_export_and_its_value_translates_to_the_records_and_back() {
    for (row, value) in fixtures::export_values().into_iter().enumerate() {
        let input = render::parts::exports::Exports {
            tasks: Box::new([render::exports::Exported {
                task: copy_of(b"vpn"),
                values: Box::new([render::evaluate::Entry { key: copy_of(b"value"), value: Some(value) }]),
            }]),
        };
        let back = exports_back(translate::exports(input.clone()));
        assert_eq!(back, input, "row {row}: typed exports");
    }
}
#[test]
fn a_value_32_deep_translates_without_recursion() {
    let input = fixtures::nested_export(fixtures::EXPORT_PRINT_DEPTH);
    let back = value_back(translate::value(input.clone()));
    assert_eq!(back, input);
}
const FAULTS: &[render::errors::ExportFault] = &[
    render::errors::ExportFault::Form,
    render::errors::ExportFault::Root { root: render::errors::Root::Tasks },
    render::errors::ExportFault::Root { root: render::errors::Root::Exports },
    render::errors::ExportFault::Root { root: render::errors::Root::Debian },
];
#[test]
fn each_export_fault_translates_and_back() {
    for (row, input) in FAULTS.iter().enumerate() {
        let crate::records::errors::Schema::Export { fault } =
            crate::translate::errors::schema(render::errors::Schema::Export { fault: *input })
        else {
            panic!("export fault")
        };
        let back = fault_back(fault);
        assert_eq!(back, *input, "row {row}: export fault");
    }
}
