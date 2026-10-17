//! Secret names and places (format.md, 9.4).
use crate::form::text::Lines;
use alloc::boxed::Box;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Secrets {
    pub files: Box<[SecretFile]>,
    pub passwords: Box<[Password]>,
    pub identity: Box<[Identity]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SecretFile {
    pub name: Box<[u8]>,
    pub path: Box<[u8]>,
    pub mode: u16,
    pub owner: Box<[u8]>,
    pub group: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Password {
    pub user: Box<[u8]>,
    pub secret: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Identity {
    pub name: Box<[u8]>,
    pub kind: IdentityKind,
    pub path: Box<[u8]>,
    pub public_path: Option<Box<[u8]>>,
    pub mode: u16,
    pub owner: Box<[u8]>,
    pub group: Box<[u8]>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum IdentityKind {
    SshHostKey { algorithm: Algorithm },
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Algorithm {
    Ed25519,
    Ecdsa,
    Rsa,
}

impl Secrets {
    pub(crate) fn text(&self, lines: &mut Lines) {
        for file in &self.files {
            lines.put(b"  secret");
            lines.pad_to(13);
            lines.put(&file.name);
            lines.put(b" ");
            place(lines, &file.path, file.mode, &file.owner, &file.group);
        }
        for password in &self.passwords {
            lines.put(b"  password");
            lines.pad_to(13);
            lines.put(&password.user);
            lines.put(b" ");
            lines.put(&password.secret);
            lines.put(b"\n");
        }
        for item in &self.identity {
            lines.put(b"  identity");
            lines.pad_to(13);
            lines.put(&item.name);
            lines.put(b" ssh-host-key ");
            lines.put(algorithm(item.kind));
            lines.put(b" ");
            lines.put(&item.path);
            lines.put(b" ");
            lines.put(item.public_path.as_deref().unwrap_or(b"-"));
            lines.put(b" ");
            owned(lines, item.mode, &item.owner, &item.group);
        }
    }
    pub(crate) fn json(&self, json: &mut skein_json::writer::Encoder) {
        json.object_start();
        json.key(b"files");
        json.object_start();
        for file in &self.files {
            json.key(&file.name);
            json.object_start();
            json.key(b"group");
            json.string(&file.group);
            json_place(json, &file.path, file.mode, &file.owner);
            json.object_end();
        }
        json.object_end();
        json.key(b"identity");
        json.object_start();
        for item in &self.identity {
            json.key(&item.name);
            json.object_start();
            json.key(b"algorithm");
            json.string(algorithm(item.kind));
            json.key(b"group");
            json.string(&item.group);
            json.key(b"kind");
            json.string(b"ssh-host-key");
            json_place(json, &item.path, item.mode, &item.owner);
            if let Some(path) = &item.public_path {
                json.key(b"public_path");
                json.string(path);
            }
            json.object_end();
        }
        json.object_end();
        json.key(b"passwords");
        json.object_start();
        for password in &self.passwords {
            json.key(&password.user);
            json.string(&password.secret);
        }
        json.object_end();
        json.object_end();
    }
}
fn algorithm(kind: IdentityKind) -> &'static [u8] {
    let IdentityKind::SshHostKey { algorithm } = kind;
    match algorithm {
        Algorithm::Ed25519 => b"ed25519",
        Algorithm::Ecdsa => b"ecdsa",
        Algorithm::Rsa => b"rsa",
    }
}
fn place(lines: &mut Lines, path: &[u8], mode: u16, owner: &[u8], group: &[u8]) {
    lines.put(path);
    lines.put(b" ");
    owned(lines, mode, owner, group);
}
fn owned(lines: &mut Lines, mode: u16, owner: &[u8], group: &[u8]) {
    lines.mode(mode);
    lines.put(b" ");
    lines.put(owner);
    lines.put(b":");
    lines.put(group);
    lines.put(b"\n");
}
fn json_place(json: &mut skein_json::writer::Encoder, path: &[u8], mode: u16, owner: &[u8]) {
    json.key(b"mode");
    let mut digits = [b'0'; 4];
    for (digit, shift) in digits.iter_mut().zip([9_u32, 6, 3, 0]) {
        *digit = b'0'.saturating_add(u8::try_from((mode >> shift) & 7).expect("octal digit"));
    }
    json.string(&digits);
    json.key(b"owner");
    json.string(owner);
    json.key(b"path");
    json.string(path);
}
