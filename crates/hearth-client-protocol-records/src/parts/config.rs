//! The config part's file tree (render.md, 4.2).
use crate::form::text::Lines;
use alloc::boxed::Box;
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
    File { content_digest: [u8; 32], mode: u16, owner: Box<[u8]>, group: Box<[u8]> },
}
impl Config {
    pub(crate) fn text(&self, lines: &mut Lines) {
        let mut column = 27;
        for entry in &*self.tree {
            column = column.max(entry.path.len().saturating_add(4));
        }
        for entry in &*self.tree {
            lines.put(b"  ");
            lines.put(&entry.path);
            lines.pad_to(column);
            match &entry.node {
                Node::File { content_digest, mode, owner, group } => {
                    lines.put(b"file ");
                    lines.mode(*mode);
                    lines.put(b" ");
                    lines.put(owner);
                    lines.put(b":");
                    lines.put(group);
                    lines.put(b" ");
                    lines.hex12(content_digest);
                }
            }
            lines.put(b"\n");
        }
    }
}
