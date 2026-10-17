use crate::{
    records::diff as records,
    render::{diff, parts::meta},
    translate::changes,
};
pub(super) fn back(change: records::MetaChange) -> meta::Change {
    meta::Change {
        base: pair_back(change.base),
        member: pair_back(change.member),
        environment: pair_back(change.environment),
        placement: placement_pair_back(change.placement),
    }
}
fn pair_back<T>(pair: Option<records::Pair<T>>) -> Option<meta::Pair<T>> {
    let pair = pair?;
    Some(meta::Pair { before: pair.before, after: pair.after })
}
fn placement_back(value: crate::records::parts::meta::Placement) -> meta::Placement {
    meta::Placement {
        kind: super::client_kind_back(value.kind),
        firmware: super::optional_client_firmware_back(value.firmware),
        hardware: value.hardware,
        on: value.on,
    }
}
#[test]
fn each_meta_change_translates_and_back() {
    for (row, input) in crate::tests::fixtures::meta_changes().into_iter().enumerate() {
        let records::PartChange::Meta(change) = changes::part_change(diff::PartChange::Meta(input.clone())) else {
            panic!("row {row}: meta change");
        };
        assert_eq!(back(change), input, "row {row}: meta change");
    }
}

fn placement_pair_back(
    pair: Option<records::Pair<crate::records::parts::meta::Placement>>,
) -> Option<meta::Pair<meta::Placement>> {
    let pair = pair?;
    Some(meta::Pair { before: placement_back(pair.before), after: placement_back(pair.after) })
}

const ERROR_SIDES: &[(Option<diff::Side>, Option<records::Side>)] = &[
    (None, None),
    (Some(diff::Side::From), Some(records::Side::From)),
    (Some(diff::Side::To), Some(records::Side::To)),
];
#[test]
fn an_errors_side_crosses_to_records() {
    for (row, &(side, expected)) in ERROR_SIDES.iter().enumerate() {
        let mut error = crate::tests::fixtures::decode_error();
        error.side = side;
        assert_eq!(crate::translate::errors::error(error).side, expected, "row {row}: error side");
    }
}
