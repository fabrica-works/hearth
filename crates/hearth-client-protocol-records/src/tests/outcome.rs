use crate::outcome::{self, Outcome};
const CODES: [(Outcome, u8); 5] =
    [(Outcome::Done, 0), (Outcome::Negative, 1), (Outcome::Failed, 2), (Outcome::Usage, 64), (Outcome::TryAgain, 75)];
#[test]
fn each_outcome_exits_with_its_client_readme_code() {
    for (outcome, expected) in CODES {
        assert_eq!(outcome::code(outcome), expected, "client/README.md, 3.3's table");
    }
}
