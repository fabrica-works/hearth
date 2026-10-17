//! Static trees translated without interpreting them (render.md, 4.2).
use crate::{records::parts::config as target, render::parts::config as source};
#[must_use]
pub fn config(config: source::Config) -> target::Config {
    let source::Config { tree } = config;
    let mut translated = skein_lib::List::with_capacity(u32::try_from(tree.len()).expect("bounded tree"));
    for source::TreeEntry { path, node } in tree {
        let node = match node {
            source::Node::Link { target, owner, group } => target::Node::Link { target, owner, group },
            source::Node::Directory { mode, owner, group } => target::Node::Directory { mode, owner, group },
            source::Node::File { content_digest, mode, owner, group } => {
                target::Node::File { content_digest: content_digest.0, mode, owner, group }
            }
        };
        translated.push(target::TreeEntry { path, node }).expect("tree capacity");
    }
    target::Config { tree: translated.into_boxed() }
}
