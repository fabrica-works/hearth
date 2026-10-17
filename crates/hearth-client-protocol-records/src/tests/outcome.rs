use crate::outcome::{self, Outcome};
const CODES: [(Outcome, u8); 5] =
    [(Outcome::Done, 0), (Outcome::Negative, 1), (Outcome::Failed, 2), (Outcome::Usage, 64), (Outcome::TryAgain, 75)];
#[test]
fn each_outcome_exits_with_its_client_readme_code() {
    for (outcome, expected) in CODES {
        assert_eq!(outcome::code(outcome), expected, "client/README.md, 3.3's table");
    }
}

const CLASSES: [(crate::errors::Class, Outcome); 2] =
    [(crate::errors::Class::Configuration, Outcome::Failed), (crate::errors::Class::Operational, Outcome::TryAgain)];
#[test]
fn an_errors_class_gives_failed_or_try_again() {
    for (class, expected) in CLASSES {
        assert_eq!(outcome::of(class), expected, "{class:?}");
    }
}
