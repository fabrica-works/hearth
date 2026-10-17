//! Declaration schemas and reads (format.md, 6.1 to 6.5).
use super::documents::{position, value};
use crate::{Limits, config, render::revision::declaration as target};
use alloc::boxed::Box;
use skein_lib::List;
#[must_use]
pub fn declaration(declaration: config::declaration::Declaration, limits: &Limits) -> target::Declaration {
    let config::declaration::Declaration {
        at,
        types,
        settings,
        reads,
        requires: _,
        scripts: _,
        exports: _,
        bless: _,
        entries: _,
    } = declaration;
    let mut translated = List::with_capacity(u32::try_from(types.len()).expect("bounded types"));
    for config::schema::Type { at, form: written } in types {
        translated.push(target::Type { at: position(at), form: form(written, limits) }).expect("type count");
    }
    let mut translated_reads = List::with_capacity(u32::try_from(reads.len()).expect("bounded reads"));
    for config::declaration::Read { task, at, settings, exports, optional } in reads {
        translated_reads
            .push(target::Read { task: text(task), at: position(at), settings: texts(settings), exports, optional })
            .expect("read count");
    }
    target::Declaration {
        at: position(at),
        types: translated.into_boxed(),
        settings: specs(settings, limits),
        reads: translated_reads.into_boxed(),
    }
}
#[must_use]
pub fn text(text: config::declaration::Text) -> target::Text {
    let config::declaration::Text { at, text } = text;
    target::Text { at: position(at), text }
}
fn texts(texts: Box<[config::declaration::Text]>) -> Box<[target::Text]> {
    let mut translated = List::with_capacity(u32::try_from(texts.len()).expect("bounded texts"));
    for written in texts {
        translated.push(text(written)).expect("text count");
    }
    translated.into_boxed()
}
fn number(number: Option<config::declaration::Number>) -> Option<target::Number> {
    let config::declaration::Number { at, value } = number?;
    Some(target::Number { at: position(at), value })
}
fn default(default: Option<config::json::Value>) -> Option<crate::render::revision::Value> {
    Some(value(default?))
}
fn specs(specs: Box<[config::schema::Spec]>, _limits: &Limits) -> Box<[target::Spec]> {
    let mut translated = List::with_capacity(u32::try_from(specs.len()).expect("bounded specs"));
    for config::schema::Spec { name, at, ty, default: written, optional, min, max, max_length } in specs {
        translated
            .push(target::Spec {
                name: text(name),
                at: position(at),
                ty,
                default: default(written),
                optional,
                min: number(min),
                max: number(max),
                max_length: number(max_length),
            })
            .expect("spec count");
    }
    translated.into_boxed()
}
#[must_use]
pub fn form(form: config::schema::Form, limits: &Limits) -> target::Form {
    match form {
        config::schema::Form::Bool => target::Form::Bool,
        config::schema::Form::Int => target::Form::Int,
        config::schema::Form::String => target::Form::String,
        config::schema::Form::Path => target::Form::Path,
        config::schema::Form::Mode => target::Form::Mode,
        config::schema::Form::Duration => target::Form::Duration,
        config::schema::Form::Secret => target::Form::Secret,
        config::schema::Form::Target => target::Form::Target,
        config::schema::Form::File => target::Form::File,
        config::schema::Form::Enum(values) => target::Form::Enum(texts(values)),
        config::schema::Form::List(ty) => target::Form::List(ty),
        config::schema::Form::Map(ty) => target::Form::Map(ty),
        config::schema::Form::Object(fields) => target::Form::Object(specs(fields, limits)),
    }
}
