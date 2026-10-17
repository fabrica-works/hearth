//! Shared input builders and sweep observations.
use crate::world::{Outcome, Run, Settings};
use hearth_protocol_config::{Fault, json::Waiting, limits::Limits};
#[derive(Default, Debug)]
pub struct Seen {
    bits: u32,
}
impl Seen {
    pub const EVERY_FAULT_AND_ENDING: u32 = 0x3fff;
    pub fn record(&mut self, run: &Run, _settings: &Settings) {
        self.bits |= match &run.outcome {
            Some(Outcome::Decoded) => 1 << 9,
            Some(Outcome::Failed(error)) => match error.fault {
                Fault::Json(_) => 1,
                Fault::Stream(_) => 1 << 1,
                Fault::NotObject => 1 << 2,
                Fault::Duplicate { .. } => 1 << 3,
                Fault::NotInteger => 1 << 4,
                Fault::Range => 1 << 5,
                Fault::Nul => 1 << 6,
                Fault::Keys => 1 << 7,
                Fault::Note => 1 << 8,
                Fault::Shape(_) | Fault::Declaration(_) => panic!("shape is above the machine"),
            },
            None => 0,
        };
        if run.early_end {
            self.bits |= 1 << 10;
        }
        self.bits |= match run.closed_at {
            Waiting::Decode => 1 << 11,
            Waiting::Bytes => 1 << 12,
            Waiting::Close => 1 << 13,
            Waiting::Nothing => 0,
        };
    }
    pub fn assert_fell(&self, expected: u32) {
        assert_eq!(self.bits, expected, "each injected fault and ending fell");
    }
}
#[must_use]
pub fn at_the_limits(limits: &Limits) -> Vec<(Vec<u8>, &'static str)> {
    let width = limits.keys as usize;
    let mut widest =
        format!("{{{}}}", (0..width).map(|n| format!("\"k{n}\":0")).collect::<Vec<_>>().join(",")).into_bytes();
    let mut excess = widest.clone();
    excess.pop();
    excess.extend_from_slice(b",\"extra\":0}");
    let deep = format!(
        "{{\"a\":{}0{}}}",
        "[".repeat(limits.depth.saturating_sub(1) as usize),
        "]".repeat(limits.depth.saturating_sub(1) as usize)
    )
    .into_bytes();
    let past_depth =
        format!("{{\"a\":{}0{}}}", "[".repeat(limits.depth as usize), "]".repeat(limits.depth as usize)).into_bytes();
    let text = format!("{{\"a\":\"{}\"}}", "a".repeat(limits.document.saturating_sub(8) as usize)).into_bytes();
    let past_text = format!("{{\"a\":\"{}\"}}", "a".repeat(limits.document.saturating_sub(7) as usize)).into_bytes();
    let array = format!(
        "{{\"a\":[{}]}}",
        (0..limits.document.saturating_sub(9) / 2).map(|_| "0").collect::<Vec<_>>().join(",")
    )
    .into_bytes();
    widest.shrink_to_fit();
    vec![
        (widest, "widest"),
        (excess, "past width"),
        (deep, "deepest"),
        (past_depth, "past depth"),
        (text, "full string"),
        (past_text, "past length"),
        (array.clone(), "full array"),
        ([array, vec![b' '; 16]].concat(), "past array length"),
    ]
}

#[must_use]
pub const fn at(offset: u32, line: u32, column: u32) -> hearth_protocol_config::Position {
    hearth_protocol_config::Position { offset, line, column }
}
#[must_use]
pub fn text(at: hearth_protocol_config::Position, input: &[u8]) -> hearth_protocol_config::declaration::Text {
    hearth_protocol_config::declaration::Text { at, text: input.into() }
}

