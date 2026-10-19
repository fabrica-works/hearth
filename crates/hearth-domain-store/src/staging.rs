//! A layer's life past the index: staging, sealing and discarding (domain/store.md, 4.2).
use crate::vocabulary::{Answer, Output, Step};
use skein_lib::Queue;
/// Transitional: h04b replaces this with the writable snapshot of the parent (step 1), and its
/// cleanup of a failed stage.
pub(super) fn stage(out: &mut Queue<Output>) {
    out.push(Output::Answer(Answer::Unusable { step: Step::Stage }));
}
/// Transitional: h04b replaces this with the walk, freeze, record and publish (steps 3 to 6).
pub(super) fn seal(out: &mut Queue<Output>) {
    out.push(Output::Answer(Answer::Unusable { step: Step::Seal }));
}
/// Transitional: h04c replaces this with the drop of the staging subvolume.
pub(super) fn discard(out: &mut Queue<Output>) {
    out.push(Output::Answer(Answer::Unusable { step: Step::Discard }));
}
