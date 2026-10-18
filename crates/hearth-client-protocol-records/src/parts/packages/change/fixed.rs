//! Seed changes by name; text prints the new row (render.md, 6.1).
use super::{Lines, SeedGroupKeyed, SeedIds, SeedUserKeyed, SeedsChange};
use skein_json::writer::Encoder;
impl SeedUserKeyed {
    fn sides(&self) -> (&[u8], Option<&SeedIds>, Option<&SeedIds>) {
        match self {
            Self::Added { key, value } => (key, None, Some(value)),
            Self::Removed { key, value } => (key, Some(value), None),
            Self::Changed { key, before, after } => (key, Some(before), Some(after)),
        }
    }
}
impl SeedGroupKeyed {
    fn sides(&self) -> (&[u8], Option<&u32>, Option<&u32>) {
        match self {
            Self::Added { key, value } => (key, None, Some(value)),
            Self::Removed { key, value } => (key, Some(value), None),
            Self::Changed { key, before, after } => (key, Some(before), Some(after)),
        }
    }
}
impl SeedsChange {
    pub(super) fn text(&self, lines: &mut Lines) {
        for entry in &self.users {
            let (key, before, after) = entry.sides();
            seed_header(lines, b"user", key, before.is_some(), after.is_some());
            let ids = after.or(before).expect("seed row");
            lines.decimal(u64::from(ids.uid));
            lines.put(b" ");
            match ids.gid {
                Some(gid) => lines.decimal(u64::from(gid)),
                None => lines.put(b"-"),
            }
            lines.put(b"\n");
        }
        for entry in &self.groups {
            let (key, before, after) = entry.sides();
            seed_header(lines, b"group", key, before.is_some(), after.is_some());
            lines.decimal(u64::from(*after.or(before).expect("seed row")));
            lines.put(b"\n");
        }
    }
    pub(super) fn json(&self, json: &mut Encoder) {
        json.object_start();
        json.key(b"groups");
        json.array_start();
        for entry in &self.groups {
            let (key, before, after) = entry.sides();
            crate::diff::placed::keyed(json, key, before, after, |gid, json| json.unsigned(u64::from(*gid)));
        }
        json.array_end();
        json.key(b"users");
        json.array_start();
        for entry in &self.users {
            let (key, before, after) = entry.sides();
            crate::diff::placed::keyed(json, key, before, after, ids);
        }
        json.array_end();
        json.object_end();
    }
}
fn ids(ids: &SeedIds, json: &mut Encoder) {
    json.object_start();
    json.key(b"gid");
    match ids.gid {
        Some(gid) => json.unsigned(u64::from(gid)),
        None => json.null(),
    }
    json.key(b"uid");
    json.unsigned(u64::from(ids.uid));
    json.object_end();
}
fn seed_header(lines: &mut Lines, kind: &[u8], key: &[u8], before: bool, after: bool) {
    crate::diff::placed::header(lines, b"packages", b"seed ", before, after);
    lines.put(kind);
    lines.put(b" ");
    lines.put(key);
    lines.pad_to(52);
}
