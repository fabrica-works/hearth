//! Resolution records returned through the store (render.md, 5.7).
use crate::{render, store};
use skein_lib::Token;
pub(crate) fn answer(request: Token, answer: store::vocabulary::Answer) -> render::vocabulary::Event {
    use store::vocabulary::Answer;
    match answer {
        Answer::Found { resolution } => render::vocabulary::Event::Found { request, digest: found(resolution) },
        Answer::Absent => render::vocabulary::Event::Absent { request },
        Answer::Declined { .. } => render::vocabulary::Event::Failed {
            request,
            class: render::vocabulary::Class::Operational,
            at: None,
            failure: render::vocabulary::Failure::Store,
        },
        Answer::Waiting
        | Answer::Strays { .. }
        | Answer::Cached { .. }
        | Answer::Refused { .. }
        | Answer::Ready
        | Answer::Full { .. }
        | Answer::Unusable { .. }
        | Answer::Unopened { .. } => unreachable!("find answer"),
    }
}
#[must_use]
pub fn found(resolution: store::vocabulary::Resolution) -> render::vocabulary::Digest {
    resolution.digest
}
