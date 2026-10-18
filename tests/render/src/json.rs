//! A document model independent of the render domain (format.md, 2.2).
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
