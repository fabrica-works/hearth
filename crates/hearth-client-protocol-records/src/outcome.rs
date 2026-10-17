//! Terminal outcomes (client/README.md, 3.3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Outcome {
    Done,
    Negative,
    Failed,
    Usage,
    TryAgain,
}
/// The command's process exit code.
#[must_use]
pub fn code(outcome: Outcome) -> u8 {
    match outcome {
        Outcome::Done => 0,
        Outcome::Negative => 1,
        Outcome::Failed => 2,
        Outcome::Usage => 64,
        Outcome::TryAgain => 75,
    }
}

/// An error class's terminal outcome (client/README.md, 3.3).
#[must_use]
pub fn of(class: crate::errors::Class) -> Outcome {
    match class {
        crate::errors::Class::Configuration => Outcome::Failed,
        crate::errors::Class::Operational => Outcome::TryAgain,
    }
}
