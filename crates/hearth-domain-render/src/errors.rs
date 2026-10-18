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
    Reserved(Reserved),
    Exported(Exported),
    Ids(Ids),

    Resolution(Resolution),
    Guard(Guard),
    Collision(Collision),
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
    Record,
    Closure,
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
mod held;

/// The key of the first error excluded from the ordered prefix (7.1).
#[derive(Debug, Hash)]
struct Order {
    unplaced: bool,
    path: Box<[u8]>,
    position: (u32, u32),
    target: Option<Box<[u8]>>,
}
/// The longest ordered prefix within both of render.md, 7.1's bounds.
#[derive(Debug, Hash)]
pub struct Collector {
    kept: skein_lib::List<Error>,
    cut: Option<Order>,
    bytes: u64,
    left_out: u32,
    outcome: crate::vocabulary::Outcome,
    side: Option<crate::diff::Side>,
}
impl Collector {
    #[must_use]
    pub fn new(limits: &crate::limits::Limits) -> Self {
        Self {
            kept: skein_lib::List::with_capacity(limits.errors),
            cut: None,
            bytes: limits.error_bytes,
            left_out: 0,
            outcome: crate::vocabulary::Outcome::Done,
            side: None,
        }
    }
    pub(crate) fn side(&mut self, side: crate::diff::Side) {
        self.side = Some(side);
    }
    /// Counts every class, including errors beyond either prefix bound.
    pub fn push(&mut self, mut error: Error) {
        use crate::vocabulary::Outcome;
        error.side = self.side;
        self.outcome = self.outcome.worst(match error.class {
            Class::Configuration => Outcome::Failed,
            Class::Operational => Outcome::TryAgain,
        });
        if let Some(cut) = &self.cut
            && held::past(&error, cut)
        {
            self.left_out = self.left_out.saturating_add(1);
            return;
        }
        // Insertion is stable: equal places retain their arrival order.
        for index in 0..self.kept.len() {
            let kept = self.kept.get_mut(index).expect("kept index");
            if held::compare(&error, kept).is_lt() {
                core::mem::swap(&mut error, kept);
            }
        }
        if let Err(error) = self.kept.push(error) {
            self.cut = Some(held::order(&error));
            self.left_out = self.left_out.saturating_add(1);
        }
        let mut bytes = 0_u64;
        let mut count = 0_u32;
        for error in &self.kept {
            bytes = bytes.saturating_add(held::bytes(error));
            if bytes > self.bytes {
                self.cut = Some(held::order(error));
                break;
            }
            count = count.saturating_add(1);
        }
        self.left_out = self.left_out.saturating_add(self.kept.len().saturating_sub(count));
        self.kept.truncate(count);
    }
    pub(crate) fn extend(&mut self, errors: Box<[Error]>) {
        for error in errors {
            self.push(error);
        }
    }
    #[must_use]
    pub fn finish(self) -> (Box<[Error]>, u32) {
        (self.kept.into_boxed(), self.left_out)
    }
    /// Error records and their owned bytes, plus the excluded error's key.
    #[must_use]
    pub fn worst_case(limits: &crate::limits::Limits) -> Option<u64> {
        skein_lib::List::<Error>::worst_case(limits.errors)?.checked_add(limits.error_bytes)?.checked_add(4_096 + 63)
    }
    #[cfg(test)]
    pub(crate) fn errors(&self) -> Box<[Error]> {
        self.kept.to_boxed()
    }
    pub(crate) fn get(&self, index: u32) -> Option<&Error> {
        self.kept.get(index)
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
    Source { first: revision::declaration::Name, second: revision::declaration::Name },
    Export { fault: ExportFault },
    Unlisted { reader: Box<[u8]>, root: Root, task: Box<[u8]>, setting: Option<Box<[u8]>> },
    Dynamic { root: Root },
    Task { task: Box<[u8]> },
    Unread { task: Box<[u8]>, setting: Box<[u8]>, declared: Box<[Box<[u8]>]> },
    Wrong { expected: TypeName },
    Enum { value: Box<[u8]>, listed: Box<[Box<[u8]>]> },
    Beneath { allowed: Box<[Box<[u8]>]> },
    Below { min: i64 },
    Above { max: i64 },
    Length { max_length: i64 },
    Null,
    UndeclaredField { field: Box<[u8]>, declared: Box<[Box<[u8]>]> },
    MissingField { field: Box<[u8]> },
    Undeclared { setting: Box<[u8]>, declared: Box<[Box<[u8]>]> },
    Missing { setting: Box<[u8]> },
    Absent { task: Box<[u8]> },
    Malformed { format: Format },
    Target { name: Box<[u8]>, environment: Box<[u8]> },
    File { path: Box<[u8]> },
    Large { path: Box<[u8]>, max: u64 },
    Main,
    Security,
    Component { package: Box<[u8]>, component: Box<[u8]> },
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Format {
    DataName,
    Sha256,
    Link,
    Path,
    Mode,
    Duration,
    Secret,
    File,
    Key,
    Timestamp,
    Debian,
    Preseed,
    Module,
    Param,
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
    NoTree,
    NoTemplate,
    NoScript,
    Unused,
    LinkTarget,
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
    Mmdebstrap,
    RevParse,
    LsTree,
    CatFile,
    Diff,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Ended {
    Timeout,
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
    Write,
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
    Unplaced,
    Unknown,
    Elsewhere { environment: Box<[u8]> },
    Container,
}

/// A false guard's name and message (format.md, 5.9).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Guard {
    pub name: Box<[u8]>,
    pub message: Box<[u8]>,
}

/// A key produced twice by one task's expanded maps (format.md, 5.2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Collision {
    Key { key: Box<[u8]> },
    Given { subject: Subject, name: Box<[u8]>, first: Box<[u8]>, second: Box<[u8]> },
    Differs { subject: Subject, name: Box<[u8]>, first: Box<[u8]>, second: Box<[u8]> },
    Shared { id: Id, value: u32, first: Box<[u8]>, second: Box<[u8]> },
    Hidden { path: Box<[u8]>, data: Box<[u8]>, first: Box<[u8]>, second: Box<[u8]> },
    Overlap { subject: Subject, first: Box<[u8]>, second: Box<[u8]> },
}

/// The namespace of a collision (render.md, 3.7).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Subject {
    Tree,
    DataName,
    DataPath,
    Secret,
    Identity,
    Download,
    User,
    Group,
    Preset,
    Tmpfiles,
    Preseed,
    Parameter,
    Range,
}
/// The account id namespace (format.md, 5.6).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Id {
    Uid,
    Gid,
}

