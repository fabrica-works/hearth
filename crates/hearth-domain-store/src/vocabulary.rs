//! Typed store requests and terminals (store.md, 6.6, 8.2).
use skein_lib::Wall;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Event {
    Open,
    Locked,
    Busy,
    Loaded { record: Option<StoreRecord> },
    Done,
    Failed,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Request {
    Lock,
    LoadRecord,
    WriteRecord { record: StoreRecord },
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Answer {
    Ready,
    Unopened { why: Why },
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Output {
    Request(Request),
    Answer(Answer),
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Why {
    Lock,
    Record,
    Owner { recorded: u32, builder: u32 },
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct StoreRecord {
    pub layout: u64,
    pub made: Wall,
    pub range: Option<Range>,
    pub uid: u32,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Range {
    pub base: u32,
    pub count: u32,
}
