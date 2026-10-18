use super::{ConfigChange, LineKeyed, PresetKeyed, Script, ScriptsChange};
use crate::form::{json::field, text::Lines};
use skein_json::writer::Encoder;
fn header(lines: &mut Lines, mark: &[u8], list: &[u8], key: &[u8]) {
    lines.put(b"  config   ");
    lines.put(mark);
    lines.put(b" ");
    lines.put(list);
    if !key.is_empty() {
        lines.put(b" ");
        lines.put(key);
    }
    lines.pad_to(52);
}
impl PresetKeyed {
    fn text(&self, lines: &mut Lines) {
        let (mark, key, value) = match self {
            Self::Added { key, value } => (b"+", key, value),
            Self::Removed { key, value } => (b"-", key, value),
            Self::Changed { key, after, .. } => (b"~", key, after),
        };
        header(lines, mark, b"presets", key);
        value.text(lines);
        lines.put(b"\n");
    }
    fn json(&self, json: &mut Encoder) {
        json.object_start();
        match self {
            Self::Added { key, value } | Self::Removed { key, value } => {
                field(json, b"key", key);
                field(
                    json,
                    b"kind",
                    match self {
                        Self::Added { .. } => b"added",
                        Self::Removed { .. } => b"removed",
                        Self::Changed { .. } => unreachable!("added or removed"),
                    },
                );
                json.key(b"value");
                value.json(json);
            }
            Self::Changed { key, before, after } => {
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
}
impl LineKeyed {
    fn text(&self, lines: &mut Lines) {
        let (mark, key, value) = match self {
            Self::Added { key, value } => (b"+", key, value),
            Self::Removed { key, value } => (b"-", key, value),
            Self::Changed { key, after, .. } => (b"~", key, after),
        };
        header(lines, mark, b"tmpfiles", key);
        value.text(lines);
        lines.put(b"\n");
    }
    fn json(&self, json: &mut Encoder) {
        json.object_start();
        match self {
            Self::Added { key, value } | Self::Removed { key, value } => {
                field(json, b"key", key);
                field(
                    json,
                    b"kind",
                    match self {
                        Self::Added { .. } => b"added",
                        Self::Removed { .. } => b"removed",
                        Self::Changed { .. } => unreachable!("added or removed"),
                    },
                );
                json.key(b"value");
                value.json(json);
            }
            Self::Changed { key, before, after } => {
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
}
impl ScriptsChange {
    pub(crate) fn text(&self, lines: &mut Lines, part: &[u8]) {
        for row in &self.before {
            if script(&self.after, &row.task).is_none() {
                crate::diff::placed::header(lines, part, b"scripts", true, false);
                lines.pad_to(52);
                row.text(lines);
                lines.put(b"\n");
            }
        }
        for row in &self.after {
            let mark = match script(&self.before, &row.task) {
                Some(before) if before == row => continue,
                Some(_) => b"~",
                None => b"+",
            };
            crate::diff::placed::header(lines, part, b"scripts", mark != b"+", true);
            lines.pad_to(52);
            row.text(lines);
            lines.put(b"\n");
        }
    }
    pub(crate) fn json(&self, json: &mut Encoder) {
        json.object_start();
        json.key(b"after");
        scripts_json(&self.after, json);
        json.key(b"before");
        scripts_json(&self.before, json);
        json.object_end();
    }
}
fn script<'a>(scripts: &'a [Script], task: &[u8]) -> Option<&'a Script> {
    let mut found = None;
    for script in scripts {
        if script.task.as_ref() == task {
            found = Some(script);
        }
    }
    found
}
fn scripts_json(scripts: &[Script], json: &mut Encoder) {
    json.array_start();
    for script in scripts {
        script.json(json);
    }
    json.array_end();
}
impl ConfigChange {
    pub(super) fn directives_text(&self, lines: &mut Lines) {
        for row in &self.presets {
            row.text(lines);
        }
        for row in &self.tmpfiles {
            row.text(lines);
        }
        if let Some(scripts) = &self.scripts {
            scripts.text(lines, b"config");
        }
    }
    pub(super) fn directives_json(&self, json: &mut Encoder) {
        json.key(b"presets");
        json.array_start();
        for row in &self.presets {
            row.json(json);
        }
        json.array_end();
        if let Some(scripts) = &self.scripts {
            json.key(b"scripts");
            scripts.json(json);
        }
        json.key(b"tmpfiles");
        json.array_start();
        for row in &self.tmpfiles {
            row.json(json);
        }
        json.array_end();
    }
}