/// Reference roots restricted to literal names (format.md, 6.4).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Root {
    Debian,
    Tasks,
    Exports,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ExportFault {
    Form,
    Root { root: Root },
}

/// Another target's exports failed (render.md, 3.4).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Exported {
    pub exporter: Box<[u8]>,
    pub task: Box<[u8]>,
    pub cause: Cause,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Cause {
    Decode(Decode),
    Repository(Repository),
    Composition(Composition),
    Schema(Schema),
    Evaluation(crate::evaluate::FaultKind),
}

/// Apt's configuration verdict and the bounded closure (render.md, 5.5, 7.2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Resolution {
    TooLarge { max: u64 },
    Unknown { package: Box<[u8]> },
    Unsatisfiable { package: Option<Box<[u8]>> },
    Absent { uri: Box<[u8]>, suite: Box<[u8]> },
    Missing { package: Box<[u8]> },
    Unmatched { name: Box<[u8]>, version: Box<[u8]>, architecture: Box<[u8]> },
    Disagree { name: Box<[u8]>, version: Box<[u8]>, architecture: Box<[u8]> },
    TooMany { max: u32 },
}

/// A fixed id is missing for an entry's owner (render.md, 7.2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Ids {
    pub owner: Box<[u8]>,
    pub of: Owns,
    pub entry: Box<[u8]>,
    pub why: Why,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Owns {
    Data,
    Secret,
    Identity,
    Download,
    User,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Why {
    NoUser,
    NoGroup,
    NoUid,
    NoGid,
    OneId,
    NotPackageUser,
    NotPackageGroup,
}

mod reserved;
pub use reserved::{Reserved, Rule};
