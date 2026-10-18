use super::fixtures;
use crate::{errors::Schema, kinds::cmdline, parts::boot, revision::Runs};
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
const MALFORMED: &[(&[u8], &(Schema, crate::revision::Position))] = &[
    (b"a b", &fixtures::BAD_CMDLINE_PARAMETER),
    (b"\"", &fixtures::BAD_CMDLINE_PARAMETER),
    (b"'", &fixtures::BAD_CMDLINE_PARAMETER),
    (b"\\", &fixtures::BAD_CMDLINE_PARAMETER),
    (b"$", &fixtures::BAD_CMDLINE_PARAMETER),
    (b";", &fixtures::BAD_CMDLINE_PARAMETER),
    (b"&", &fixtures::BAD_CMDLINE_PARAMETER),
    (b"|", &fixtures::BAD_CMDLINE_PARAMETER),
    (b"<", &fixtures::BAD_CMDLINE_PARAMETER),
    (b">", &fixtures::BAD_CMDLINE_PARAMETER),
    (b"{", &fixtures::BAD_CMDLINE_PARAMETER),
    (b"}", &fixtures::BAD_CMDLINE_PARAMETER),
    (b"#", &fixtures::BAD_CMDLINE_PARAMETER),
    (b"\n", &fixtures::BAD_CMDLINE_PARAMETER),
    (b"\x00", &fixtures::BAD_CMDLINE_PARAMETER),
    (b"\x7f", &fixtures::BAD_CMDLINE_PARAMETER),
];
#[test]
fn a_param_with_a_space_a_quote_a_shell_byte_or_a_control_byte_is_malformed() {
    for (row, (param, expected)) in MALFORMED.iter().enumerate() {
        assert_eq!(
            &cmdline::entry(&fixtures::cmdline(b"quiet", Some(param), None)).expect_err("bad parameter"),
            *expected,
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
        fixtures::BAD_CMDLINE_PARAMETER
    );
}
#[test]
fn identical_entries_of_two_tasks_make_one_parameter() {
    let expansion = fixtures::boot_expansion(&[(b"a", b"quiet", b"quiet", None), (b"b", b"quiet", b"quiet", None)]);
    let rows = fixtures::collisions::agreeing(&expansion).expect("equal parameters");
    assert_eq!(fixtures::parameter_values(&rows.parameters).as_ref(), &[b"quiet".as_slice()]);
}
#[test]
fn a_key_two_tasks_give_differently_keeps_the_first_tasks() {
    let expansion = fixtures::boot_expansion(&[(b"a", b"quiet", b"quiet", None), (b"b", b"quiet", b"debug", None)]);
    let errors = fixtures::collisions::agreeing(&expansion).expect_err("differing parameters");
    assert_eq!(
        fixtures::collisions::kinds(&errors).as_ref(),
        &[(
            crate::errors::Kind::Collision(crate::errors::Collision::Differs {
                subject: crate::errors::Subject::Parameter,
                name: copy_of(b"quiet"),
                first: copy_of(b"a"),
                second: copy_of(b"b"),
            }),
            crate::errors::Class::Configuration
        )]
    );
}
const MUTABLE: &[(bool, bool)] = &[(false, false), (true, true)];
#[test]
fn mutable_comes_from_the_composed_placement() {
    for (row, &(mutable, expected)) in MUTABLE.iter().enumerate() {
        let mut placement = fixtures::containers::placement(None);
        placement.mutable = mutable;
        assert_eq!(
            boot::build(&fixtures::boot_expansion(&[]), &placement).expect("placement").mutable,
            expected,
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
