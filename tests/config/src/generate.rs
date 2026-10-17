//! Bounded generated objects, and hostile mutations covering hearth's refusals.
use hearth_protocol_config::limits::Limits;
use skein_lib::Rng;
#[must_use]
pub fn document(rng: &mut Rng, _limits: &Limits) -> Vec<u8> {
    object(rng, 0).into_bytes()
}
fn object(rng: &mut Rng, depth: u32) -> String {
    let mut entries = Vec::new();
    for n in 0..rng.below(5) {
        entries.push(format!("\"k{n}\":{}", value(rng, depth)));
    }
    if rng.chance(500) {
        entries.push("\"_note\":\"a note é\\n\"".to_owned());
    }
    format!("{{\r\n{}\n}}", entries.join(", "))
}
fn value(rng: &mut Rng, depth: u32) -> String {
    match rng.below(if depth < 3 { 7 } else { 5 }) {
        0 => "null".to_owned(),
        1 => "true".to_owned(),
        2 => "false".to_owned(),
        3 => format!("-{}", rng.below(1_000_000)),
        4 => "\"aé€😀\\u0062\\n\"".to_owned(),
        5 => object(rng, depth + 1),
        6 => {
            let mut values = Vec::new();
            for _ in 0..rng.below(6) {
                values.push(value(rng, depth + 1));
            }
            format!("[{}]", values.join(","))
        }
        _ => unreachable!(),
    }
}
const HOSTILE: &[&[u8]] = &[
    br#"{"a":1,"a":2}"#,
    br#"{"x":{"a":1,"a":2}}"#,
    br#"{"a":1.1}"#,
    br#"{"a":1e2}"#,
    br#"{"a":9223372036854775808}"#,
    br#"{"a":-9223372036854775809}"#,
    br#"{"a":"\u0000"}"#,
    br#"{"\u0000":1}"#,
    br#"{"_note":null}"#,
    b"[]",
    b"null",
    b"{",
    br#"{"a":"\q"}"#,
    br#"{"a":[1,]}"#,
    br#"{"a":[[[[[0]]]]]}"#,
    br#"{"a":0,"b":0,"c":0,"d":0,"e":0}"#,
];
#[must_use]
pub fn mutate(rng: &mut Rng, bytes: &[u8]) -> Vec<u8> {
    match rng.below(4) {
        0 | 1 => HOSTILE[usize::try_from(rng.below(HOSTILE.len() as u64)).expect("draw fits usize")].to_vec(),
        2 => bytes[..usize::try_from(rng.below(bytes.len() as u64 + 1)).expect("draw fits usize")].to_vec(),
        3 => {
            let mut out = bytes.to_vec();
            if !out.is_empty() {
                let i = usize::try_from(rng.below(out.len() as u64)).expect("draw fits usize");
                out[i] = b'!';
            }
            out
        }
        _ => unreachable!(),
    }
}
