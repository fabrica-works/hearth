use crate::{records, render};
pub(super) fn config_back(config: records::parts::config::Config) -> render::parts::config::Config {
    let mut tree = skein_lib::List::with_capacity(u32::try_from(config.tree.len()).expect("bounded tree"));
    for records::parts::config::TreeEntry { path, node } in config.tree {
        let node = super::node_back(node);
        tree.push(render::parts::config::TreeEntry { path, node }).expect("tree room");
    }
    render::parts::config::Config { tree: tree.into_boxed() }
}
