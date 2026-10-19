//! The packages closure's change in client forms (domain/render.md, 6.1).
use crate::{
    diff::{Listed, Moved, ResolutionChange},
    form::text::Lines,
};
use skein_json::writer::Encoder;
impl ResolutionChange {
    pub(crate) fn json(&self, json: &mut Encoder) {
        json.object_start();
        match self {
            Self::Same => super::json::field(json, b"kind", b"same"),
            Self::NewKey => super::json::field(json, b"kind", b"new_key"),
            Self::Resolved { added, removed, versions_changed } => {
                json.key(b"added");
                listed(json, added);
                super::json::field(json, b"kind", b"resolved");
                json.key(b"removed");
                listed(json, removed);
                json.key(b"versions_changed");
                json.array_start();
                for Moved { name, architecture, before, after } in versions_changed {
                    json.object_start();
                    super::json::field(json, b"after", after);
                    super::json::field(json, b"architecture", architecture);
                    super::json::field(json, b"before", before);
                    super::json::field(json, b"name", name);
                    json.object_end();
                }
                json.array_end();
            }
        }
        json.object_end();
    }
    pub(crate) fn text(&self, lines: &mut Lines) {
        lines.put(b"  packages   resolution ");
        match self {
            Self::Same => lines.put(b"same\n"),
            Self::NewKey => lines.put(b"new key\n"),
            Self::Resolved { added, removed, versions_changed } => {
                lines.put(b"new key, resolved\n");
                for (sign, packages) in [(b"+".as_slice(), added), (b"-", removed)] {
                    for Listed { name, architecture, version } in packages {
                        lines.put(b"    ");
                        lines.put(sign);
                        lines.put(b" ");
                        package_text(lines, name, architecture);
                        lines.put(b" ");
                        lines.put(version);
                        lines.put(b"\n");
                    }
                }
                for Moved { name, architecture, before, after } in versions_changed {
                    lines.put(b"    ~ ");
                    package_text(lines, name, architecture);
                    lines.put(b" ");
                    lines.put(before);
                    lines.put(" → ".as_bytes());
                    lines.put(after);
                    lines.put(b"\n");
                }
            }
        }
    }
}
fn package_text(lines: &mut Lines, name: &[u8], architecture: &[u8]) {
    lines.put(name);
    lines.put(b":");
    lines.put(architecture);
}
fn listed(json: &mut Encoder, packages: &[Listed]) {
    json.array_start();
    for Listed { name, architecture, version } in packages {
        json.object_start();
        super::json::field(json, b"architecture", architecture);
        super::json::field(json, b"name", name);
        super::json::field(json, b"version", version);
        json.object_end();
    }
    json.array_end();
}
