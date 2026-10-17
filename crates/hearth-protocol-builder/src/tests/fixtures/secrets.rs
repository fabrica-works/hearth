use crate::render::parts::secrets::{Algorithm, Identity, IdentityKind, Password, SecretFile, Secrets};
use skein_lib::bytes::copy_of;
#[must_use]
pub fn secret_records() -> [Secrets; 3] {
    [
        (Algorithm::Ed25519, Some(b"/etc/ssh/key.pub".as_slice())),
        (Algorithm::Ecdsa, None),
        (Algorithm::Rsa, Some(b"/etc/ssh/key.pub".as_slice())),
    ]
    .map(|(algorithm, public_path)| {
        let mut public = None;
        if let Some(path) = public_path {
            public = Some(copy_of(path));
        }
        Secrets {
            files: Box::new([SecretFile {
                name: copy_of(b"tls/key"),
                path: copy_of(b"/etc/key"),
                mode: 0o640,
                owner: copy_of(b"alice"),
                group: copy_of(b"ops"),
            }]),
            passwords: Box::new([Password { user: copy_of(b"alice"), secret: copy_of(b"users/alice") }]),
            identity: Box::new([Identity {
                name: copy_of(b"ssh"),
                kind: IdentityKind::SshHostKey { algorithm },
                path: copy_of(b"/etc/ssh/key"),
                public_path: public,
                mode: 0o600,
                owner: copy_of(b"root"),
                group: copy_of(b"root"),
            }]),
        }
    })
}
