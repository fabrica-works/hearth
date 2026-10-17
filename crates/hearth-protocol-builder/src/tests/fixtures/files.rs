use crate::render::parts::config::Node;
#[must_use]
pub fn tree_nodes() -> [Node; 2] {
    [
        Node::Link {
            target: b"/proc/self/mounts".as_slice().into(),
            owner: b"root".as_slice().into(),
            group: b"root".as_slice().into(),
        },
        Node::Directory { mode: 0o750, owner: b"app".as_slice().into(), group: b"adm".as_slice().into() },
    ]
}
