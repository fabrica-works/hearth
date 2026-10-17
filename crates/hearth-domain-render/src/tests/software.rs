use super::fixtures;
use crate::{
    errors::{Format, Kind, Schema},
    parts::software::{self, How},
    revision::declaration::{Map, Name},
};
use fixtures::kinds::text;
const DEFAULTS: &[(&[u8], How)] = &[(b"file", How::File { mode: 0o755 }), (b"tar", fixtures::TAR_WITH_NO_STRIP)];
#[test]
fn a_file_defaults_to_mode_0755_and_a_tar_to_strip_0() {
    for (row, (unpack, how)) in DEFAULTS.iter().enumerate() {
        let software = fixtures::software(&[text(Name::Unpack, unpack)]);
        assert_eq!(&fixtures::only(&software.downloads).how, how, "row {row}");
    }
}
#[test]
fn the_url_stays_outside_the_digest() {
    assert_eq!(
        software::digest(&fixtures::mirrored(b"https://a.example/rg")),
        software::digest(&fixtures::mirrored(b"https://b.example/rg")),
        "render.md, 4.2: a mirror rebuilds nothing"
    );
}
#[test]
fn the_url_is_retained_in_the_fetch_hint() {
    let software = fixtures::mirrored(b"https://b.example/rg");
    assert_eq!(fixtures::only(&software.fetch).url.as_ref(), b"https://b.example/rg");
}
#[test]
fn a_file_mode_past_0777_is_above() {
    assert_eq!(
        fixtures::software_fault(&[fixtures::download_mode(fixtures::MAX_DOWNLOAD_MODE + 1)]).kind,
        Kind::Schema(Schema::Above { max: fixtures::MAX_DOWNLOAD_MODE })
    );
}
const OUTSIDE: &[&[u8]] = &[b"/srv/tool", b"/optional", b"/usr/localx", b"/opt", b"/usr/local"];
#[test]
fn a_path_outside_opt_and_usr_local_is_beneath() {
    for (row, path) in OUTSIDE.iter().enumerate() {
        assert_eq!(
            fixtures::software_fault(&[text(Name::Path, path)]).kind,
            Kind::Schema(Schema::Beneath {
                allowed: Box::new([b"/opt".as_slice().into(), b"/usr/local".as_slice().into()])
            }),
            "row {row}"
        );
    }
}
#[test]
fn a_strip_past_255_is_above() {
    assert_eq!(
        fixtures::software_fault(&[text(Name::Unpack, b"tar"), fixtures::strip(fixtures::MAX_STRIP + 1)]).kind,
        Kind::Schema(Schema::Above { max: fixtures::MAX_STRIP })
    );
}
#[test]
fn a_negative_strip_is_below() {
    assert_eq!(
        fixtures::software_fault(&[text(Name::Unpack, b"tar"), fixtures::strip(fixtures::MIN_STRIP - 1)]).kind,
        Kind::Schema(Schema::Below { min: fixtures::MIN_STRIP })
    );
}
struct Forbidden {
    unpack: &'static [u8],
    field: Name,
    name: &'static [u8],
    allowed: &'static [u8],
}
const FORBIDDEN: &[Forbidden] = &[
    Forbidden { unpack: b"file", field: Name::Strip, name: b"strip", allowed: b"mode" },
    Forbidden { unpack: b"tar", field: Name::Mode, name: b"mode", allowed: b"strip" },
];
#[test]
fn strip_on_a_file_and_mode_on_a_tar_are_undeclared_fields() {
    for (row, Forbidden { unpack, field, name, allowed }) in FORBIDDEN.iter().enumerate() {
        assert_eq!(
            fixtures::software_fault(&[text(Name::Unpack, unpack), text(*field, b"0")]).kind,
            Kind::Schema(Schema::UndeclaredField {
                field: (*name).into(),
                declared: Box::new([
                    b"url".as_slice().into(),
                    b"sha256".as_slice().into(),
                    b"unpack".as_slice().into(),
                    b"path".as_slice().into(),
                    b"owner".as_slice().into(),
                    b"group".as_slice().into(),
                    (*allowed).into()
                ])
            }),
            "row {row}"
        );
    }
}
const BAD_SHA: &[&[u8]] = &[
    b"abc",
    b"BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD",
    b"ga7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
];
#[test]
fn a_sha256_that_is_not_64_lower_hex_is_malformed() {
    for (row, sha) in BAD_SHA.iter().enumerate() {
        assert_eq!(
            fixtures::software_fault(&[text(Name::Sha256, sha)]).kind,
            Kind::Schema(Schema::Malformed { format: Format::Sha256 }),
            "row {row}"
        );
    }
}
#[test]
fn two_paths_of_one_sha256_keep_the_first_url() {
    let expansion = fixtures::concrete_expansion(
        Map::Software,
        &[
            (
                b"a",
                &[
                    (Name::Url, b"https://z.example/tool"),
                    (Name::Sha256, fixtures::ABC_SHA256_HEX),
                    (Name::Unpack, b"file"),
                    (Name::Path, b"/opt/z"),
                ],
            ),
            (
                b"z",
                &[
                    (Name::Url, b"https://a.example/tool"),
                    (Name::Sha256, fixtures::ABC_SHA256_HEX),
                    (Name::Unpack, b"file"),
                    (Name::Path, b"/opt/a"),
                ],
            ),
        ],
    );
    let software = software::build(&expansion, Box::new([])).expect("downloads");
    assert_eq!(fixtures::only(&software.fetch).url.as_ref(), b"https://a.example/tool");
}
#[test]
fn no_entry_makes_an_empty_part() {
    let software =
        software::build(&fixtures::concrete_expansion(Map::Software, &[]), Box::new([])).expect("empty part");
    assert_eq!(software, software::Software { downloads: Box::new([]), scripts: Box::new([]), fetch: Box::new([]) });
}
#[test]
fn a_software_script_without_a_download_makes_a_software_part() {
    let software = fixtures::rendered_software_scripts(&[(b"motd", fixtures::DEFAULT_SCRIPT_ORDER)], false);
    assert_eq!(
        software.scripts.as_ref(),
        &[software::Script {
            task: b"motd".as_slice().into(),
            order: fixtures::DEFAULT_SCRIPT_ORDER,
            content_digest: fixtures::CACHED_SCRIPT_DIGEST
        }]
    );
}
#[test]
fn a_software_script_is_digested_in_the_software_part() {
    let software = fixtures::rendered_software_scripts(&[(b"motd", fixtures::DEFAULT_SCRIPT_ORDER)], true);
    assert_eq!(software::digest(&software), fixtures::SOFTWARE_SCRIPT_DIGEST);
}

