use crate::render;
use skein_lib::bytes::copy_of;
pub const EXPORT_PRINT_DEPTH: u32 = 32;
#[must_use]
pub fn export_values() -> [render::evaluate::Value; 5] {
    use render::evaluate::{Entry, Value};
    [
        Value::Bool(true),
        Value::Int(-7),
        Value::String(copy_of(b"vpn")),
        Value::List(Box::new([Value::Int(1)])),
        Value::Map(Box::new([Entry { key: copy_of(b"a"), value: Some(Value::Int(2)) }])),
    ]
}
pub fn nested_export(depth: u32) -> render::evaluate::Value {
    let mut value = render::evaluate::Value::Int(1);
    for _ in 0..depth {
        value = render::evaluate::Value::List(Box::new([value]));
    }
    value
}

#[must_use]
pub fn export_specs() -> [render::parts::Spec; 2] {
    let plain = super::spec();
    let mut member = super::spec();
    member.meta.member = Some(copy_of(b"1"));
    [plain, member]
}
