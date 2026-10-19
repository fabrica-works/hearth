//! Typed packages differences (domain/render.md, 6.1).
use super::{Apt, Box, Initramfs, Lines, Preseed, Sources, Variant, apt, initramfs, names, preseeds, sources, variant};
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Change {
    pub resolution: crate::diff::ResolutionChange,
    pub request: Names,
    pub seeds: SeedsChange,
    pub sources: Option<SourcesPair>,
    pub variant: Option<VariantPair>,
    pub preseeds: Option<PreseedsPair>,
    pub initramfs: Option<InitramfsPair>,
    pub apt: Option<AptPair>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Names {
    pub added: Box<[Box<[u8]>]>,
    pub removed: Box<[Box<[u8]>]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SourcesPair {
    pub before: Sources,
    pub after: Sources,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct VariantPair {
    pub before: Variant,
    pub after: Variant,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PreseedsPair {
    pub before: Box<[Preseed]>,
    pub after: Box<[Preseed]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct InitramfsPair {
    pub before: Option<Initramfs>,
    pub after: Option<Initramfs>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct AptPair {
    pub before: Apt,
    pub after: Apt,
}

impl Change {
    pub(crate) fn json(&self, json: &mut skein_json::writer::Encoder) {
        json.object_start();
        crate::form::json::field(json, b"kind", b"packages");
        json.key(b"packages");
        json.object_start();
        if let Some(pair) = &self.apt {
            pair_json(json, b"apt", &pair.before, &pair.after, apt);
        }
        if let Some(pair) = &self.initramfs {
            pair_json(json, b"initramfs", pair.before.as_ref(), pair.after.as_ref(), initramfs);
        }
        if let Some(pair) = &self.preseeds {
            pair_json(json, b"preseeds", pair.before.as_ref(), pair.after.as_ref(), preseeds);
        }
        if !self.request.added.is_empty() || !self.request.removed.is_empty() {
            json.key(b"request");
            json.object_start();
            json.key(b"added");
            names(json, &self.request.added);
            json.key(b"removed");
            names(json, &self.request.removed);
            json.object_end();
        }
        json.key(b"seeds");
        self.seeds.json(json);
        json.key(b"resolution");
        self.resolution.json(json);
        if let Some(pair) = &self.sources {
            pair_json(json, b"sources", &pair.before, &pair.after, sources);
        }
        if let Some(pair) = &self.variant {
            pair_json(json, b"variant", pair.before, pair.after, variant);
        }
        json.object_end();
        json.object_end();
    }
    pub(crate) fn text(&self, lines: &mut Lines) {
        self.resolution.text(lines);
        self.seeds.text(lines);
        for (sign, names) in [(b"+".as_slice(), &self.request.added), (b"-", &self.request.removed)] {
            if !names.is_empty() {
                lines.put(b"  packages ");
                lines.put(sign);
                for name in names {
                    lines.put(b" ");
                    lines.put(name);
                }
                lines.put(b"\n");
            }
        }
        if let Some(pair) = &self.sources {
            pair_text(lines, b"sources", &pair.before, &pair.after, sources);
        }
        if let Some(pair) = &self.variant {
            pair_text(lines, b"variant", pair.before, pair.after, variant);
        }
        if let Some(pair) = &self.preseeds {
            pair_text(lines, b"preseeds", pair.before.as_ref(), pair.after.as_ref(), preseeds);
        }
        if let Some(pair) = &self.initramfs {
            pair_text(lines, b"initramfs", pair.before.as_ref(), pair.after.as_ref(), initramfs);
        }
        if let Some(pair) = &self.apt {
            pair_text(lines, b"apt", &pair.before, &pair.after, apt);
        }
    }
}
fn pair_json<T: Copy>(
    json: &mut skein_json::writer::Encoder,
    name: &[u8],
    before: T,
    after: T,
    encode: fn(&mut skein_json::writer::Encoder, T),
) {
    json.key(name);
    json.object_start();
    json.key(b"after");
    encode(json, after);
    json.key(b"before");
    encode(json, before);
    json.object_end();
}
fn pair_text<T: Copy>(
    lines: &mut Lines,
    name: &[u8],
    before: T,
    after: T,
    encode: fn(&mut skein_json::writer::Encoder, T),
) {
    lines.put(b"  packages ~ ");
    lines.put(name);
    lines.put(b" ");
    value_text(lines, before, encode);
    lines.put(" → ".as_bytes());
    value_text(lines, after, encode);
    lines.put(b"\n");
}
pub(crate) fn value_text<T: Copy>(lines: &mut Lines, value: T, encode: fn(&mut skein_json::writer::Encoder, T)) {
    let limits = skein_json::writer::Limits { length: u32::MAX, depth: 16 };
    let mut measure = skein_json::writer::Encoder::measure(&limits);
    encode(&mut measure, value);
    let mut json = skein_json::writer::Encoder::write(measure.measured().expect("typed field"), &limits);
    encode(&mut json, value);
    lines.put(&json.finish());
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SeedsChange {
    pub users: Box<[SeedUserKeyed]>,
    pub groups: Box<[SeedGroupKeyed]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SeedIds {
    pub uid: u32,
    pub gid: Option<u32>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum SeedUserKeyed {
    Added { key: Box<[u8]>, value: SeedIds },
    Removed { key: Box<[u8]>, value: SeedIds },
    Changed { key: Box<[u8]>, before: SeedIds, after: SeedIds },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum SeedGroupKeyed {
    Added { key: Box<[u8]>, value: u32 },
    Removed { key: Box<[u8]>, value: u32 },
    Changed { key: Box<[u8]>, before: u32, after: u32 },
}

mod fixed;
