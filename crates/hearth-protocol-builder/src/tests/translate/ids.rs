use crate::{records::errors as source, render::errors as target};
fn back(value: source::Ids) -> target::Ids {
    let source::Ids { owner, of, entry, why } = value;
    let of = match of {
        source::Owns::Data => target::Owns::Data,
        source::Owns::Secret => target::Owns::Secret,
        source::Owns::Identity => target::Owns::Identity,
        source::Owns::Download => target::Owns::Download,
        source::Owns::User => target::Owns::User,
    };
    let why = match why {
        source::Why::NoUser => target::Why::NoUser,
        source::Why::NoGroup => target::Why::NoGroup,
        source::Why::NoUid => target::Why::NoUid,
        source::Why::NoGid => target::Why::NoGid,
        source::Why::OneId => target::Why::OneId,
        source::Why::NotPackageUser => target::Why::NotPackageUser,
        source::Why::NotPackageGroup => target::Why::NotPackageGroup,
    };
    target::Ids { owner, of, entry, why }
}
#[test]
fn an_ids_error_translates_and_back() {
    for (row, input) in crate::tests::fixtures::ids_errors().into_iter().enumerate() {
        let mut error = crate::tests::fixtures::decode_error();
        error.kind = target::Kind::Ids(input.clone());
        let source::Kind::Ids(ids) = crate::translate::errors::error(error).kind else { panic!("ids kind") };
        assert_eq!(back(ids), input, "row {row}: every ids field");
    }
}
