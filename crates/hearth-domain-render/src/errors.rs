//! Decode errors retain their revision places (render.md, 7.1).
use crate::revision::{self, Oid, Position};
pub use crate::vocabulary::Class;
use alloc::boxed::Box;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Error {
    pub side: Option<crate::diff::Side>,
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
    Store(StoreError),
    Evaluation(crate::evaluate::FaultKind),
    Schema(Schema),
    Decode(Decode),
    Repository(Repository),
    Tool(Tool),
    Composition(Composition),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Decode {
    Syntax(Syntax),
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
    Template,
    Document,
    Decoded,
    Static,
    Entries,
    Path,
    Paths,
    Tasks,
    Targets,
}

pub(crate) fn refused(path: &[u8], position: Option<Position>, fault: Decode, also: Option<Position>) -> Error {
    let task = match revision::class(path) {
        revision::Class::Declaration { task }
        | revision::Class::Tree { task, .. }
        | revision::Class::Template { task, .. }
        | revision::Class::Script { task, .. }
        | revision::Class::Readme { task }
        | revision::Class::Refused { task: Some(task), .. } => Some(skein_lib::bytes::copy_of(task)),
        revision::Class::Fleet
        | revision::Class::Environment { .. }
        | revision::Class::Host { .. }
        | revision::Class::Refused { task: None, .. }
        | revision::Class::Other => None,
    };
    Error {
        side: None,
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
    side: Option<crate::diff::Side>,
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
            side: None,
        }
    }
    /// Counts every error's class, including errors beyond the bound.
    pub(crate) fn side(&mut self, side: crate::diff::Side) {
        self.side = Some(side);
    }
    pub fn push(&mut self, mut error: Error) {
        use crate::vocabulary::Outcome;
        error.side = self.side;
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

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Schema {
    Task { task: Box<[u8]> },
    Unread { task: Box<[u8]>, setting: Box<[u8]>, declared: Box<[Box<[u8]>]> },
    Wrong { expected: TypeName },
    Enum { value: Box<[u8]>, listed: Box<[Box<[u8]>]> },
    Below { min: i64 },
    Above { max: i64 },
    Length { max_length: i64 },
    Null,
    UndeclaredField { field: Box<[u8]>, declared: Box<[Box<[u8]>]> },
    MissingField { field: Box<[u8]> },
    Undeclared { setting: Box<[u8]>, declared: Box<[Box<[u8]>]> },
    Missing { setting: Box<[u8]> },
    Absent { task: Box<[u8]> },
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum TypeName {
    Bool,
    Int,
    String,
    Enum,
    List,
    Map,
    Object,
    Path,
    Mode,
    Duration,
    Secret,
    Target,
    File,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Repository {
    Stray,
    Environment { env: Box<[u8]> },
    Name,
    Declaration,
    Static,
    Destination,
    Submodule,
    Link,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Tool {
    pub command: Command,
    pub ended: Ended,
    pub tail: Box<[Box<[u8]>]>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Command {
    RevParse,
    LsTree,
    CatFile,
    Diff,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Ended {
    Start,
    Exit(u8),
    Signal(u32),
    Output,
    Missing { oid: Oid },
    Stream,
}

pub(crate) fn repository(path: &[u8], task: Option<&[u8]>, why: Repository) -> Error {
    Error {
        side: None,
        kind: Kind::Repository(why),
        class: Class::Configuration,
        at: Some(Place { path: skein_lib::bytes::copy_of(path), position: None }),
        also: None,
        target: None,
        task: copy_task(task),
    }
}

fn copy_task(task: Option<&[u8]>) -> Option<Box<[u8]>> {
    Some(skein_lib::bytes::copy_of(task?))
}

/// A store failure in an answer (render.md, 7.2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct StoreError {
    pub store: Box<[u8]>,
    pub fault: StoreFault,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum StoreFault {
    Full { limit: StoreLimit, most: u32 },
    Failed { step: Step },
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum StoreLimit {
    Resolutions,
    CacheFiles,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Step {
    List,
    Remove,
    Load,
    Empty,
    Keep,
}

pub mod syntax;
pub use syntax::Syntax;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Composition {
    Duplicate,
    Long,
    Placement,
    Members,
    Unplaced,
    On { on: Box<[u8]>, host: Host },
    Null,
    Task,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Host {
    Unknown,
    Elsewhere { environment: Box<[u8]> },
    Container,
}
