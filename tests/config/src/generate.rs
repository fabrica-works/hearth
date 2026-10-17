//! Bounded generated objects, and hostile mutations covering hearth's refusals.
use hearth_protocol_config::limits::Limits;
use skein_lib::Rng;
use std::fmt::Write;
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

/// A generated declaration and every string handed up, at its written position.
#[derive(Debug)]
pub struct Written {
    pub bytes: Vec<u8>,
    pub texts: Vec<hearth_protocol_config::declaration::Text>,
    spans: Vec<(&'static str, usize, usize)>,
    setting_limit: u32,
}
struct Writing {
    bytes: Vec<u8>,
    texts: Vec<hearth_protocol_config::declaration::Text>,
}
impl Writing {
    fn raw(&mut self, value: &str) {
        self.bytes.extend_from_slice(value.as_bytes());
    }
    fn text(&mut self, value: &str) {
        let at = crate::reference::position(&self.bytes, self.bytes.len());
        self.texts.push(crate::fixtures::text(at, value.as_bytes()));
        self.raw(&format!("\"{value}\""));
    }
}
const MAPS: &[(&str, &[&str])] = &[
    ("packages", &["component"]),
    ("preseeds", &["owner", "type", "value"]),
    ("initramfs", &[]),
    ("software", &["url"]),
    ("files", &["template"]),
    ("users", &["uid", "groups", "subordinate"]),
    ("groups", &["gid"]),
    ("presets", &["action"]),
    ("tmpfiles", &["type"]),
    ("data", &["path"]),
    ("secrets", &["path"]),
    ("identity", &["kind"]),
    ("cmdline", &["order"]),
    ("live", &["paths"]),
];
#[must_use]
pub fn declaration(rng: &mut Rng, limits: &Limits) -> Written {
    let mut w = Writing { bytes: Vec::new(), texts: Vec::new() };
    let mut spans = Vec::new();
    w.raw("{");
    let mut sections: Vec<(&str, String)> =
        vec![("doc", "\"d\"".into()), ("scripts", "{\"customize.sh\":{\"order\":300}}".into())];
    // These fragments use unique text values; the writer records each marked string as it writes it.
    sections.extend([
        ("bless", "{\"ignore_failed\":[@unit]}".into()),
        ("reads", "{@task:{\"settings\":[@setting],\"exports\":true,\"optional\":true}}".into()),
        ("requires", "{@guard:{\"if\":@condition,\"message\":@message}}".into()),
        ("exports", "{@export:{\"type\":{\"list\":\"int\"},\"doc\":\"d\",\"value\":@expression}}".into()),
    ]);
    let count = rng.between(1, u64::from(limits.settings));
    let mut settings = Vec::new();
    for n in 0..count {
        settings.push(format!("@s{n}:{{\"type\":{{\"map\":{{\"object\":{{@f{n}:{{\"type\":{{\"enum\":[@e{n}]}},\"doc\":\"d\",\"default\":0}}}}}}}},\"doc\":\"d\",\"optional\":true}}"));
    }
    sections.push(("settings", format!("{{{}}}", settings.join(","))));
    for &(map, fields) in MAPS {
        let mut entry = Vec::new();
        for field in fields {
            entry.push(format!("\"{field}\":@{map}_{field}"));
        }
        sections.push((map, format!("{{@{map}_key:{{{}}}}}", entry.join(","))));
    }
    for (index, (name, fragment)) in sections.into_iter().enumerate() {
        if index > 0 {
            w.raw(",\n");
        }
        w.raw(&format!("\"{name}\":"));
        let start = w.bytes.len();
        let mut cursor = 0;
        for (offset, _) in fragment.match_indices('@') {
            if offset < cursor {
                continue;
            }
            w.raw(&fragment[cursor..offset]);
            let length = fragment[offset + 1..].bytes().take_while(|b| b.is_ascii_alphanumeric() || *b == b'_').count();
            let marker = &fragment[offset + 1..offset + 1 + length];
            w.text(&format!("{marker}_{}", rng.below(1000)));
            cursor = offset + 1 + length;
        }
        w.raw(&fragment[cursor..]);
        spans.push((name, start, w.bytes.len()));
    }
    w.raw("}");
    Written { bytes: w.bytes, texts: w.texts, spans, setting_limit: limits.settings }
}
/// One injected declaration fault, with the place selected by the generator.
#[derive(Debug)]
pub struct Broken {
    pub bytes: Vec<u8>,
    pub at: hearth_protocol_config::Position,
    pub fault: hearth_protocol_config::Declared,
}

use hearth_protocol_config::{Declared, Expected, Object, Required};
const BREAKS: &[(&str, &str, usize, Declared)] = &[
    ("doc", r#""d","bad":0"#, 4, Declared::Unknown { of: Object::Declaration }),
    ("reads", r#"{"x":{"bad":0}}"#, 6, Declared::Unknown { of: Object::Read }),
    ("requires", r#"{"x":{"bad":0}}"#, 6, Declared::Unknown { of: Object::Guard }),
    ("scripts", r#"{"customize.sh":{"bad":0}}"#, 17, Declared::Unknown { of: Object::Script }),
    ("bless", r#"{"bad":0}"#, 1, Declared::Unknown { of: Object::Bless }),
    (
        "packages",
        r#"{"k":{"bad":0}}"#,
        6,
        Declared::Unknown { of: Object::Entry(hearth_protocol_config::entries::Map::Packages) },
    ),
    (
        "preseeds",
        r#"{"k":{"bad":0}}"#,
        6,
        Declared::Unknown { of: Object::Entry(hearth_protocol_config::entries::Map::Preseeds) },
    ),
    (
        "initramfs",
        r#"{"k":{"bad":0}}"#,
        6,
        Declared::Unknown { of: Object::Entry(hearth_protocol_config::entries::Map::Initramfs) },
    ),
    (
        "software",
        r#"{"k":{"bad":0}}"#,
        6,
        Declared::Unknown { of: Object::Entry(hearth_protocol_config::entries::Map::Software) },
    ),
    (
        "files",
        r#"{"k":{"bad":0}}"#,
        6,
        Declared::Unknown { of: Object::Entry(hearth_protocol_config::entries::Map::Files) },
    ),
    (
        "users",
        r#"{"k":{"bad":0}}"#,
        6,
        Declared::Unknown { of: Object::Entry(hearth_protocol_config::entries::Map::Users) },
    ),
    (
        "groups",
        r#"{"k":{"bad":0}}"#,
        6,
        Declared::Unknown { of: Object::Entry(hearth_protocol_config::entries::Map::Groups) },
    ),
    (
        "presets",
        r#"{"k":{"bad":0}}"#,
        6,
        Declared::Unknown { of: Object::Entry(hearth_protocol_config::entries::Map::Presets) },
    ),
    (
        "tmpfiles",
        r#"{"k":{"bad":0}}"#,
        6,
        Declared::Unknown { of: Object::Entry(hearth_protocol_config::entries::Map::Tmpfiles) },
    ),
    (
        "data",
        r#"{"k":{"bad":0}}"#,
        6,
        Declared::Unknown { of: Object::Entry(hearth_protocol_config::entries::Map::Data) },
    ),
    (
        "secrets",
        r#"{"k":{"bad":0}}"#,
        6,
        Declared::Unknown { of: Object::Entry(hearth_protocol_config::entries::Map::Secrets) },
    ),
    (
        "identity",
        r#"{"k":{"bad":0}}"#,
        6,
        Declared::Unknown { of: Object::Entry(hearth_protocol_config::entries::Map::Identity) },
    ),
    (
        "cmdline",
        r#"{"k":{"bad":0}}"#,
        6,
        Declared::Unknown { of: Object::Entry(hearth_protocol_config::entries::Map::Cmdline) },
    ),
    (
        "live",
        r#"{"k":{"bad":0}}"#,
        6,
        Declared::Unknown { of: Object::Entry(hearth_protocol_config::entries::Map::Live) },
    ),
    ("users", r#"{"k":{"subordinate":{"bad":0}}}"#, 21, Declared::Unknown { of: Object::Range }),
    ("settings", r#"{"s":{"type":"int","doc":"d","bad":0}}"#, 29, Declared::Unknown { of: Object::Spec }),
    ("exports", r#"{"e":{"type":"int","doc":"d","value":"x","bad":0}}"#, 41, Declared::Unknown { of: Object::Export }),
    ("doc", r#""d","checks":{}"#, 4, Declared::Later),
    ("reads", "0", 0, Declared::Wrong { expected: Expected::Object }),
    ("requires", r#"{"x":{"message":"d"}}"#, 5, Declared::Missing { key: Required::If }),
    ("reads", r#"{"Bad":{}}"#, 1, Declared::Name),
    ("doc", r#""a\nb""#, 0, Declared::Doc),
    ("scripts", r#"{"customize.sh":{"order":-1}}"#, 25, Declared::Order),
    ("settings", r#"{"s":{"type":"int","doc":"d","default":0,"optional":true}}"#, 41, Declared::Optional),
    ("settings", r#"{"s":{"type":"bool","doc":"d","min":0}}"#, 30, Declared::Bound),
    ("settings", r#"{"s":{"type":"int","doc":"d","min":1,"max":0}}"#, 35, Declared::Above),
    ("settings", r#"{"s":{"type":{"enum":[]},"doc":"d"}}"#, 21, Declared::Enum),
    ("exports", r#"{"e":{"type":{"list":"secret"},"doc":"d","value":"x"}}"#, 21, Declared::Secret),
];
#[must_use]
pub fn break_once(rng: &mut Rng, written: &Written) -> Broken {
    let selected = usize::try_from(rng.below(BREAKS.len() as u64 + 1)).expect("bounded case index");
    let (name, replacement, offset, fault) = if selected == BREAKS.len() {
        let mut settings = String::from("{");
        for n in 0..written.setting_limit {
            if n > 0 {
                settings.push(',');
            }
            write!(settings, "\"s{n}\":{{\"type\":\"bool\",\"doc\":\"d\"}}").expect("String write");
        }
        settings.push(',');
        let offset = settings.len();
        settings.push_str("\"zzzz\":{\"type\":\"bool\",\"doc\":\"d\"}}");
        ("settings", settings, offset, Declared::Settings)
    } else {
        let &(name, replacement, offset, fault) = &BREAKS[selected];
        (name, replacement.to_owned(), offset, fault)
    };
    let &(_, start, end) = written.spans.iter().find(|(section, _, _)| *section == name).expect("written section");
    let mut bytes = written.bytes[..start].to_vec();
    bytes.extend_from_slice(replacement.as_bytes());
    bytes.extend_from_slice(&written.bytes[end..]);
    let at = crate::reference::position(&bytes, start + offset);
    Broken { bytes, at, fault }
}