/// Declaration faults observed by the mutation sweep.
#[derive(Default, Debug)]
pub struct Tally {
    faults: Vec<hearth_protocol_config::Declared>,
}
use hearth_protocol_config::{Expected, Object, Required};
impl Tally {
    pub const EVERY_DECLARED: &'static [hearth_protocol_config::Declared] = &[
        hearth_protocol_config::Declared::Unknown { of: Object::Declaration },
        hearth_protocol_config::Declared::Unknown { of: Object::Read },
        hearth_protocol_config::Declared::Unknown { of: Object::Guard },
        hearth_protocol_config::Declared::Unknown { of: Object::Script },
        hearth_protocol_config::Declared::Unknown { of: Object::Bless },
        hearth_protocol_config::Declared::Unknown { of: Object::Entry(hearth_protocol_config::entries::Map::Packages) },
        hearth_protocol_config::Declared::Unknown { of: Object::Entry(hearth_protocol_config::entries::Map::Preseeds) },
        hearth_protocol_config::Declared::Unknown {
            of: Object::Entry(hearth_protocol_config::entries::Map::Initramfs),
        },
        hearth_protocol_config::Declared::Unknown { of: Object::Entry(hearth_protocol_config::entries::Map::Software) },
        hearth_protocol_config::Declared::Unknown { of: Object::Entry(hearth_protocol_config::entries::Map::Files) },
        hearth_protocol_config::Declared::Unknown { of: Object::Entry(hearth_protocol_config::entries::Map::Users) },
        hearth_protocol_config::Declared::Unknown { of: Object::Entry(hearth_protocol_config::entries::Map::Groups) },
        hearth_protocol_config::Declared::Unknown { of: Object::Entry(hearth_protocol_config::entries::Map::Presets) },
        hearth_protocol_config::Declared::Unknown { of: Object::Entry(hearth_protocol_config::entries::Map::Tmpfiles) },
        hearth_protocol_config::Declared::Unknown { of: Object::Entry(hearth_protocol_config::entries::Map::Data) },
        hearth_protocol_config::Declared::Unknown { of: Object::Entry(hearth_protocol_config::entries::Map::Secrets) },
        hearth_protocol_config::Declared::Unknown { of: Object::Entry(hearth_protocol_config::entries::Map::Identity) },
        hearth_protocol_config::Declared::Unknown { of: Object::Entry(hearth_protocol_config::entries::Map::Cmdline) },
        hearth_protocol_config::Declared::Unknown { of: Object::Entry(hearth_protocol_config::entries::Map::Live) },
        hearth_protocol_config::Declared::Unknown { of: Object::Range },
        hearth_protocol_config::Declared::Unknown { of: Object::Spec },
        hearth_protocol_config::Declared::Unknown { of: Object::Export },
        hearth_protocol_config::Declared::Later,
        hearth_protocol_config::Declared::Wrong { expected: Expected::Object },
        hearth_protocol_config::Declared::Missing { key: Required::If },
        hearth_protocol_config::Declared::Name,
        hearth_protocol_config::Declared::Doc,
        hearth_protocol_config::Declared::Order,
        hearth_protocol_config::Declared::Optional,
        hearth_protocol_config::Declared::Bound,
        hearth_protocol_config::Declared::Above,
        hearth_protocol_config::Declared::Enum,
        hearth_protocol_config::Declared::Secret,
        hearth_protocol_config::Declared::Settings,
    ];
    pub fn record(&mut self, fault: hearth_protocol_config::Declared) {
        use hearth_protocol_config::Declared;
        match fault {
            Declared::Unknown { of: Object::Type } => panic!("type objects refuse Wrong(Type)"),
            Declared::Unknown {
                of:
                    Object::Declaration
                    | Object::Read
                    | Object::Guard
                    | Object::Script
                    | Object::Bless
                    | Object::Entry(_)
                    | Object::Range
                    | Object::Spec
                    | Object::Export,
            }
            | Declared::Later
            | Declared::Wrong { .. }
            | Declared::Missing { .. }
            | Declared::Name
            | Declared::Doc
            | Declared::Order
            | Declared::Optional
            | Declared::Bound
            | Declared::Above
            | Declared::Enum
            | Declared::Settings
            | Declared::Secret => {}
        }
        if !self.faults.contains(&fault) {
            self.faults.push(fault);
        }
    }
    pub fn assert_fell(&self, expected: &[hearth_protocol_config::Declared]) {
        for fault in expected {
            assert!(self.faults.contains(fault), "fault never fell: {fault:?}");
        }
        assert_eq!(self.faults.len(), expected.len());
    }
}
/// Collect the strings actually handed up; written positions determine their order.
#[must_use]
pub fn declaration_texts(
    declaration: &hearth_protocol_config::declaration::Declaration,
) -> Vec<hearth_protocol_config::declaration::Text> {
    use hearth_protocol_config::schema::Form;
    let mut texts = Vec::new();
    for read in &declaration.reads {
        texts.push(read.task.clone());
        texts.extend_from_slice(&read.settings);
    }
    for guard in &declaration.requires {
        texts.extend([guard.name.clone(), guard.when.clone(), guard.message.clone()]);
    }
    if let Some(bless) = &declaration.bless {
        texts.extend_from_slice(&bless.ignore_failed);
    }
    for spec in &declaration.settings {
        texts.push(spec.name.clone());
    }
    for export in &declaration.exports {
        texts.extend([export.name.clone(), export.value.clone()]);
    }
    for ty in &declaration.types {
        match &ty.form {
            Form::Object(fields) => texts.extend(fields.iter().map(|field| field.name.clone())),
            Form::Enum(values) => texts.extend_from_slice(values),
            Form::Bool
            | Form::Int
            | Form::String
            | Form::Path
            | Form::Mode
            | Form::Duration
            | Form::Secret
            | Form::Target
            | Form::File
            | Form::List(_)
            | Form::Map(_) => {}
        }
    }
    for map in &declaration.entries {
        for entry in &map.entries {
            texts.push(entry.key.clone());
            texts.extend(entry.when.clone());
            texts.extend(entry.for_each.clone());
            for field in &entry.fields {
                literal_texts(&field.value, &mut texts);
            }
        }
    }
    texts.sort_by_key(|text| text.at.offset);
    texts
}
fn literal_texts(
    value: &hearth_protocol_config::entries::Literal,
    texts: &mut Vec<hearth_protocol_config::declaration::Text>,
) {
    use hearth_protocol_config::entries::Literal;
    match value {
        Literal::Text(text) | Literal::Plain(text) | Literal::Typed(text) => texts.push(text.clone()),
        Literal::Strings { each, .. } => texts.extend_from_slice(each),
        Literal::Range { fields, .. } => {
            for field in fields {
                literal_texts(&field.value, texts);
            }
        }
        Literal::Int(_) | Literal::Bool { .. } => {}
    }
}
/// Valid declarations that fill each schema allocation at these test-sized limits.
#[must_use]
pub fn declarations_at_the_limits(limits: &Limits) -> Vec<(Vec<u8>, &'static str)> {
    let spec = "{\"type\":\"int\",\"doc\":\"d\"}";
    let settings = (0..limits.settings).map(|n| format!("\"s{n}\":{spec}")).collect::<Vec<_>>().join(",");
    let fields = (0..limits.keys).map(|n| format!("\"f{n}\":{spec}")).collect::<Vec<_>>().join(",");
    let exports = (0..limits.keys)
        .map(|n| format!("\"e{n}\":{{\"type\":{{\"map\":\"int\"}},\"doc\":\"d\",\"value\":\"x\"}}"))
        .collect::<Vec<_>>()
        .join(",");
    let deep = format!(
        "{}\"int\"{}",
        "{\"list\":".repeat(limits.depth.saturating_sub(3) as usize),
        "}".repeat(limits.depth.saturating_sub(3) as usize)
    );
    vec![
        (format!("{{\"settings\":{{{settings}}}}}").into_bytes(), "most settings"),
        (
            format!("{{\"settings\":{{\"s\":{{\"type\":{{\"object\":{{{fields}}}}},\"doc\":\"d\"}}}}}}").into_bytes(),
            "most object fields",
        ),
        (format!("{{\"exports\":{{{exports}}}}}").into_bytes(), "most exports"),
        (format!("{{\"settings\":{{\"s\":{{\"type\":{deep},\"doc\":\"d\"}}}}}}").into_bytes(), "deepest types"),
    ]
}
