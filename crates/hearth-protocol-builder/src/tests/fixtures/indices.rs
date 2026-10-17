use super::{REQUEST, mmdebstrap_name, roots, source_lines};
use crate::{Asked, builder::vocabulary::Event, indices::Indices, limits, render};
use hearth_protocol_mmdebstrap::{packages, vector};
use skein_io::file;
use skein_lib::{Env, List, Queue, Time, Token, Wall, bytes};
pub const INDEX_READ: u32 = 64;
pub const INRELEASE_NAME: &[u8] = b"one_dists_forky_InRelease";
pub const PACKAGES_NAME: &[u8] = b"one_dists_forky_main_binary-amd64_Packages";
pub const INDEX_NAMES: &[&[u8]] = &[INRELEASE_NAME, PACKAGES_NAME];
pub const INDEX_PACKAGES: &[u8] = b"Package:apt\nVersion:1\nArchitecture:all\nFilename:a\nSize:1\nSHA256:0000000000000000000000000000000000000000000000000000000000000000";
pub const INDEX_DIRECTORY: Token = Token::new(89);
pub const INDEX_FILE: Token = Token::new(90);
pub struct ReadIndices {
    pub result: Result<Box<[render::resolution::Line]>, ()>,
    pub events: List<render::vocabulary::Event>,
    pub steps: Box<[u8]>,
    pub read: usize,
    pub closed: u32,
}
pub fn index_sources() -> Box<[vector::Line]> {
    let mut lines = List::with_capacity(1);
    for line in source_lines().into_iter().take(1) {
        lines.push(line).expect("source line");
    }
    lines.into_boxed()
}
pub fn read_indices(names: &[&[u8]], release: &[u8], packages: &[u8]) -> ReadIndices {
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits::Limits { ..limits::SHIPPED } };
    let mut reader = Indices::new(
        REQUEST,
        roots().staging.fd,
        b"scratch",
        index_sources(),
        &mmdebstrap_name(b"amd64"),
        INDEX_READ,
        &env,
    );
    let mut above = Queue::with_capacity(1);
    let mut below = Queue::with_capacity(1);
    let mut run =
        ReadIndices { result: Err(()), events: List::with_capacity(8), steps: Box::new([]), read: 0, closed: 0 };
    let mut steps = List::with_capacity(4096);
    let mut content = b"".as_slice();
    let mut page = 0_usize;
    for _ in 0_u32..4096 {
        if let Some(success) = reader.resume(&env, &mut above, &mut below) {
            if success {
                run.result = Ok(reader.lines());
            }
            run.steps = steps.into_boxed();
            return run;
        }
        if let Some(Event::Render(event)) = above.pop() {
            run.events.push(event).expect("stanzas");
        }
        let Some(Asked::File(request)) = below.pop() else { continue };
        let event = match request {
            file::Request::OpenDirectory { owner, no_follow, .. } => {
                assert!(no_follow);
                index_step(&mut steps, b"directory");
                file::Event::Opened { owner, file: INDEX_DIRECTORY, len: 0 }
            }
            file::Request::ListPage { owner, max, .. } => {
                index_step(&mut steps, b"list");
                let end = page.saturating_add(usize::try_from(max).expect("page bound")).min(names.len());
                let entries = index_entries(names.get(page..end).expect("page range"));
                page = end;
                file::Event::Listed { owner, entries }
            }
            file::Request::OpenReadNoFollow { owner, name, .. } => {
                content = if name.ends_with(b"_InRelease") { release } else { packages };
                index_step(&mut steps, &name);
                file::Event::Opened {
                    owner,
                    file: INDEX_FILE,
                    len: u64::try_from(content.len()).expect("fixture size"),
                }
            }
            file::Request::ReadAt { owner, offset, max, .. } => {
                assert!(max <= INDEX_READ);
                let start = usize::try_from(offset).expect("fixture offset");
                let end = start.saturating_add(usize::try_from(max).expect("read bound")).min(content.len());
                let bytes = bytes::copy_of(content.get(start..end).expect("read range"));
                run.read = run.read.checked_add(bytes.len()).expect("bounded reads");
                file::Event::Read { owner, bytes }
            }
            file::Request::Close { owner, .. } => {
                run.closed = run.closed.checked_add(1).expect("close count");
                file::Event::Closed { owner }
            }
            file::Request::Lock { .. }
            | file::Request::OpenPrivate { .. }
            | file::Request::MakeDirectory { .. }
            | file::Request::Create { .. }
            | file::Request::CreateNoFollow { .. }
            | file::Request::OpenRead { .. }
            | file::Request::OpenWrite { .. }
            | file::Request::OpenWriteNoFollow { .. }
            | file::Request::Load { .. }
            | file::Request::Scan { .. }
            | file::Request::Stat { .. }
            | file::Request::Store { .. }
            | file::Request::WriteAt { .. }
            | file::Request::Cut { .. }
            | file::Request::Sync { .. }
            | file::Request::SyncDirectory { .. }
            | file::Request::Rename { .. }
            | file::Request::Remove { .. }
            | file::Request::List { .. } => panic!("index read operation"),
        };
        reader.up(event, &env);
    }
    panic!("bounded index reader settles");
}
#[must_use]
pub fn package_stanzas() -> [packages::Stanza; 2] {
    super::insts().map(|inst| packages::Stanza {
        package: inst.name,
        version: inst.version,
        architecture: inst.architecture,
        filename: b"pool/package.deb".as_slice().into(),
        size: 7,
        sha256: super::ABC.0,
    })
}

