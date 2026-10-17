use crate::{
    limits,
    vector::{Apt, Component, Name, Simulate, Sources, Timestamp, Variant},
};
use alloc::boxed::Box;
use skein_lib::List;
pub fn name(bytes: &[u8]) -> Name {
    Name::new(bytes, &limits::SHIPPED).unwrap()
}
pub fn forky_main() -> Sources {
    Sources {
        suite: name(b"forky"),
        architecture: name(b"amd64"),
        components: Box::from([Component::Main]),
        archive: Timestamp::new(b"20261001T030405Z").unwrap(),
        updates: false,
        security: None,
    }
}
pub fn simulate(names: &[&[u8]]) -> Simulate {
    let mut request = List::with_capacity(limits::SHIPPED.request);
    for n in names {
        request.push(name(n)).unwrap();
    }
    Simulate {
        sources: forky_main(),
        request: request.into_boxed(),
        variant: Variant::Apt,
        apt: Apt { install_recommends: false },
    }
}

pub fn opened() -> (
    skein_lib::Env<limits::Limits>,
    crate::output::Reader,
    skein_lib::Queue<crate::output::Seen>,
    skein_lib::Queue<skein_lib::stream::Down>,
) {
    let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits: limits::SHIPPED };
    let mut reader = crate::output::Reader::new(&env.limits);
    let mut below = skein_lib::Queue::with_capacity(1);
    crate::output::open(&mut reader, &env, &mut below);
    assert!(below.pop().is_some(), "open demands a scan");
    (env, reader, skein_lib::Queue::with_capacity(1), below)
}

pub const APT_LINES: [&[u8]; 9] = [
    b"Package: apt\n",
    b"Description: package manager\n",
    b" for Debian\n",
    b"Version: 3.1.6\n",
    b"Architecture: amd64\n",
    b"Filename: pool/main/a/apt/apt_3.1.6_amd64.deb\n",
    b"Size: 1736\n",
    b"SHA256: 2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae\n",
    b"\n",
];
pub const APT: &[u8] = b"Package: apt\nDescription: package manager\n for Debian\nVersion: 3.1.6\nArchitecture: amd64\nFilename: pool/main/a/apt/apt_3.1.6_amd64.deb\nSize: 1736\nSHA256: 2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae\n";
pub fn apt_stanza() -> crate::packages::Stanza {
    const TEXT: [&[u8]; 4] = [b"apt", b"3.1.6", b"amd64", b"pool/main/a/apt/apt_3.1.6_amd64.deb"];
    const EXPECTED: (u64, [u8; 32]) = (
        1736,
        [
            44, 38, 180, 107, 104, 255, 198, 143, 249, 155, 69, 60, 29, 48, 65, 52, 19, 66, 45, 112, 100, 131, 191,
            160, 249, 138, 94, 136, 98, 102, 231, 174,
        ],
    );
    crate::packages::Stanza {
        package: name(TEXT[0]),
        version: skein_lib::bytes::copy_of(TEXT[1]),
        architecture: name(TEXT[2]),
        filename: skein_lib::bytes::copy_of(TEXT[3]),
        size: EXPECTED.0,
        sha256: EXPECTED.1,
    }
}
pub fn opened_list() -> (
    skein_lib::Env<limits::Limits>,
    crate::packages::Stanzas,
    skein_lib::Queue<crate::packages::Found>,
    skein_lib::Queue<skein_lib::stream::Down>,
) {
    let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits: limits::SHIPPED };
    let mut reader = crate::packages::Stanzas::new(&env.limits);
    let mut below = skein_lib::Queue::with_capacity(crate::packages::UP_MAX_OUT.below);
    crate::packages::open(&mut reader, &env, &mut below);
    assert!(below.pop().is_some(), "open demands a scan");
    (env, reader, skein_lib::Queue::with_capacity(crate::packages::UP_MAX_OUT.above), below)
}
