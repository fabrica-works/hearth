use crate::vocabulary::Outcome;
const OUTCOMES: [Outcome; 5] = [Outcome::Usage, Outcome::Failed, Outcome::TryAgain, Outcome::Negative, Outcome::Done];
const WORST: [[Outcome; 5]; 5] = [
    [Outcome::Usage, Outcome::Usage, Outcome::Usage, Outcome::Usage, Outcome::Usage],
    [Outcome::Usage, Outcome::Failed, Outcome::Failed, Outcome::Failed, Outcome::Failed],
    [Outcome::Usage, Outcome::Failed, Outcome::TryAgain, Outcome::TryAgain, Outcome::TryAgain],
    [Outcome::Usage, Outcome::Failed, Outcome::TryAgain, Outcome::Negative, Outcome::Negative],
    [Outcome::Usage, Outcome::Failed, Outcome::TryAgain, Outcome::Negative, Outcome::Done],
];
#[test]
fn the_worst_outcome_wins_over_every_pair() {
    for (row, left) in OUTCOMES.iter().enumerate() {
        for (column, right) in OUTCOMES.iter().enumerate() {
            assert_eq!(left.worst(*right), WORST[row][column], "{left:?}, {right:?}");
        }
    }
}
