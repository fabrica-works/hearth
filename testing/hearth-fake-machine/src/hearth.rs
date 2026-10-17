//! The program table above the generic machine (testing.md, 3.5).
pub mod universe;
pub mod version;
use crate::linux::{self, Exit, Item, Node, Op, Pid, Refusal, Reply, Sizes};
use alloc::collections::BTreeMap;
#[derive(Debug)]
pub struct Machine {
    linux: linux::Machine,
    programs: BTreeMap<Node, Tool>,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Tool {
    Git,
}
impl Machine {
    #[must_use]
    pub fn new(seed: u64, sizes: Sizes) -> Self {
        Self { linux: linux::Machine::new(seed, sizes), programs: BTreeMap::new() }
    }
    pub fn install(&mut self, path: &[u8], tool: Tool) -> Node {
        let nodes = self.linux.lay(&[Item::file(path, 0o755, b"")]);
        let node = *nodes.first().expect("one installed file");
        self.programs.insert(node, tool);
        node
    }
    pub fn call(&mut self, pid: Pid, op: Op) -> Result<Reply, Refusal> {
        let result = self.linux.call(pid, op);
        if let Ok(Reply::Spawned { child, .. }) = &result
            && self.tool(*child).is_none()
        {
            self.linux.finish(*child, Exit::Code(127));
        }
        result
    }
    #[must_use]
    pub fn tool(&self, pid: Pid) -> Option<Tool> {
        self.programs.get(&self.linux.program(pid)).copied()
    }
    #[must_use]
    pub const fn linux(&self) -> &linux::Machine {
        &self.linux
    }
    pub const fn linux_mut(&mut self) -> &mut linux::Machine {
        &mut self.linux
    }
}
