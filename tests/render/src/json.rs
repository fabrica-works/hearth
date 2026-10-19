//! A document model independent of the render domain (protocol/format.md, 2.2).
use hearth_domain_render::{
    compose::{Composed, Composite},
    revision::{self, Position},
};
use skein_lib::Rng;
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Json {
    Null,
    Bool(bool),
    Int(i64),
    String(String),
    List(Vec<Json>),
    Map(Vec<(String, Json)>),
}
#[must_use]
pub fn object(entries: &[(&str, Json)]) -> Json {
    Json::Map(entries.iter().map(|(key, value)| ((*key).to_owned(), value.clone())).collect())
}
#[must_use]
pub fn string(text: &str) -> Json {
    Json::String(text.to_owned())
}
/// Small branching keeps the depth-eight sweep within 30.3's budget.
pub const WIDTH: u64 = 3;
#[must_use]
pub fn generate(rng: &mut Rng, depth: u32) -> Json {
    draw(rng, depth, true)
}
fn draw(rng: &mut Rng, depth: u32, deletions: bool) -> Json {
    match rng.below(if depth == 0 { 3 } else { 5 }) {
        0 => Json::Bool(rng.below(2) != 0),
        1 => Json::Int(i64::try_from(rng.below(256)).expect("small integer") - 128),
        2 => Json::String(format!("value{}", rng.below(16))),
        3 => Json::List((0..rng.below(WIDTH)).map(|_| draw(rng, depth - 1, false)).collect()),
        _ => Json::Map(
            (0..rng.below(WIDTH))
                .map(|index| {
                    let value =
                        if deletions && rng.below(WIDTH) == 0 { Json::Null } else { draw(rng, depth - 1, deletions) };
                    (format!("key{index}"), value)
                })
                .collect(),
        ),
    }
}
#[must_use]
pub fn value(json: &Json) -> revision::Value {
    located(json, &mut 0)
}
fn located(json: &Json, counter: &mut u32) -> revision::Value {
    *counter += 1;
    let at = Position { line: *counter, column: 1 };
    let kind = match json {
        Json::Null => revision::Kind::Null,
        Json::Bool(value) => revision::Kind::Bool(*value),
        Json::Int(value) => revision::Kind::Int(*value),
        Json::String(value) => revision::Kind::String(value.as_bytes().into()),
        Json::List(values) => revision::Kind::List(values.iter().map(|value| located(value, counter)).collect()),
        Json::Map(entries) => revision::Kind::Map(
            entries
                .iter()
                .map(|(key, value)| {
                    let value = located(value, counter);
                    revision::Pair { key: key.as_bytes().into(), at: value.at, value }
                })
                .collect(),
        ),
    };
    revision::Value { at, kind }
}
#[must_use]
pub fn plain(value: &Composed) -> Json {
    match &value.kind {
        Composite::Bool(value) => Json::Bool(*value),
        Composite::Int(value) => Json::Int(*value),
        Composite::String(value) => Json::String(String::from_utf8(value.to_vec()).expect("generated UTF-8")),
        Composite::List(values) => Json::List(values.iter().map(plain).collect()),
        Composite::Map(entries) => Json::Map(
            entries
                .iter()
                .map(|pair| (String::from_utf8(pair.key.to_vec()).expect("generated key"), plain(&pair.value)))
                .collect(),
        ),
    }
}
#[must_use]
pub fn print(rng: &mut Rng, json: &Json) -> Vec<u8> {
    let mut out = String::new();
    write(rng, json, &mut out);
    out.into_bytes()
}
fn quoted(text: &str, out: &mut String) {
    out.push('"');
    for character in text.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            character => out.push(character),
        }
    }
    out.push('"');
}
fn gap(rng: &mut Rng, out: &mut String) {
    out.push_str(match rng.below(4) {
        0 => "",
        1 => " ",
        2 => "\n",
        _ => "\t",
    });
}
fn write(rng: &mut Rng, value: &Json, out: &mut String) {
    match value {
        Json::Null => out.push_str("null"),
        Json::Bool(value) => out.push_str(if *value { "true" } else { "false" }),
        Json::Int(value) => out.push_str(&value.to_string()),
        Json::String(value) => quoted(value, out),
        Json::List(values) => {
            out.push('[');
            for (index, value) in values.iter().enumerate() {
                if index != 0 {
                    out.push(',');
                }
                gap(rng, out);
                write(rng, value, out);
            }
            gap(rng, out);
            out.push(']');
        }
        Json::Map(entries) => {
            out.push('{');
            let mut entries: Vec<_> = entries.iter().collect();
            for index in (1..entries.len()).rev() {
                let chosen = usize::try_from(rng.below(u64::try_from(index + 1).expect("map size"))).expect("index");
                entries.swap(index, chosen);
            }
            for (index, (key, value)) in entries.into_iter().enumerate() {
                if index != 0 {
                    out.push(',');
                }
                gap(rng, out);
                quoted(key, out);
                out.push(':');
                gap(rng, out);
                write(rng, value, out);
            }
            if !entries_empty(value) {
                out.push(',');
            }
            quoted("_note", out);
            out.push(':');
            quoted(&format!("draw {}", rng.below(1_000_000)), out);
            gap(rng, out);
            out.push('}');
        }
    }
}
fn entries_empty(value: &Json) -> bool {
    matches!(value, Json::Map(entries) if entries.is_empty())
}