pub const INDEX_PACKAGES_SIZE: u64 = 128;
pub const INDEX_PACKAGES_SHA256: render::vocabulary::Digest = render::vocabulary::Digest([
    0xd7, 0x6e, 0xbe, 0x56, 0x94, 0xd2, 0x21, 0x1b, 0x1c, 0x9d, 0x09, 0xa0, 0xc0, 0xb0, 0xfc, 0xcb, 0x83, 0xe9, 0x9c,
    0xe0, 0x1f, 0x43, 0x6b, 0x41, 0xf1, 0x4b, 0xb0, 0x8a, 0x46, 0x42, 0x93, 0x88,
]);
pub const EMPTY_SHA256: render::vocabulary::Digest = render::vocabulary::Digest([
    0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14, 0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f, 0xb9, 0x24, 0x27, 0xae, 0x41,
    0xe4, 0x64, 0x9b, 0x93, 0x4c, 0xa4, 0x95, 0x99, 0x1b, 0x78, 0x52, 0xb8, 0x55,
]);
pub const INDEX_STEPS: &[u8] = b"directory\nlist\nscratch/root/var/lib/apt/lists/one_dists_forky_InRelease\nscratch/root/var/lib/apt/lists/one_dists_forky_main_binary-amd64_Packages\n";
pub fn read_test_indices() -> ReadIndices {
    read_indices(INDEX_NAMES, b"abc", INDEX_PACKAGES)
}

pub const RESOLVE_INDEX_NAMES: &[&[u8]] = &[
    b"snapshot.debian.org_archive_debian_20261001T030405Z_dists_forky_InRelease",
    b"snapshot.debian.org_archive_debian_20261001T030405Z_dists_forky_main_binary-amd64_Packages",
];

fn index_step(steps: &mut List<u8>, step: &[u8]) {
    for byte in step {
        steps.push(*byte).expect("index steps");
    }
    steps.push(b'\n').expect("newline");
}
pub fn index_entries(names: &[&[u8]]) -> Box<[file::Entry]> {
    let mut entries = List::with_capacity(u32::try_from(names.len()).expect("entry count"));
    for name in names {
        entries
            .push(file::Entry { name: bytes::copy_of(name), kind: skein_io::kernel::Kind::File })
            .expect("entry count");
    }
    entries.into_boxed()
}
