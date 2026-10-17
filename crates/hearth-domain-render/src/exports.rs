//! A task's typed exports depend on its settings and target alone (format.md, 8.1).
use crate::{
    entries::Fault,
    errors::{ExportFault, Kind, Root, Schema},
    evaluate::{self, Bindings, Budget, Context, Entry, Evaluator, Op, Value, template::Does},
    limits::Limits,
    revision::declaration::{Declaration, Interpolated},
    schema::{Against, Check},
};
use alloc::boxed::Box;
use skein_lib::List;
mod cache;
pub(crate) use cache::{Cache, Fleet};
mod conform;
mod value;
pub use conform::conform;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Exported {
    pub task: Box<[u8]>,
    pub values: Box<[Entry]>,
}
/// Static export faults, at the value string's opening quote.
pub fn check(declaration: &Declaration, document: u32, faults: &mut List<crate::schema::Fault>) {
    for export in &declaration.exports {
        let schema = match expression(&export.value) {
            None => Some(Schema::Export { fault: ExportFault::Form }),
            Some(expression) => {
                let mut refused = None;
                for operation in &expression.operations {
                    let Op::Name { name, missing: _ } = &operation.op else { continue };
                    let root = match name.as_ref() {
                        b"tasks" => Some(Root::Tasks),
                        b"exports" => Some(Root::Exports),
                        b"debian" => Some(Root::Debian),
                        _name => None,
                    };
                    if let Some(root) = root {
                        refused = Some(Schema::Export { fault: ExportFault::Root { root } });
                        break;
                    }
                }
                if refused.is_none()
                    && let Some((schema, _offset)) = crate::schema::references::check(expression, declaration)
                {
                    refused = Some(schema);
                }
                refused
            }
        };
        if let Some(schema) = schema {
            crate::schema::fault(
                faults,
                crate::compose::Provenance { layer: crate::compose::Layer::Defaults, document, at: export.value.at() },
                schema,
            );
        }
    }
}
fn expression(value: &Interpolated) -> Option<&evaluate::Expression> {
    let Interpolated::Parts { at: _, parts } = value else { return None };
    let [instruction] = parts.as_ref() else { return None };
    match &instruction.does {
        Does::Output(expression) => Some(expression),
        Does::Text(_)
        | Does::Branch { .. }
        | Does::Enter { .. }
        | Does::Set { .. }
        | Does::Jump { .. }
        | Does::Advance { .. } => None,
    }
}
/// One target's typed values; absent optionals remain absent.
pub fn evaluate(
    against: &Against<'_>,
    check: &Check<'_>,
    settings: &Value,
    target: &Value,
    limits: &Limits,
    left: &mut u64,
) -> Result<Exported, Fault> {
    evaluation(against, check, settings, target, limits, left, &mut false)
}
fn evaluation(
    against: &Against<'_>,
    check: &Check<'_>,
    settings: &Value,
    target: &Value,
    limits: &Limits,
    left: &mut u64,
    limited: &mut bool,
) -> Result<Exported, Fault> {
    let empty = Value::Map(Box::new([]));
    let context = Context { target, settings, debian: &empty, tasks: &empty, exports: &empty };
    let mut values =
        skein_lib::Map::with_capacity(u32::try_from(against.declaration.exports.len()).expect("export count"));
    for export in &against.declaration.exports {
        let at = export.value.at();
        let Some(expression) = expression(&export.value) else {
            return Err(failure(Kind::Schema(Schema::Export { fault: ExportFault::Form }), at));
        };
        let operand = match evaluate::expression(
            &mut Evaluator::new(limits),
            expression,
            &context,
            &Bindings::with_capacity(0),
            &mut Budget::new(limits),
        ) {
            Ok(value) => value,
            Err(fault) => return Err(failure(Kind::Evaluation(fault.kind), at)),
        };
        let (value, stored) = if operand.undefined() && export.optional {
            (None, 0)
        } else {
            let value = match value::own(&operand, limits) {
                Ok(value) => value,
                Err(kind) => return Err(failure(Kind::Evaluation(kind), at)),
            };
            let stored = match conform::checked(&value, against, check, export.ty, limits) {
                Ok(size) => size,
                Err(schema) => return Err(failure(Kind::Schema(schema), at)),
            };
            (Some(value), stored)
        };
        let mut charge = evaluate::ELEMENT.saturating_add(u64::try_from(export.name.text.len()).expect("export name"));
        charge = charge.saturating_add(stored);
        *left = match left.checked_sub(charge) {
            Some(left) => left,
            None => {
                *limited = true;
                return Err(failure(Kind::Evaluation(evaluate::FaultKind::Limit), at));
            }
        };
        values.insert(export.name.text.clone(), Entry { key: export.name.text.clone(), value }).expect("export count");
    }
    let mut ordered = List::with_capacity(values.len());
    for (_, entry) in &values {
        ordered.push(entry.clone()).expect("export count");
    }
    Ok(Exported { task: Box::new([]), values: ordered.into_boxed() })
}

fn failure(kind: Kind, at: crate::revision::Position) -> Fault {
    Fault { kind, at, also: None }
}

/// An absent read list is the empty root, including guard-only test contexts.
pub(crate) fn of<'a>(roots: &'a Value, task: &[u8]) -> &'a Value {
    if let Value::Map(tasks) = roots {
        for entry in tasks {
            if entry.key.as_ref() == task {
                return entry.value.as_ref().expect("assembled export root");
            }
        }
    }
    roots
}
