//! Decode errors retain their revision places (render.md, 7.1).
use crate::revision::{self, Position};
pub use crate::vocabulary::Class;
use alloc::boxed::Box;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Error {
    pub kind: Kind,
    pub class: Class,
    pub at: Option<Place>,
    pub also: Option<Place>,
    pub target: Option<Box<[u8]>>,
    pub task: Option<Box<[u8]>>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Place {
    pub path: Box<[u8]>,
    pub position: Option<Position>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Kind {
    Decode(Decode),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Decode {
    Declaration(Declared),
    ListNull,
    Unexpected,
    Trailing,
    TooDeep,
    StringTooLong,
    NumberTooLong,
    Number,
    Escape,
    Surrogate,
    Utf8,
    Control,
    Truncated,
    NotObject,
    NotInteger,
    Range,
    Nul,
    Keys,
    Note,
    Name,
    Placement,
    Members,
    Member,
    Duplicate { key: Box<[u8]> },
    Settings { task: Box<[u8]> },
    Unknown { field: Box<[u8]> },
    Wrong { field: Field },
    Missing { field: Field },
    Misplaced { field: Field },
    TooLarge { limit: Limit, max: u64 },
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Field {
    Kind,
    Firmware,
    Hardware,
    On,
    Mutable,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Limit {
    Document,
    Decoded,
    Static,
}

pub(crate) fn refused(path: &[u8], position: Option<Position>, fault: Decode, also: Option<Position>) -> Error {
    let task = match revision::class(path) {
        revision::Class::Declaration { task } | revision::Class::Tree { task, .. } => {
            Some(skein_lib::bytes::copy_of(task))
        }
        revision::Class::Fleet
        | revision::Class::Environment { .. }
        | revision::Class::Host { .. }
        | revision::Class::Other => None,
    };
    Error {
        kind: Kind::Decode(fault),
        class: Class::Configuration,
        at: Some(Place { path: skein_lib::bytes::copy_of(path), position }),
        also: second(path, also),
        target: None,
        task,
    }
}
fn second(path: &[u8], position: Option<Position>) -> Option<Place> {
    Some(Place { path: skein_lib::bytes::copy_of(path), position: Some(position?) })
}
/// Placed errors precede unplaced ones; equal places retain arrival order (7.1).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Order {
    unplaced: bool,
    path: Box<[u8]>,
    position: (u32, u32),
    target: Option<Box<[u8]>>,
    arrival: u64,
}
/// The first configured number of errors, with the rest counted (render.md, 7.1).
#[derive(Debug)]
pub struct Collector {
    kept: skein_lib::Map<Order, Error>,
    left_out: u32,
    arrival: u64,
    outcome: crate::vocabulary::Outcome,
}
impl Collector {
    #[must_use]
    pub fn new(limits: &crate::limits::Limits) -> Self {
        let left_out = 0;
        Self {
            kept: skein_lib::Map::with_capacity(limits.errors),
            left_out,
            arrival: 0,
            outcome: crate::vocabulary::Outcome::Done,
        }
    }
    /// Counts every error's class, including errors beyond the bound.
    pub fn push(&mut self, error: Error) {
        use crate::vocabulary::Outcome;
        self.outcome = self.outcome.worst(match error.class {
            Class::Configuration => Outcome::Failed,
            Class::Operational => Outcome::TryAgain,
        });
        let (path, position) = match &error.at {
            Some(place) => (
                place.path.clone(),
                match place.position {
                    Some(position) => (position.line, position.column),
                    None => (0, 0),
                },
            ),
            None => (skein_lib::bytes::copy_of(b""), (0, 0)),
        };
        let order =
            Order { unplaced: error.at.is_none(), path, position, target: error.target.clone(), arrival: self.arrival };
        self.arrival = self.arrival.checked_add(1).expect("a bounded call has fewer than u64 errors");
        if let Err((order, error)) = self.kept.insert(order, error) {
            self.left_out = self.left_out.saturating_add(1);
            if let Some((last, _)) = self.kept.last()
                && order < *last
            {
                let last = last.clone();
                drop(self.kept.remove(&last));
                self.kept.insert(order, error).expect("displaced one error");
            }
        }
    }
    #[must_use]
    pub fn finish(mut self) -> (Box<[Error]>, u32) {
        let mut errors = skein_lib::List::with_capacity(self.kept.len());
        for _ in 0..self.kept.len() {
            let order = self.kept.first().expect("remaining error").0.clone();
            let error = self.kept.remove(&order).expect("first error");
            errors.push(error).expect("kept bound");
        }
        (errors.into_boxed(), self.left_out)
    }
    /// Includes owned bytes: three paths, target twice, task, and a decoded key.
    #[must_use]
    pub fn worst_case(limits: &crate::limits::Limits) -> Option<u64> {
        let bytes = u64::from(limits.document).checked_add(12_288)?.checked_add(189)?;
        skein_lib::Map::<Order, Error>::worst_case(limits.errors)?
            .checked_add(bytes.checked_mul(u64::from(limits.errors))?)
    }
    #[cfg(test)]
    pub(crate) fn errors(&self) -> Box<[Error]> {
        let mut errors = skein_lib::List::with_capacity(self.kept.len());
        for (_, error) in &self.kept {
            errors.push(error.clone()).expect("kept bound");
        }
        errors.into_boxed()
    }
    pub(crate) fn get(&self, index: u32) -> Option<&Error> {
        let (_, error) = self.kept.iter().nth(usize::try_from(index).expect("u32 fits usize"))?;
        Some(error)
    }
    pub(crate) const fn left_out(&self) -> u32 {
        self.left_out
    }
    pub(crate) const fn outcome(&self) -> crate::vocabulary::Outcome {
        self.outcome
    }
}
pub(crate) fn push(errors: &mut Collector, error: Error) {
    errors.push(error);
}

/// Why a decoded task.json is no declaration (format.md, 5.1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Declared {
    Unknown { of: Object },
    Later,
    Wrong { expected: Expected },
    Missing { key: Required },
    Name,
    Doc,
    Order,
    Optional,
    Bound,
    Above,
    Enum,
    Settings,
    Secret,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Object {
    Declaration,
    Read,
    Guard,
    Script,
    Bless,
    Entry(revision::declaration::Map),
    Range,
    Spec,
    Export,
    Type,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Expected {
    Object,
    String,
    Strings,
    Bool,
    Int,
    Field(Takes),
    True,
    Type,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Required {
    If,
    Message,
    Type,
    Doc,
    Value,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Takes {
    Text,
    Int,
    Bool,
    Strings,
    Range,
}