const REQUIRED: &[(Name, &[u8])] =
    &[(Name::Url, b"url"), (Name::Sha256, b"sha256"), (Name::Unpack, b"unpack"), (Name::Path, b"path")];
#[test]
fn a_download_requires_its_url_sha256_unpack_and_path() {
    for (row, (field, name)) in REQUIRED.iter().enumerate() {
        assert_eq!(
            fixtures::missing_download_field(*field).kind,
            Kind::Schema(Schema::MissingField { field: (*name).into() }),
            "row {row}"
        );
    }
}
const ACCEPTED_STRIPS: &[(i64, How)] =
    &[(fixtures::MIN_STRIP, fixtures::TAR_WITH_NO_STRIP), (fixtures::MAX_STRIP, fixtures::TAR_WITH_MAX_STRIP)];
#[test]
fn a_tar_accepts_both_strip_limits() {
    for (row, (strip, how)) in ACCEPTED_STRIPS.iter().enumerate() {
        let software = fixtures::software(&[text(Name::Unpack, b"tar"), fixtures::strip(*strip)]);
        assert_eq!(&fixtures::only(&software.downloads).how, how, "row {row}");
    }
}

#[test]
fn software_scripts_sort_by_order_then_task() {
    let software = fixtures::rendered_software_scripts(fixtures::ORDERED_SCRIPTS, false);
    assert_eq!(
        software.scripts.as_ref(),
        &[
            software::Script {
                task: b"a".as_slice().into(),
                order: 100,
                content_digest: fixtures::CACHED_SCRIPT_DIGEST
            },
            software::Script {
                task: b"b".as_slice().into(),
                order: 100,
                content_digest: fixtures::CACHED_SCRIPT_DIGEST
            },
            software::Script {
                task: b"z".as_slice().into(),
                order: 200,
                content_digest: fixtures::CACHED_SCRIPT_DIGEST
            },
        ]
    );
}
