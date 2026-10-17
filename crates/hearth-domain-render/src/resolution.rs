//! A resolution run's decoded install lines and classified cause (render.md, 5.5).
use crate::vocabulary::Class;
use alloc::boxed::Box;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Inst {
    pub name: Box<[u8]>,
    pub version: Box<[u8]>,
    pub architecture: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Index {
    pub path: Box<[u8]>,
    pub size: u64,
    pub sha256: hearth_lib::encode::Digest,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Line {
    pub indices: Box<[Index]>,
    pub uri: Box<[u8]>,
    pub suite: Box<[u8]>,
    pub components: Box<[Box<[u8]>]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Tools {
    pub apt: Box<[u8]>,
    pub mmdebstrap: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Costs {
    pub fetched: u64,
    pub took: skein_lib::Duration,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Cause {
    Fetch,
    Resolve,
    Space,
    Absent { uri: Box<[u8]>, suite: Box<[u8]> },
    Unknown(Box<[u8]>),
    Unsatisfiable(Option<Box<[u8]>>),
    Other,
}
impl Cause {
    #[must_use]
    pub const fn class(&self) -> Class {
        match self {
            Self::Fetch | Self::Resolve | Self::Space => Class::Operational,
            Self::Absent { .. } | Self::Unknown(_) | Self::Unsatisfiable(_) | Self::Other => Class::Configuration,
        }
    }
}
