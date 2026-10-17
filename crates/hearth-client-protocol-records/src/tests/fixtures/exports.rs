use super::{Part, PartRecord, Spec, copy_of, duplicate, x1_prod};
use crate::parts::exports::{Exports, Named, TaskExports, Value};
use skein_lib::List;
pub const EXPORT_PRINT_DEPTH: u32 = 32;
pub const EXPORT_PRINT_BRACKETS: usize = 32;
pub fn with_exports(mut spec: Spec, task: &[u8], name: &[u8], value: Value) -> Spec {
    spec.parts = Box::new([Part {
        digest: [0; 32],
        record: PartRecord::Exports(Exports {
            tasks: Box::new([TaskExports {
                task: copy_of(task),
                values: Box::new([Named { name: copy_of(name), value }]),
            }]),
        }),
    }]);
    spec
}
pub fn export_spec(value: Value) -> Box<Spec> {
    Box::new(with_exports(*x1_prod(), b"vpn", b"nest", value))
}
pub fn lists_around(depth: u32, value: &[u8]) -> Value {
    let mut value = Value::String(copy_of(value));
    for _ in 0..depth {
        value = Value::List(Box::new([value]));
    }
    value
}
pub fn has_run(bytes: &[u8], parts: &[&[u8]]) -> bool {
    let mut expected = List::with_capacity(u32::try_from(bytes.len()).expect("printed length"));
    for part in parts {
        for byte in *part {
            expected.push(*byte).expect("printed length");
        }
    }
    skein_lib::bytes::find(bytes, expected.as_slice()).is_some()
}
pub fn printed_exports(form: crate::form::Form) -> Box<[u8]> {
    crate::form::print(
        form,
        &crate::records::Record::Spec(Box::new(with_exports(*x1_prod(), b"vpn", b"port", Value::Int(51820)))),
        None,
        &crate::limits::SHIPPED,
    )
    .expect("exports fit")
}
pub fn export_error(fault: crate::errors::ExportFault) -> crate::errors::Error {
    let mut error = duplicate();
    error.at = None;
    error.also = None;
    error.kind = crate::errors::Kind::Schema(crate::errors::Schema::Export { fault });
    error
}
