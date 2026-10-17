use crate::{
    form::{json::field, text::Lines},
    parts::data::{DataChange, DataKeyed},
};
impl DataChange {
    pub(crate) fn text(&self, lines: &mut Lines) {
        for entry in &self.entries {
            let (mark, key, value) = match entry {
                DataKeyed::Added { key, value } => (b"+", key, value),
                DataKeyed::Removed { key, value } => (b"-", key, value),
                DataKeyed::Changed { key, after, .. } => (b"~", key, after),
            };
            lines.put(b"  data     ");
            lines.put(mark);
            lines.put(b" ");
            lines.put(key);
            lines.pad_to(52);
            lines.put(value.class.name());
            lines.put(b" ");
            lines.put(&value.path);
            lines.put(b"\n");
        }
    }
    pub(crate) fn json(&self, json: &mut skein_json::writer::Encoder) {
        json.object_start();
        field(json, b"kind", b"data");
        json.key(b"data");
        json.object_start();
        json.key(b"entries");
        json.array_start();
        for entry in &self.entries {
            json.object_start();
            match entry {
                DataKeyed::Added { key, value } => {
                    field(json, b"key", key);
                    field(json, b"kind", b"added");
                    json.key(b"value");
                    value.json(json);
                }
                DataKeyed::Removed { key, value } => {
                    field(json, b"key", key);
                    field(json, b"kind", b"removed");
                    json.key(b"value");
                    value.json(json);
                }
                DataKeyed::Changed { key, before, after } => {
                    json.key(b"after");
                    after.json(json);
                    json.key(b"before");
                    before.json(json);
                    field(json, b"key", key);
                    field(json, b"kind", b"changed");
                }
            }
            json.object_end();
        }
        json.array_end();
        json.object_end();
        json.object_end();
    }
}
