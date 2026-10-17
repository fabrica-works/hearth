//! The static file tree's canonical record (render.md, 4.2).
use crate::revision::Mode;
use alloc::boxed::Box;
use hearth_lib::{
    encode::{Digest, Encoder},
    hash::H,
};
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Config {
    pub tree: Box<[TreeEntry]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TreeEntry {
    pub path: Box<[u8]>,
    pub node: Node,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Node {
    File { content_digest: Digest, mode: u16, owner: Box<[u8]>, group: Box<[u8]> },
}
#[must_use]
pub fn file(path: &[u8], mode: Mode, content_digest: Digest) -> TreeEntry {
    TreeEntry {
        path: absolute(path),
        node: Node::File {
            content_digest,
            mode: if mode == Mode::Executable { 0o755 } else { 0o644 },
            owner: skein_lib::bytes::copy_of(b"root"),
            group: skein_lib::bytes::copy_of(b"root"),
        },
    }
}
#[must_use]
pub fn digest(config: &Config) -> Digest {
    let mut hash = H::new(b"part:config", 1);
    encode(config, hash.values());
    hash.finish()
}
pub fn encode(config: &Config, record: &mut Encoder) {
    record.map(1);
    record.key(b"tree");
    record.map(u64::try_from(config.tree.len()).expect("bounded tree"));
    for entry in &config.tree {
        record.key(&entry.path);
        match &entry.node {
            Node::File { content_digest, mode, owner, group } => {
                record.map(5);
                record.key(b"content_digest");
                record.digest(content_digest);
                record.key(b"group");
                record.bytes(group);
                record.key(b"kind");
                record.bytes(b"file");
                record.key(b"mode");
                record.uint(u64::from(*mode));
                record.key(b"owner");
                record.bytes(owner);
            }
        }
    }
}

fn absolute(path: &[u8]) -> Box<[u8]> {
    let mut writer = skein_lib::Writer::new(path.len().saturating_add(1));
    writer.put(b"/").expect("prefix room");
    writer.put(path).expect("path room");
    writer.finish()
}
