//! Names and places for the activator, never values (render.md, 4.3).
use crate::{
    entries::Expansion,
    errors::Error,
    kinds,
    revision::declaration::{Map, Name},
};
use alloc::boxed::Box;
use hearth_lib::{
    encode::{Digest, Encoder},
    hash::H,
};
use skein_lib::{List, bytes::copy_of};
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
pub fn build(expansion: &Expansion) -> Result<Secrets, Box<[Error]>> {
    let files = expansion.of(Map::Secrets);
    let identity = expansion.of(Map::Identity);
    let users = expansion.of(Map::Users);
    let capacity =
        u32::try_from(files.len().saturating_add(identity.len()).saturating_add(users.len())).expect("expanded count");
    let mut faults = List::with_capacity(capacity);
    let mut kept = skein_lib::Map::with_capacity(capacity);
    let mut keys = skein_lib::Map::with_capacity(capacity);
    let mut passwords = skein_lib::Map::with_capacity(capacity);
    for located in &files {
        match kinds::secrets::file(located) {
            Ok(file) => {
                if !kept.contains_key(&file.name) {
                    kept.insert(file.name.clone(), file).expect("entry count");
                }
            }
            Err(error) => {
                faults.push(*error).expect("entry count");
            }
        }
    }
    for located in &identity {
        match kinds::identity::entry(located) {
            Ok(item) => {
                if !keys.contains_key(&item.name) {
                    keys.insert(item.name.clone(), item).expect("entry count");
                }
            }
            Err(error) => {
                faults.push(*error).expect("entry count");
            }
        }
    }
    for located in &users {
        if let Some(secret) = located.entry.text(Name::Password) {
            if let Err(schema) = kinds::secrets::formed(secret, crate::errors::Format::Secret) {
                faults.push(kinds::secrets::fault(located, Some(Name::Password), schema)).expect("entry count");
            } else if !passwords.contains_key(&located.entry.key) {
                passwords
                    .insert(
                        located.entry.key.clone(),
                        Password { user: located.entry.key.clone(), secret: copy_of(secret) },
                    )
                    .expect("entry count");
            }
        }
    }
    if !faults.is_empty() {
        return Err(faults.into_boxed());
    }
    let mut files = List::with_capacity(kept.len());
    for (_, file) in &kept {
        files.push(file.clone()).expect("entry count");
    }
    let mut identity = List::with_capacity(keys.len());
    for (_, item) in &keys {
        identity.push(item.clone()).expect("entry count");
    }
    let mut users = List::with_capacity(passwords.len());
    for (_, password) in &passwords {
        users.push(password.clone()).expect("entry count");
    }
    Ok(Secrets { files: files.into_boxed(), passwords: users.into_boxed(), identity: identity.into_boxed() })
}
#[must_use]
pub fn digest(secrets: &Secrets) -> Digest {
    let mut hash = H::new(b"part:secrets", 1);
    encode(secrets, hash.values());
    hash.finish()
}
fn owners(record: &mut Encoder, mode: u16, owner: &[u8], path: &[u8]) {
    record.key(b"mode");
    record.uint(u64::from(mode));
    record.key(b"owner");
    record.bytes(owner);
    record.key(b"path");
    record.bytes(path);
}
fn encode(secrets: &Secrets, record: &mut Encoder) {
    record.map(3);
    record.key(b"files");
    record.map(u64::try_from(secrets.files.len()).expect("entry count"));
    for file in &secrets.files {
        record.key(&file.name);
        record.map(4);
        record.key(b"group");
        record.bytes(&file.group);
        owners(record, file.mode, &file.owner, &file.path);
    }
    record.key(b"identity");
    record.map(u64::try_from(secrets.identity.len()).expect("entry count"));
    for item in &secrets.identity {
        record.key(&item.name);
        record.map(if item.public_path.is_some() { 7 } else { 6 });
        record.key(b"algorithm");
        let IdentityKind::SshHostKey { algorithm } = item.kind;
        record.bytes(match algorithm {
            Algorithm::Ed25519 => b"ed25519",
            Algorithm::Ecdsa => b"ecdsa",
            Algorithm::Rsa => b"rsa",
        });
        record.key(b"group");
        record.bytes(&item.group);
        record.key(b"kind");
        record.bytes(b"ssh-host-key");
        owners(record, item.mode, &item.owner, &item.path);
        if let Some(path) = &item.public_path {
            record.key(b"public_path");
            record.bytes(path);
        }
    }
    record.key(b"passwords");
    record.map(u64::try_from(secrets.passwords.len()).expect("entry count"));
    for password in &secrets.passwords {
        record.key(&password.user);
        record.bytes(&password.secret);
    }
}
