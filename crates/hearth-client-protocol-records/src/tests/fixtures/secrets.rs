use crate::parts::secrets::{Algorithm, Identity, IdentityKind, Password, SecretFile, Secrets};
use skein_lib::bytes::copy_of;
pub fn secret_record(algorithm: Algorithm) -> Secrets {
    Secrets {
        files: Box::new([SecretFile {
            name: copy_of(b"tls/key"),
            path: copy_of(b"/etc/key"),
            mode: 0o640,
            owner: copy_of(b"root"),
            group: copy_of(b"ops"),
        }]),
        passwords: Box::new([Password { user: copy_of(b"free"), secret: copy_of(b"users/free.passwd") }]),
        identity: Box::new([Identity {
            name: copy_of(b"ssh"),
            kind: IdentityKind::SshHostKey { algorithm },
            path: copy_of(b"/etc/ssh/key"),
            public_path: Some(copy_of(b"/etc/ssh/key.pub")),
            mode: 0o600,
            owner: copy_of(b"root"),
            group: copy_of(b"root"),
        }]),
    }
}
pub fn secret_spec() -> crate::records::Record {
    let mut spec = super::x1_prod();
    spec.parts = Box::new([crate::records::Part {
        digest: [0; 32],
        record: crate::records::PartRecord::Secrets(secret_record(Algorithm::Ed25519)),
    }]);
    crate::records::Record::Spec(spec)
}
