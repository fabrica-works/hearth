use crate::render::errors::{Ids, Owns, Why};
use skein_lib::bytes::copy_of;
#[must_use]
pub fn ids_errors() -> [Ids; 7] {
    [
        (Owns::Data, Why::NoUser),
        (Owns::Secret, Why::NoGroup),
        (Owns::Identity, Why::NoUid),
        (Owns::Secret, Why::NoGid),
        (Owns::User, Why::OneId),
        (Owns::Download, Why::NotPackageUser),
        (Owns::Download, Why::NotPackageGroup),
    ]
    .map(|(of, why)| Ids { owner: copy_of(b"mail"), of, entry: copy_of(b"key"), why })
}
