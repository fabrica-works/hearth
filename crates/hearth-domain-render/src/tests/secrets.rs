use super::fixtures;
use crate::{
    errors::{Format, Kind, Schema},
    parts::secrets::{self, Secrets},
    revision::declaration::{Map, Name},
};
use skein_lib::bytes::copy_of;
#[test]
fn a_secret_defaults_to_mode_0600_owned_by_root() {
    let expansion =
        fixtures::secret_entry(Map::Secrets, b"mail/tls/mx1.key", &[(Name::Path, b"/etc/ssl/private/mx1.key")]);
    assert_eq!(
        secrets::build(&expansion).expect("valid").files.as_ref(),
        &[secrets::SecretFile {
            name: copy_of(b"mail/tls/mx1.key"),
            path: copy_of(b"/etc/ssl/private/mx1.key"),
            mode: 0o600,
            owner: copy_of(b"root"),
            group: copy_of(b"root")
        }]
    );
}
const BAD_NAMES: &[&[u8]] = &[b"Users/x", b"users/../x", b"a//b", b"."];
#[test]
fn a_secret_name_out_of_2_4s_form_is_malformed() {
    for (row, name) in BAD_NAMES.iter().enumerate() {
        assert_eq!(
            fixtures::secret_fault(Map::Secrets, name, &[(Name::Path, b"/etc/key")]).kind,
            Kind::Schema(Schema::Malformed { format: Format::Secret }),
            "row {row}"
        );
    }
}
#[test]
fn a_secret_without_a_path_is_a_missing_field() {
    assert_eq!(
        fixtures::secret_fault(Map::Secrets, b"key", &[]).kind,
        Kind::Schema(Schema::MissingField { field: copy_of(b"path") })
    );
}
#[test]
fn a_mode_that_is_not_octal_is_malformed() {
    assert_eq!(
        fixtures::secret_fault(Map::Secrets, b"key", &[(Name::Path, b"/etc/key"), (Name::Mode, b"0844")]).kind,
        Kind::Schema(Schema::Malformed { format: Format::Mode })
    );
}
#[test]
fn a_password_names_a_secret_by_user() {
    let expansion = fixtures::secret_entry(Map::Users, b"free", &[(Name::Password, b"users/free.passwd")]);
    assert_eq!(
        secrets::build(&expansion).expect("valid").passwords.as_ref(),
        &[secrets::Password { user: copy_of(b"free"), secret: copy_of(b"users/free.passwd") }]
    );
}
#[test]
fn an_ssh_host_key_needs_its_algorithm() {
    assert_eq!(
        fixtures::secret_fault(Map::Identity, b"ssh", &[(Name::Kind, b"ssh-host-key"), (Name::Path, b"/etc/key")]).kind,
        Kind::Schema(Schema::MissingField { field: copy_of(b"algorithm") })
    );
}
#[test]
fn an_identity_without_a_kind_is_a_missing_field() {
    assert_eq!(
        fixtures::secret_fault(Map::Identity, b"ssh", &[]).kind,
        Kind::Schema(Schema::MissingField { field: copy_of(b"kind") })
    );
}
#[test]
fn an_algorithm_but_the_three_is_an_enum_fault() {
    assert_eq!(
        fixtures::secret_fault(
            Map::Identity,
            b"ssh",
            &[(Name::Kind, b"ssh-host-key"), (Name::Algorithm, b"dsa"), (Name::Path, b"/etc/key")]
        )
        .kind,
        Kind::Schema(Schema::Enum {
            value: copy_of(b"dsa"),
            listed: Box::new([copy_of(b"ed25519"), copy_of(b"ecdsa"), copy_of(b"rsa")])
        })
    );
}
const BAD_KINDS: &[&[u8]] = &[b"wireguard-key", b"backup-key"];
#[test]
fn an_identity_kind_but_ssh_host_key_is_an_enum_fault() {
    for (row, kind) in BAD_KINDS.iter().enumerate() {
        assert_eq!(
            fixtures::secret_fault(
                Map::Identity,
                b"key",
                &[(Name::Kind, kind), (Name::Algorithm, b"rsa"), (Name::Path, b"/etc/key")]
            )
            .kind,
            Kind::Schema(Schema::Enum { value: copy_of(kind), listed: Box::new([copy_of(b"ssh-host-key")]) }),
            "row {row}"
        );
    }
}
#[test]
fn no_entry_makes_an_empty_part() {
    assert_eq!(
        secrets::build(&fixtures::concrete_expansion(Map::Secrets, &[])),
        Ok(Secrets { files: Box::new([]), passwords: Box::new([]), identity: Box::new([]) })
    );
}
#[test]
fn the_empty_secrets_digest_matches_the_design_vector() {
    let secrets = secrets::build(&fixtures::concrete_expansion(Map::Secrets, &[])).expect("empty part");
    assert_eq!(
        secrets::digest(&secrets),
        fixtures::digest("ac870b5c968ec04e58555e8ee45118c89bb9d37e50c0e9ee8fda233d395e9349")
    );
}

#[test]
fn a_secret_files_places_are_in_the_digest() {
    let secrets = Secrets {
        files: Box::new([fixtures::secret_file(
            b"mail/tls/mx1.key",
            b"/etc/ssl/private/mx1.key",
            0o600,
            b"root",
            b"root",
        )]),
        passwords: Box::new([]),
        identity: Box::new([]),
    };
    assert_eq!(
        secrets::digest(&secrets),
        fixtures::digest("450f91369d7d58ac445bde45d565516b887c1564d7cb20edc56367fdecaca7ce")
    );
}
