use super::*;
pub fn config() -> crate::parts::config::Config {
    use crate::{
        parts::config::{self, Config},
        revision::Mode,
    };
    Config {
        groups: Box::new([]),
        users: Box::new([]),
        members: Box::new([]),
        tree: Box::new([
            config::file(
                b"etc/motd",
                Mode::File,
                digest("775324fcea7f2cdcbf11df25550e362669caceb6e69c8370374d2d436bb63123"),
            ),
            config::file(
                b"usr/local/bin/motd",
                Mode::Executable,
                digest("f487088ec8b18ff569dfede6f12e5fa91983f9f6dd2f3e0b6cee4349a231e745"),
            ),
        ]),
    }
}

pub fn spec(target: &[u8], parts: &[crate::parts::Part]) -> crate::parts::Spec {
    let mut meta = meta();
    meta.target = bytes::copy_of(target);
    crate::parts::Spec {
        resolution: None,
        meta,
        meta_digest: Digest([1; 32]),
        digest: Digest([2; 32]),
        commit: crate::revision::Oid(bytes::copy_of(b"0123456789abcdef0123456789abcdef01234567")),
        parts: parts.into(),
    }
}
