//! Boot record forms (domain/render.md, 4.3).
use alloc::boxed::Box;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Change {
    pub before: Boot,
    pub after: Boot,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Boot {
    pub params: Box<[Box<[u8]>]>,
    pub mutable: bool,
}
impl Boot {
    pub(crate) fn text(&self, lines: &mut crate::form::text::Lines) {
        if !self.params.is_empty() {
            lines.put(b"  params    ");
            for param in &self.params {
                lines.put(b" ");
                lines.put(param);
            }
            lines.put(b"\n");
        }
        lines.put(b"  mutable    ");
        lines.put(if self.mutable { b"true" } else { b"false" });
        lines.put(b"\n");
    }
    pub(crate) fn json(&self, json: &mut skein_json::writer::Encoder) {
        json.object_start();
        json.key(b"mutable");
        json.boolean(self.mutable);
        json.key(b"params");
        json.array_start();
        for param in &self.params {
            json.string(param);
        }
        json.array_end();
        json.object_end();
    }
}

impl Change {
    pub(crate) fn json(&self, json: &mut skein_json::writer::Encoder) {
        json.object_start();
        crate::form::json::field(json, b"kind", b"boot");
        json.key(b"boot");
        json.object_start();
        json.key(b"after");
        self.after.json(json);
        json.key(b"before");
        self.before.json(json);
        json.object_end();
        json.object_end();
    }
    pub(crate) fn text(&self, lines: &mut crate::form::text::Lines) {
        lines.put(b"  boot     ~ ");
        super::packages::value_text(lines, &self.before, encode);
        lines.put(" → ".as_bytes());
        super::packages::value_text(lines, &self.after, encode);
        lines.put(b"\n");
    }
}

fn encode(json: &mut skein_json::writer::Encoder, value: &Boot) {
    value.json(json);
}
