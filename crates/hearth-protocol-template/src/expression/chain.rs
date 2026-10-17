//! Straight lookup links, with index operands kept above the outer chain (format.md, 7.6).
use crate::{Missing, Op, Operation};
use skein_lib::List;
#[derive(Debug)]
pub(super) struct Chain {
    pub links: List<u32>,
    pub start: u32,
    pub active: bool,
}
impl Chain {
    pub fn end(&mut self) {
        self.links.truncate(self.start);
        self.active = false;
    }
    pub fn link(&mut self, index: u32) {
        if self.active {
            self.links.push(index).expect("links fit text bytes");
        }
    }
    pub fn enter(&mut self) -> (u32, bool) {
        let saved = (self.start, self.active);
        self.start = self.links.len();
        self.active = false;
        saved
    }
    pub fn leave(&mut self, saved: (u32, bool)) {
        self.end();
        self.start = saved.0;
        self.active = saved.1;
    }
    pub fn mark(&self, out: &mut List<Operation>) {
        for index in self.start..self.links.len() {
            let index = *self.links.get(index).expect("link exists");
            match &mut out.get_mut(index).expect("emitted link").op {
                Op::Key { missing, .. } | Op::Item { missing } => *missing = Missing::Undefined,
                Op::Name { .. }
                | Op::String(_)
                | Op::Integer(_)
                | Op::Boolean(_)
                | Op::List { .. }
                | Op::Filter(_)
                | Op::Not
                | Op::Equal
                | Op::NotEqual
                | Op::Less
                | Op::LessOrEqual
                | Op::Greater
                | Op::GreaterOrEqual
                | Op::In
                | Op::NotIn
                | Op::Concat
                | Op::Add
                | Op::Subtract
                | Op::Defined
                | Op::NotDefined
                | Op::JumpIfFalseKeeping { .. }
                | Op::JumpIfTrueKeeping { .. }
                | Op::JumpIfFalse { .. }
                | Op::Jump { .. } => unreachable!("only lookup links recorded"),
            }
        }
    }
}
