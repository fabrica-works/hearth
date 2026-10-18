//! Check installed paths once every produced file has a known length.
use super::{Side, Target};
use crate::{
    parts::config::Node,
    reserved::{self, Installed, Shape},
    vocabulary::Outcome,
};
use skein_lib::List;
impl Target {
    pub(super) fn reserved(&self, side: &mut Side) -> bool {
        let mut installed = List::with_capacity(super::files::capacity(&self.plans));
        for plan in &self.plans {
            for install in &plan.installs {
                let Some(file) = self.tree.get(&install.path) else { continue };
                let shape = match &file.node {
                    Node::File { content_digest, .. } => {
                        Shape::File { empty: side.sizes.get(content_digest) == Some(&0) }
                    }
                    Node::Link { .. } => Shape::Link,
                    Node::Directory { .. } => Shape::Directory,
                };
                installed
                    .push(Installed { task: &plan.task, path: &install.path, at: install.at, shape })
                    .expect("plan bound");
            }
        }
        let faults = reserved::tree(installed.as_slice());
        let valid = faults.is_empty();
        for fault in faults {
            side.errors.push(reserved::error(fault, &self.meta));
        }
        if !valid {
            side.outcome = side.outcome.worst(Outcome::Failed);
        }
        valid
    }
}
