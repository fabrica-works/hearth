use super::fixtures;
use crate::{
    errors::{Format, Schema},
    kinds::cmdline,
    parts::boot,
    revision::Runs,
};
use skein_lib::bytes::copy_of;
#[test]
fn parameters_sort_by_order_then_key_then_task() {
    let boot = fixtures::built_boot(&[
        (b"c", b"console", b"console=ttyS0,115200n8", Some(900)),
        (b"b", b"audit", b"audit=0", None),
        (b"a", b"quiet", b"quiet", Some(100)),
        (b"d", b"zero", b"zero", None),
    ]);
    assert_eq!(
        boot.params.as_ref(),
        [copy_of(b"quiet"), copy_of(b"audit=0"), copy_of(b"zero"), copy_of(b"console=ttyS0,115200n8")]
    );
}
const ORDERS: &[(i64, Result<u16, Schema>)] = &[
    (fixtures::MIN_CMDLINE_ORDER - 1, Err(Schema::Below { min: fixtures::MIN_CMDLINE_ORDER })),
    (fixtures::MIN_CMDLINE_ORDER, Ok(0)),
    (fixtures::MAX_CMDLINE_ORDER, Ok(999)),
    (fixtures::MAX_CMDLINE_ORDER + 1, Err(Schema::Above { max: fixtures::MAX_CMDLINE_ORDER })),
];
#[test]
fn order_defaults_to_500_and_ranges_over_0_to_999() {
    for (row, (order, expected)) in ORDERS.iter().enumerate() {
        let entry = fixtures::cmdline(b"quiet", Some(b"quiet"), Some(*order));
        let order = match cmdline::entry(&entry) {
            Ok(param) => Ok(param.order),
            Err((fault, _)) => Err(fault),
        };
        assert_eq!(order, *expected, "row {row}");
    }
}
#[test]
fn a_parameter_without_order_defaults_to_500() {
    assert_eq!(
        cmdline::entry(&fixtures::cmdline(b"quiet", Some(b"quiet"), None)).expect("valid parameter").order,
        fixtures::DEFAULT_CMDLINE_ORDER
    );
}
const MALFORMED: &[&[u8]] =
    &[b"a b", b"\"", b"'", b"\\", b"$", b";", b"&", b"|", b"<", b">", b"{", b"}", b"#", b"\n", b"\x00", b"\x7f"];
#[test]
fn a_param_with_a_space_a_quote_a_shell_byte_or_a_control_byte_is_malformed() {
    for (row, param) in MALFORMED.iter().enumerate() {
        assert_eq!(
            cmdline::entry(&fixtures::cmdline(b"quiet", Some(param), None)).expect_err("bad parameter"),
            (Schema::Malformed { format: Format::Param }, fixtures::FIELD_AT),
            "row {row}"
        );
    }
}
#[test]
fn a_containers_boot_part_has_no_params() {
    let mut placement = fixtures::containers::placement(None);
    placement.kind = Runs::Container;
    assert_eq!(
        boot::build(&fixtures::boot_expansion(&[(b"a", b"quiet", b"quiet", None)]), &placement)
            .expect("container")
            .params
            .as_ref(),
        []
    );
}
#[test]
fn a_missing_param_is_a_missing_field() {
    assert_eq!(
        cmdline::entry(&fixtures::cmdline(b"quiet", None, None)).expect_err("missing parameter"),
        (Schema::MissingField { field: copy_of(b"param") }, fixtures::composition::at(1, 14))
    );
}
#[test]
fn an_empty_param_is_malformed() {
    assert_eq!(
        cmdline::entry(&fixtures::cmdline(b"quiet", Some(b""), None)).expect_err("empty parameter"),
        (Schema::Malformed { format: Format::Param }, fixtures::FIELD_AT)
    );
}
#[test]
fn identical_entries_of_two_tasks_make_one_parameter() {
    let boot = fixtures::built_boot(&[(b"a", b"quiet", b"quiet", None), (b"b", b"quiet", b"quiet", None)]);
    assert_eq!(boot.params.as_ref(), [copy_of(b"quiet")]);
}
#[test]
fn a_key_two_tasks_give_differently_keeps_the_first_tasks() {
    let boot = fixtures::built_boot(&[(b"a", b"quiet", b"quiet", None), (b"b", b"quiet", b"debug", None)]);
    assert_eq!(boot.params.as_ref(), [copy_of(b"quiet")]);
}
const MUTABLE: &[bool] = &[false, true];
#[test]
fn mutable_comes_from_the_composed_placement() {
    for (row, &mutable) in MUTABLE.iter().enumerate() {
        let mut placement = fixtures::containers::placement(None);
        placement.mutable = mutable;
        assert_eq!(
            boot::build(&fixtures::boot_expansion(&[]), &placement).expect("placement").mutable,
            mutable,
            "row {row}"
        );
    }
}
#[test]
fn the_boot_digest_is_h_of_part_boot_over_its_record() {
    assert_eq!(
        boot::digest(&fixtures::built_boot(&[])),
        fixtures::digest("bc6cab4172522d9c426b60fc416b1adb10b9897d81dcf644be48b00e9902a077")
    );
}
