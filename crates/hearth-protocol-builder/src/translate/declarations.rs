//! Declaration schemas and reads (format.md, 6.1 to 6.5).
use super::documents::{position, value};
use crate::{Limits, config, render::revision::declaration as target};
use alloc::boxed::Box;
use skein_lib::List;
pub fn declaration(
    declaration: config::declaration::Declaration,
    limits: &Limits,
) -> Result<target::Declaration, Refusal> {
    let config::declaration::Declaration {
        at,
        types,
        settings,
        reads,
        requires,
        scripts: _,
        exports,
        bless: _,
        entries,
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
    let mut guards = List::with_capacity(u32::try_from(requires.len()).expect("bounded guards"));
    for written in requires {
        guards.push(guard(written, &limits.template)?).expect("guard count");
    }
    let mut maps = List::with_capacity(u32::try_from(entries.len()).expect("bounded maps"));
    for written in entries {
        maps.push(self::entries(written, &limits.template)?).expect("map count");
    }
    let mut published = List::with_capacity(u32::try_from(exports.len()).expect("export count"));
    for written in exports {
        published.push(export(written, &limits.template)?).expect("export count");
    }
    Ok(target::Declaration {
        at: position(at),
        types: translated.into_boxed(),
        settings: specs(settings, limits),
        reads: translated_reads.into_boxed(),
        requires: guards.into_boxed(),
        exports: published.into_boxed(),
        entries: maps.into_boxed(),
    })
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

#[derive(Debug)]
pub struct Refusal {
    pub at: crate::render::revision::Position,
    pub fault: crate::render::errors::Decode,
}
pub fn condition(
    written: config::declaration::Text,
    limits: &hearth_protocol_template::limits::Limits,
) -> Result<target::Condition, Refusal> {
    let at = position(written.at);
    let expression = match hearth_protocol_template::expression::compile(&written.text, limits) {
        Ok(expression) => expression,
        Err(error) => {
            return Err(Refusal {
                at,
                fault: crate::render::errors::Decode::Syntax(super::errors::syntax(error.kind)),
            });
        }
    };
    Ok(target::Condition { at, expression: super::operations::expression(expression) })
}
pub fn guard(
    written: config::declaration::Guard,
    limits: &hearth_protocol_template::limits::Limits,
) -> Result<target::Guard, Refusal> {
    let config::declaration::Guard { name, at, when, message } = written;
    let when = condition(when, limits)?;
    let at_message = position(message.at);
    match hearth_protocol_template::interpolation::compile(&message.text, limits) {
        Ok(hearth_protocol_template::interpolation::Interpolated::Plain) => {}
        Ok(hearth_protocol_template::interpolation::Interpolated::Parts(_)) => {
            return Err(Refusal {
                at: at_message,
                fault: crate::render::errors::Decode::Syntax(crate::render::errors::Syntax::Interpolation),
            });
        }
        Err(error) => {
            return Err(Refusal {
                at: at_message,
                fault: crate::render::errors::Decode::Syntax(super::errors::syntax(error.kind)),
            });
        }
    }
    Ok(target::Guard { name: text(name), at: position(at), when, message: text(message) })
}

pub fn entries(
    written: config::entries::Entries,
    limits: &hearth_protocol_template::limits::Limits,
) -> Result<target::Entries, Refusal> {
    let config::entries::Entries { map, at, entries } = written;
    let mut translated = List::with_capacity(u32::try_from(entries.len()).expect("bounded entries"));
    for written in entries {
        translated.push(entry(written, limits)?).expect("entry count");
    }
    Ok(target::Entries { map: super::errors::declaration_map(map), at: position(at), entries: translated.into_boxed() })
}
pub fn entry(
    written: config::entries::Entry,
    limits: &hearth_protocol_template::limits::Limits,
) -> Result<target::Entry, Refusal> {
    let config::entries::Entry { key, at, when, for_each, fields } = written;
    let when = match when {
        Some(written) => Some(condition(written, limits)?),
        None => None,
    };
    let for_each = match for_each {
        Some(written) => Some(condition(written, limits)?),
        None => None,
    };
    Ok(target::Entry {
        key: interpolated(key, limits)?,
        at: position(at),
        when,
        for_each,
        fields: self::fields(fields, limits)?,
    })
}
pub fn interpolated(
    written: config::declaration::Text,
    limits: &hearth_protocol_template::limits::Limits,
) -> Result<target::Interpolated, Refusal> {
    let at = position(written.at);
    match hearth_protocol_template::interpolation::compile(&written.text, limits) {
        Ok(hearth_protocol_template::interpolation::Interpolated::Plain) => {
            Ok(target::Interpolated::Plain { at, text: written.text })
        }
        Ok(hearth_protocol_template::interpolation::Interpolated::Parts(parts)) => {
            let mut translated = List::with_capacity(u32::try_from(parts.len()).expect("bounded parts"));
            for part in parts {
                translated.push(super::instructions::instruction(part)).expect("part count");
            }
            Ok(target::Interpolated::Parts { at, parts: translated.into_boxed() })
        }
        Err(error) => {
            Err(Refusal { at, fault: crate::render::errors::Decode::Syntax(super::errors::syntax(error.kind)) })
        }
    }
}

fn fields(
    written: Box<[config::entries::Field]>,
    limits: &hearth_protocol_template::limits::Limits,
) -> Result<Box<[target::Field]>, Refusal> {
    let mut fields = List::with_capacity(u32::try_from(written.len()).expect("field count"));
    for written in written {
        fields.push(field(written, limits)?).expect("field count");
    }
    Ok(fields.into_boxed())
}
pub fn field(
    written: config::entries::Field,
    limits: &hearth_protocol_template::limits::Limits,
) -> Result<target::Field, Refusal> {
    let config::entries::Field { name, at, value } = written;
    Ok(target::Field { name: self::name(name), at: position(at), value: literal(value, limits)? })
}
pub fn literal(
    written: config::entries::Literal,
    limits: &hearth_protocol_template::limits::Limits,
) -> Result<target::Literal, Refusal> {
    match written {
        config::entries::Literal::Text(text) => Ok(target::Literal::Text(interpolated(text, limits)?)),
        config::entries::Literal::Plain(text) => Ok(target::Literal::Plain { at: position(text.at), text: text.text }),
        config::entries::Literal::Typed(text) => Ok(target::Literal::Typed(interpolated(text, limits)?)),
        config::entries::Literal::Int(config::declaration::Number { at, value }) => {
            Ok(target::Literal::Int(target::Number { at: position(at), value }))
        }
        config::entries::Literal::Bool { at, value } => Ok(target::Literal::Bool { at: position(at), value }),
        config::entries::Literal::Strings { at, each } => {
            let mut strings = List::with_capacity(u32::try_from(each.len()).expect("string count"));
            for text in each {
                strings.push(interpolated(text, limits)?).expect("string count");
            }
            Ok(target::Literal::Strings { at: position(at), each: strings.into_boxed() })
        }
        config::entries::Literal::Range { at, fields: written } => {
            Ok(target::Literal::Range { at: position(at), fields: fields(written, limits)? })
        }
    }
}
#[must_use]
pub fn name(name: config::entries::Name) -> target::Name {
    match name {
        config::entries::Name::Action => target::Name::Action,
        config::entries::Name::Age => target::Name::Age,
        config::entries::Name::Algorithm => target::Name::Algorithm,
        config::entries::Name::Argument => target::Name::Argument,
        config::entries::Name::Base => target::Name::Base,
        config::entries::Name::Class => target::Name::Class,
        config::entries::Name::Component => target::Name::Component,
        config::entries::Name::Copy => target::Name::Copy,
        config::entries::Name::Count => target::Name::Count,
        config::entries::Name::Directory => target::Name::Directory,
        config::entries::Name::From => target::Name::From,
        config::entries::Name::Gecos => target::Name::Gecos,
        config::entries::Name::Gid => target::Name::Gid,
        config::entries::Name::Group => target::Name::Group,
        config::entries::Name::Groups => target::Name::Groups,
        config::entries::Name::Home => target::Name::Home,
        config::entries::Name::Kind => target::Name::Kind,
        config::entries::Name::Link => target::Name::Link,
        config::entries::Name::Mode => target::Name::Mode,
        config::entries::Name::Order => target::Name::Order,
        config::entries::Name::Owner => target::Name::Owner,
        config::entries::Name::Param => target::Name::Param,
        config::entries::Name::Password => target::Name::Password,
        config::entries::Name::Path => target::Name::Path,
        config::entries::Name::Paths => target::Name::Paths,
        config::entries::Name::PublicPath => target::Name::PublicPath,
        config::entries::Name::Sha256 => target::Name::Sha256,
        config::entries::Name::Shell => target::Name::Shell,
        config::entries::Name::Strip => target::Name::Strip,
        config::entries::Name::Subordinate => target::Name::Subordinate,
        config::entries::Name::Template => target::Name::Template,
        config::entries::Name::Type => target::Name::Type,
        config::entries::Name::Uid => target::Name::Uid,
        config::entries::Name::Unit => target::Name::Unit,
        config::entries::Name::Unpack => target::Name::Unpack,
        config::entries::Name::Url => target::Name::Url,
        config::entries::Name::Value => target::Name::Value,
    }
}

pub fn export(
    written: config::schema::Export,
    limits: &hearth_protocol_template::limits::Limits,
) -> Result<target::Export, Refusal> {
    let config::schema::Export { name, at, ty, optional, value } = written;
    Ok(target::Export { name: text(name), at: position(at), ty, optional, value: interpolated(value, limits)? })
}
