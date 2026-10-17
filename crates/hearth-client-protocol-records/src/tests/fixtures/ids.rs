use crate::errors::{Error, Ids, Kind, Owns, Why};
use skein_lib::bytes::copy_of;
pub fn ids_error(of: Owns, why: Why) -> Error {
    let mut error = super::modules_guard();
    error.also = None;
    error.kind = Kind::Ids(Ids {
        owner: copy_of(b"mail"),
        of,
        entry: copy_of(if of == Owns::User { b"mail" } else { b"mail/tls.key" }),
        why,
    });
    error
}
