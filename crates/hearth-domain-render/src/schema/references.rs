//! Static references in every expression, including untaken branches (format.md, 6.4).
use super::Fault;
use crate::{
    errors::{Root, Schema},
    evaluate::{
        Expression, Op, Operation,
        template::{Does, Instruction, Template},
    },
    revision::{
        Position,
        declaration::{Declaration, Interpolated, Literal},
    },
};
use alloc::boxed::Box;
use skein_lib::{List, bytes};

/// The first undeclared reference and its operation's offset. The caller supplies the reader's name.
#[must_use]
pub fn check(expression: &Expression, declaration: &Declaration) -> Option<(Schema, u32)> {
    for (index, operation) in expression.operations.iter().enumerate() {
        let Op::Name { name, missing: _ } = &operation.op else { continue };
        let tail = expression.operations.get(index.saturating_add(1)..).expect("operation cursor");
        if name.as_ref() == b"settings" {
            if let Some((setting, at, _end)) = literal(tail)
                && !super::declares(declaration, setting)
            {
                let mut declared =
                    List::with_capacity(u32::try_from(declaration.settings.len()).expect("setting bound"));
                for spec in &declaration.settings {
                    declared.push(spec.name.text.clone()).expect("setting count");
                }
                return Some((
                    Schema::Undeclared { setting: bytes::copy_of(setting), declared: declared.into_boxed() },
                    at,
                ));
            }
        } else if name.as_ref() == b"tasks" || name.as_ref() == b"exports" {
            let root = if name.as_ref() == b"tasks" { Root::Tasks } else { Root::Exports };
            if let Some(fault) = read(tail, declaration, root) {
                return Some((fault, operation.at));
            }
        }
    }
    None
}
fn literal(operations: &[Operation]) -> Option<(&[u8], u32, &[Operation])> {
    if let [Operation { at, op: Op::Key { name, missing: _ } }, tail @ ..]
    | [Operation { at, op: Op::String(name) }, Operation { at: _, op: Op::Item { missing: _ } }, tail @ ..] =
        operations
    {
        return Some((name, *at, tail));
    }
    None
}

fn read(operations: &[Operation], declaration: &Declaration, root: Root) -> Option<Schema> {
    let Some((task, _at, tail)) = literal(operations) else { return Some(Schema::Dynamic { root }) };
    let mut setting = None;
    let mut defined = false;
    if root == Root::Tasks {
        if let Some((name, _at, _tail)) = literal(tail) {
            setting = Some(name);
        } else if let Some(Operation { at: _, op: Op::Defined | Op::NotDefined }) = tail.first() {
            defined = true;
        } else {
            return Some(Schema::Dynamic { root });
        }
    }
    for read in &declaration.reads {
        if read.task.text.as_ref() != task {
            continue;
        }
        if root == Root::Exports && read.exports || defined && read.optional {
            return None;
        }
        for listed in &read.settings {
            if setting == Some(listed.text.as_ref()) {
                return None;
            }
        }
    }
    let mut member = None;
    if let Some(setting) = setting {
        member = Some(bytes::copy_of(setting));
    }
    Some(Schema::Unlisted { reader: Box::new([]), root, task: bytes::copy_of(task), setting: member })
}
/// Checks every instruction, without executing any of them.
#[must_use]
pub fn template(template: &Template, declaration: &Declaration) -> Option<(Schema, u32)> {
    instructions(&template.instructions, declaration)
}
fn instructions(instructions: &[Instruction], declaration: &Declaration) -> Option<(Schema, u32)> {
    for instruction in instructions {
        let expression = match &instruction.does {
            Does::Output(expression)
            | Does::Branch { condition: expression, over: _ }
            | Does::Enter { names: _, iterable: expression, over: _ }
            | Does::Set { name: _, value: expression } => expression,
            Does::Text(_) | Does::Jump { over: _ } | Does::Advance { back: _ } => continue,
        };
        if let Some(fault) = check(expression, declaration) {
            return Some(fault);
        }
    }
    None
}
/// Walks declaration expressions at their JSON strings' opening quotes.
pub fn declaration(declaration: &Declaration, document: u32, faults: &mut List<Fault>) {
    for guard in &declaration.requires {
        condition(&guard.when.expression, guard.when.at, declaration, document, faults);
    }
    for map in &declaration.entries {
        for entry in &map.entries {
            for when in [&entry.when, &entry.for_each].into_iter().flatten() {
                condition(&when.expression, when.at, declaration, document, faults);
            }
            interpolated(&entry.key, declaration, document, faults);
            for field in &entry.fields {
                match &field.value {
                    Literal::Range { at: _, fields } => {
                        for member in fields {
                            literal_field(&member.value, declaration, document, faults);
                        }
                    }
                    value @ (Literal::Text(_)
                    | Literal::Typed(_)
                    | Literal::Plain { at: _, text: _ }
                    | Literal::Int(_)
                    | Literal::Bool { at: _, value: _ }
                    | Literal::Strings { at: _, each: _ }) => literal_field(value, declaration, document, faults),
                }
            }
        }
    }
}
fn literal_field(value: &Literal, declaration: &Declaration, document: u32, faults: &mut List<Fault>) {
    match value {
        Literal::Text(value) | Literal::Typed(value) => interpolated(value, declaration, document, faults),
        Literal::Strings { at: _, each } => {
            for value in each {
                interpolated(value, declaration, document, faults);
            }
        }
        Literal::Plain { at: _, text: _ }
        | Literal::Int(_)
        | Literal::Bool { at: _, value: _ }
        | Literal::Range { at: _, fields: _ } => {}
    }
}
fn interpolated(value: &Interpolated, declaration: &Declaration, document: u32, faults: &mut List<Fault>) {
    if let Interpolated::Parts { at, parts } = value {
        for instruction in parts {
            if let Does::Output(expression) = &instruction.does {
                condition(expression, *at, declaration, document, faults);
            }
        }
    }
}
fn condition(
    expression: &Expression,
    at: Position,
    declaration: &Declaration,
    document: u32,
    faults: &mut List<Fault>,
) {
    if let Some((schema, _offset)) = check(expression, declaration) {
        if faults.room() == 0 {
            let mut larger = List::with_capacity(faults.capacity().max(1).saturating_mul(2));
            for fault in &*faults {
                larger.push(fault.clone()).expect("doubled capacity");
            }
            *faults = larger;
        }
        faults.push(Fault { schema, document, at }).expect("fault capacity");
    }
}
