use super::{composition::at, declaration, declared_spec, entry_output, op, parts};
use crate::{
    evaluate::{self, Value},
    exports::Exported,
    limits::SHIPPED,
    parts::exports::Exports,
    revision::declaration::{self as declared, Declaration, Form, Interpolated},
    schema::{self, Against, Check},
};
use skein_lib::{List, bytes::copy_of};
type TaskExports<'a> = (&'a [u8], &'a [(&'a [u8], Value)]);
pub fn exports(tasks: &[TaskExports<'_>]) -> Exports {
    let mut exported = List::with_capacity(u32::try_from(tasks.len()).expect("fixture count"));
    for (task, values) in tasks {
        let mut entries = List::with_capacity(u32::try_from(values.len()).expect("fixture count"));
        for (name, value) in *values {
            entries.push(evaluate::Entry { key: copy_of(name), value: Some(value.clone()) }).expect("fixture count");
        }
        exported.push(Exported { task: copy_of(task), values: entries.into_boxed() }).expect("fixture count");
    }
    crate::parts::exports::build(exported.into_boxed())
}
pub fn declared(form: Form, value: Interpolated, optional: bool) -> Declaration {
    let mut declaration = declaration(&[declared_spec(b"port"), declared_spec(b"wifi")], &[]);
    declaration.types = Box::new([declared::Type { at: at(1, 1), form }]);
    declaration.exports = Box::new([declared::Export {
        name: declared::Text { at: at(3, 5), text: copy_of(b"port") },
        at: at(3, 12),
        ty: 0,
        optional,
        value,
    }]);
    declaration
}
pub fn integer(value: i64) -> Declaration {
    declared(Form::Int, parts([entry_output([op(0, evaluate::Op::Integer(value))])]), false)
}
pub fn checked(declaration: &Declaration) -> Box<[schema::Fault]> {
    let mut faults = List::with_capacity(1);
    crate::exports::check(declaration, 0, &mut faults);
    faults.into_boxed()
}
pub fn evaluate_exports(declaration: &Declaration) -> Result<Exported, crate::entries::Fault> {
    let (defaults, _) = super::schemas::defaults(declaration);
    let against = Against { declaration, defaults: defaults.of(b"task").expect("sound declaration"), listing: &[] };
    let roots = super::Roots::x1_prod();
    crate::exports::evaluate(
        &against,
        &Check::Target { environment: b"prod", targets: &[] },
        &roots.settings,
        &roots.target,
        &SHIPPED,
        &mut { SHIPPED.expanded },
    )
}
pub fn conform(value: &Value, form: Form) -> Result<(), crate::errors::Schema> {
    let mut declaration = integer(1);
    declaration.types =
        Box::new([declared::Type { at: at(1, 1), form }, declared::Type { at: at(1, 2), form: Form::Int }]);
    let (defaults, _) = super::schemas::defaults(&declaration);
    let against =
        Against { declaration: &declaration, defaults: defaults.of(b"task").expect("sound declaration"), listing: &[] };
    crate::exports::conform(value, &against, &Check::Target { environment: b"prod", targets: &[] }, 0, &SHIPPED)
}
pub fn lists(depth: u32, value: Value) -> Value {
    let mut value = value;
    for _ in 0..depth {
        value = Value::List(Box::new([value]));
    }
    value
}

pub fn optional_export(optional: bool) -> Declaration {
    declared(
        Form::String,
        parts([entry_output([
            super::name(0, b"settings", evaluate::Missing::Undefined),
            super::key(9, b"wifi", evaluate::Missing::Undefined),
        ])]),
        optional,
    )
}

pub const MIN_UID: i64 = 1000;
pub const MAX_UID: i64 = 59999;

pub fn export_endpoint(value: Value) -> Exports {
    exports(&[(b"vpn", &[(b"endpoint", value)])])
}
pub fn exports_compared(config: bool) -> crate::diff::TargetChange {
    use crate::{
        diff::{self, Made, Needs},
        parts::{Part, PartRecord},
    };
    use hearth_lib::encode::Digest;
    let (mut before, mut after) =
        if config { super::changed_specs() } else { (super::spec(b"ion-prod", &[]), super::spec(b"ion-prod", &[])) };
    for (spec, value) in [(&mut before, 1), (&mut after, 2)] {
        let exports = export_endpoint(Value::Int(value));
        let mut parts = List::with_capacity(u32::try_from(spec.parts.len()).expect("fixture count").saturating_add(1));
        for part in &spec.parts {
            parts.push(part.clone()).expect("fixture count");
        }
        parts
            .push(Part { digest: crate::parts::exports::digest(&exports), record: PartRecord::Exports(exports) })
            .expect("fixture count");
        spec.parts = parts.into_boxed();
    }
    after.digest = Digest([3; 32]);
    diff::compare(Some(&Made::Spec(Box::new(before))), Some(&Made::Spec(Box::new(after))), Needs::Known)
}
