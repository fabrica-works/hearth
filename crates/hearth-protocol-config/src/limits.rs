//! Configured bounds (domain/render.md, 2.3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub document: u32,
    pub depth: u32,
    pub keys: u32,
    pub settings: u32,
}
pub const SHIPPED: Limits = Limits { document: 262_144, depth: 32, keys: 4_096, settings: 512 };
#[must_use]
pub fn tokenizer(limits: &Limits) -> skein_json::tokenizer::Limits {
    skein_json::tokenizer::Limits {
        length: limits.document,
        depth: limits.depth,
        string: limits.document,
        number: limits.document,
        chunk: 4_096,
    }
}
