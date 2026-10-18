//! The declaration's retained values, names and compiled expressions (2.3).
use super::{entry, size, template, value};
use crate::{evaluate::ELEMENT, limits::Limits, revision::declaration as d};

pub(crate) fn declaration(declaration: &d::Declaration, limits: &Limits) -> u64 {
    let mut bytes = 0_u64;
    for ty in &declaration.types {
        bytes = bytes.saturating_add(ELEMENT);
        let owned = match &ty.form {
            d::Form::Enum(names) => {
                let mut bytes = 0_u64;
                for name in names {
                    bytes = bytes.saturating_add(entry(&name.text));
                }
                bytes
            }
            d::Form::Object(fields) => specs(fields, limits),
            d::Form::Bool
            | d::Form::Int
            | d::Form::String
            | d::Form::Path
            | d::Form::Mode
            | d::Form::Duration
            | d::Form::Secret
            | d::Form::Target
            | d::Form::File
            | d::Form::List(_)
            | d::Form::Map(_) => 0,
        };
        bytes = bytes.saturating_add(owned);
    }
    bytes = bytes.saturating_add(specs(&declaration.settings, limits));
    for read in &declaration.reads {
        bytes = bytes.saturating_add(entry(&read.task.text));
        for setting in &read.settings {
            bytes = bytes.saturating_add(entry(&setting.text));
        }
    }
    for guard in &declaration.requires {
        bytes = bytes
            .saturating_add(entry(&guard.name.text))
            .saturating_add(size(&guard.message.text))
            .saturating_add(template::expression(&guard.when.expression));
    }
    bytes = bytes
        .saturating_add(u64::try_from(declaration.scripts.len()).expect("bounded scripts").saturating_mul(ELEMENT));
    for export in &declaration.exports {
        bytes = bytes.saturating_add(entry(&export.name.text)).saturating_add(interpolated(&export.value));
    }
    for map in &declaration.entries {
        bytes = bytes.saturating_add(ELEMENT);
        for item in &map.entries {
            bytes = bytes.saturating_add(ELEMENT).saturating_add(interpolated(&item.key));
            if let Some(condition) = &item.when {
                bytes = bytes.saturating_add(template::expression(&condition.expression));
            }
            if let Some(condition) = &item.for_each {
                bytes = bytes.saturating_add(template::expression(&condition.expression));
            }
            bytes = bytes.saturating_add(fields(&item.fields));
        }
    }
    bytes
}
fn specs(specs: &[d::Spec], limits: &Limits) -> u64 {
    let mut bytes = 0_u64;
    for spec in specs {
        bytes = bytes.saturating_add(entry(&spec.name.text)).saturating_add(entry(b"type"));
        if let Some(default) = &spec.default {
            bytes = bytes.saturating_add(entry(b"default")).saturating_add(value(default, limits));
        }
        if spec.optional {
            bytes = bytes.saturating_add(entry(b"optional"));
        }
        if spec.min.is_some() {
            bytes = bytes.saturating_add(entry(b"min"));
        }
        if spec.max.is_some() {
            bytes = bytes.saturating_add(entry(b"max"));
        }
        if spec.max_length.is_some() {
            bytes = bytes.saturating_add(entry(b"max_length"));
        }
    }
    bytes
}
fn interpolated(value: &d::Interpolated) -> u64 {
    match value {
        d::Interpolated::Plain { text, .. } => size(text),
        d::Interpolated::Parts { parts, .. } => template::instructions(parts),
    }
}
fn fields(fields: &[d::Field]) -> u64 {
    let mut bytes = 0_u64;
    for field in fields {
        bytes = bytes.saturating_add(entry(field.name.key()));
        // Ranges cannot contain another range; the parser restricts their fields.
        let owned = match &field.value {
            d::Literal::Range { fields, .. } => range(fields),
            d::Literal::Text(value) | d::Literal::Typed(value) => interpolated(value),
            d::Literal::Plain { text, .. } => size(text),
            d::Literal::Int(_) | d::Literal::Bool { .. } => 0,
            d::Literal::Strings { each, .. } => {
                let mut bytes = 0_u64;
                for item in each {
                    bytes = bytes.saturating_add(ELEMENT).saturating_add(interpolated(item));
                }
                bytes
            }
        };
        bytes = bytes.saturating_add(owned);
    }
    bytes
}
fn range(fields: &[d::Field]) -> u64 {
    let mut bytes = 0_u64;
    for field in fields {
        bytes = bytes.saturating_add(entry(field.name.key()));
        match &field.value {
            d::Literal::Text(text) | d::Literal::Typed(text) => bytes = bytes.saturating_add(interpolated(text)),
            d::Literal::Plain { text, .. } => bytes = bytes.saturating_add(size(text)),
            d::Literal::Strings { .. } | d::Literal::Range { .. } | d::Literal::Bool { .. } => {
                unreachable!("only numeric range fields reach the domain")
            }
            d::Literal::Int(_) => {}
        }
    }
    bytes
}
