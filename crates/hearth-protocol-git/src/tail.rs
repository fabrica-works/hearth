//! Bounded standard-error lines (domain/render.md, 2.2).
use crate::limits::Limits;
use alloc::boxed::Box;
use skein_lib::{List, Queue};
#[derive(Debug)]
pub struct Tail {
    lines: Queue<Box<[u8]>>,
    line: List<u8>,
    max: u32,
    pending: bool,
}
impl Tail {
    #[must_use]
    pub fn new(limits: &Limits) -> Self {
        Self {
            lines: Queue::with_capacity(limits.tail_lines),
            line: List::with_capacity(limits.tail_line),
            max: limits.tail_line,
            pending: false,
        }
    }
    pub fn admit(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            if byte == b'\n' {
                let line = core::mem::replace(&mut self.line, List::with_capacity(self.max)).into_boxed();
                if self.lines.capacity() > 0 {
                    if self.lines.room() == 0 {
                        self.lines.pop();
                    }
                    self.lines.push(line);
                }
                self.pending = false;
            } else {
                self.pending = true;
                if self.line.room() > 0 {
                    self.line.push(byte).expect("line room");
                }
            }
        }
    }
    #[must_use]
    pub fn lines(&self) -> Box<[Box<[u8]>]> {
        let mut lines = List::with_capacity(self.lines.capacity());
        for (index, line) in (&self.lines).into_iter().enumerate() {
            if !self.pending || self.lines.len() < self.lines.capacity() || index > 0 {
                lines.push(line.clone()).expect("tail capacity");
            }
        }
        if self.pending && lines.room() > 0 {
            lines.push(skein_lib::bytes::copy_of(self.line.as_slice())).expect("tail capacity");
        }
        lines.into_boxed()
    }
}
