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
