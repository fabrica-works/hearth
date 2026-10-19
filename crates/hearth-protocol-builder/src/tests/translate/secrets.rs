//! Secret records translated exhaustively (domain/render.md, 4.3).
use crate::{records::parts::secrets as source, render::parts::secrets as target};
#[must_use]
pub(super) fn back(secrets: source::Secrets) -> target::Secrets {
    let source::Secrets { files, passwords, identity } = secrets;
    let mut translated = skein_lib::List::with_capacity(u32::try_from(files.len()).expect("entry count"));
    for source::SecretFile { name, path, mode, owner, group } in files {
        translated.push(target::SecretFile { name, path, mode, owner, group }).expect("entry count");
    }
    let mut users = skein_lib::List::with_capacity(u32::try_from(passwords.len()).expect("entry count"));
    for source::Password { user, secret } in passwords {
        users.push(target::Password { user, secret }).expect("entry count");
    }
    let mut keys = skein_lib::List::with_capacity(u32::try_from(identity.len()).expect("entry count"));
    for source::Identity { name, kind, path, public_path, mode, owner, group } in identity {
        let source::IdentityKind::SshHostKey { algorithm } = kind;
        let algorithm = match algorithm {
            source::Algorithm::Ed25519 => target::Algorithm::Ed25519,
            source::Algorithm::Ecdsa => target::Algorithm::Ecdsa,
            source::Algorithm::Rsa => target::Algorithm::Rsa,
        };
        keys.push(target::Identity {
            name,
            kind: target::IdentityKind::SshHostKey { algorithm },
            path,
            public_path,
            mode,
            owner,
            group,
        })
        .expect("entry count");
    }
    target::Secrets { files: translated.into_boxed(), passwords: users.into_boxed(), identity: keys.into_boxed() }
}

#[test]
fn each_secrets_field_translates_to_the_records_and_back() {
    for (row, input) in super::super::fixtures::secret_records().into_iter().enumerate() {
        let translated = crate::translate::parts::secrets::secrets(input.clone());
        assert_eq!(back(translated), input, "row {row}: every secrets field");
    }
}
