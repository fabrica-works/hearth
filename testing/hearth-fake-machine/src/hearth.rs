//! The program table above the generic machine (testing.md, 3.5).
pub mod draw;
pub mod mirror;
pub mod universe;
pub mod version;
use crate::linux::{self, Exit, Item, Node, Op, Pid, Refusal, Reply, Sizes};
use alloc::collections::BTreeMap;
use mirror::Mirror;
#[derive(Debug)]
pub struct Machine {
    linux: linux::Machine,
    programs: BTreeMap<Node, Tool>,
    served: Option<(Mirror, Option<u16>)>,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Tool {
    Git,
    Mmdebstrap,
}
impl Machine {
    #[must_use]
    pub fn new(seed: u64, sizes: Sizes) -> Self {
        Self { linux: linux::Machine::new(seed, sizes), programs: BTreeMap::new(), served: None }
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

impl Machine {
    /// Moves the mirror onto this machine, optionally behind one proxy port.
    pub fn serve(&mut self, mirror: Mirror, proxy: Option<u16>) {
        assert!(self.served.is_none(), "the world takes its mirror back before serving another");
        self.served = Some((mirror, proxy));
    }
    pub fn unserve(&mut self) -> Option<Mirror> {
        self.served.take().map(|(mirror, _)| mirror)
    }
    #[must_use]
    pub fn reach(&self, proxy: Option<u16>) -> Option<&Mirror> {
        let (mirror, served) = self.served.as_ref()?;
        if proxy.is_none() || proxy == *served { Some(mirror) } else { None }
    }
}
