//! Terminal predicates shared by git modes.
use crate::{Up, git_machine::listing};
use alloc::boxed::Box;
pub(super) fn listing_failed(answer: Option<&Result<Box<[listing::Entry]>, listing::Error>>) -> bool {
    match answer {
        Some(Err(_)) => true,
        Some(Ok(_)) | None => false,
    }
}
pub(super) fn stream_ended(up: &Up) -> bool {
    match up {
        Up::End | Up::Failed(_) => true,
        Up::Bytes(_) | Up::Room => false,
    }
}
