//! Session 30.6's store names and command-line input models.
use hearth::{
    Call, Form, Invocation,
    records::startup::{Misuse, Startup},
};
use hearth_domain_render::vocabulary::Digest;
use hearth_domain_store::vocabulary::{Directory, Shape};
use skein_io::kernel::Kind;
use skein_lib::Rng;
use std::{
    ffi::OsString,
    os::unix::ffi::{OsStrExt, OsStringExt},
    path::{Path, PathBuf},
};
/// Each sweep draws five thousand small inputs from session 30.6's block.
pub const CASES: core::ops::Range<u64> = 0..5_000;
pub const SEED: u64 = 0x0030_6000;
/// Zero through three targets exercise empty, singleton and plural argument vectors.
const TARGETS: u64 = 3;
pub const CWD: &str = "/";
const HEX: &[u8] = b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const DIGEST: Digest = Digest([0xaa; 32]);
pub struct Name {
    pub directory: Directory,
    pub prefix: &'static [u8],
    pub suffix: &'static [u8],
    pub kind: Kind,
    pub expected: Shape,
    pub damaged: Shape,
}
pub const NAMES: &[Name] = &[
    Name {
        directory: Directory::Top,
        prefix: b"cache",
        suffix: b"",
        kind: Kind::Directory,
        expected: Shape::Fixed,
        damaged: Shape::Stray,
    },
    Name {
        directory: Directory::Top,
        prefix: b"store.record",
        suffix: b".skein-0123456789abcdef",
        kind: Kind::File,
        expected: Shape::Temporary,
        damaged: Shape::Stray,
    },
    Name {
        directory: Directory::Cache,
        prefix: HEX,
        suffix: b"",
        kind: Kind::File,
        expected: Shape::File(DIGEST),
        damaged: Shape::Stray,
    },
    Name {
        directory: Directory::Cache,
        prefix: HEX,
        suffix: b".record",
        kind: Kind::File,
        expected: Shape::Record(DIGEST),
        damaged: Shape::Stray,
    },
    Name {
        directory: Directory::Resolutions,
        prefix: HEX,
        suffix: b".record",
        kind: Kind::File,
        expected: Shape::Record(DIGEST),
        damaged: Shape::Stray,
    },
    Name {
        directory: Directory::Resolutions,
        prefix: HEX,
        suffix: b"",
        kind: Kind::File,
        expected: Shape::Stray,
        damaged: Shape::Stray,
    },
    Name {
        directory: Directory::Cache,
        prefix: HEX,
        suffix: b".tmp",
        kind: Kind::File,
        expected: Shape::Temporary,
        damaged: Shape::Stray,
    },
    Name {
        directory: Directory::Resolutions,
        prefix: HEX,
        suffix: b".tmp",
        kind: Kind::File,
        expected: Shape::Stray,
        damaged: Shape::Stray,
    },
    Name {
        directory: Directory::Resolutions,
        prefix: HEX,
        suffix: b".record.skein-0123456789abcdef",
        kind: Kind::File,
        expected: Shape::Temporary,
        damaged: Shape::Stray,
    },
    Name {
        directory: Directory::Cache,
        prefix: HEX,
        suffix: b".record",
        kind: Kind::Directory,
        expected: Shape::Stray,
        damaged: Shape::Stray,
    },
    Name {
        directory: Directory::Cache,
        prefix: HEX,
        suffix: b".record",
        kind: Kind::Symlink,
        expected: Shape::Stray,
        damaged: Shape::Stray,
    },
];
#[must_use]
pub fn name(row: &Name) -> Vec<u8> {
    [row.prefix, row.suffix].concat()
}
#[must_use]
pub fn damaged(rng: &mut Rng, name: &[u8]) -> Vec<u8> {
    let mut bytes = name.to_vec();
    let at = usize::try_from(rng.below(bytes.len() as u64)).expect("nonempty name");
    bytes[at] = u8::MAX;
    bytes
}
#[must_use]
pub fn invocation(rng: &mut Rng) -> Invocation {
    drawn(rng, None)
}
/// One input model for each supported command, render and diff.
pub const ROUND_TRIP_ROWS: usize = 2;
#[must_use]
pub fn invocations(rng: &mut Rng) -> [Invocation; ROUND_TRIP_ROWS] {
    let input = invocation(rng);
    let diff = matches!(input.call, Call::Render { .. });
    [input, drawn(rng, Some(diff))]
}
fn drawn(rng: &mut Rng, diff: Option<bool>) -> Invocation {
    let revision = format!("branch{}", rng.below(u64::from(u16::MAX))).into_bytes().into_boxed_slice();
    let targets: Box<[_]> = (0..rng.below(TARGETS + 1))
        .map(|target| format!("node{target}-prod").into_bytes().into_boxed_slice())
        .collect();
    let resolve = rng.chance(500);
    let repository = Box::<[u8]>::from(CWD.as_bytes());
    Invocation {
        proxy: None,
        form: if rng.chance(500) { Form::Text } else { Form::JsonLines },
        store: PathBuf::from(format!("/store{}", rng.below(u64::from(u16::MAX)))),
        call: if diff.unwrap_or_else(|| rng.chance(500)) {
            Call::Diff { resolve, repository, from: revision, to: b"HEAD".as_slice().into(), targets }
        } else {
            Call::Render { resolve, repository, revision, targets }
        },
    }
}
/// Encode an input invocation independently of the argument parser.
#[must_use]
pub fn arguments(invocation: &Invocation) -> Vec<OsString> {
    let mut args = Vec::new();
    let (resolve, repository, targets) = match &invocation.call {
        Call::Render { resolve, repository, revision, targets } => {
            args.push(OsString::from("render"));
            args.push(OsString::from("--at"));
            args.push(OsString::from_vec(revision.to_vec()));
            (*resolve, repository, targets)
        }
        Call::Diff { resolve, repository, from, to, targets } => {
            args.push(OsString::from("diff"));
            args.push(OsString::from_vec(from.to_vec()));
            args.push(OsString::from_vec(to.to_vec()));
            (*resolve, repository, targets)
        }
    };
    args.extend([
        OsString::from("--repo"),
        OsString::from_vec(repository.to_vec()),
        OsString::from("--store"),
        invocation.store.as_os_str().to_owned(),
    ]);
    if resolve {
        args.push(OsString::from("--resolve"));
    }
    if invocation.form == Form::JsonLines {
        args.push(OsString::from("--json"));
    }
    for target in targets {
        if matches!(invocation.call, Call::Diff { .. }) {
            args.push(OsString::from("--target"));
        }
        args.push(OsString::from_vec(target.to_vec()));
    }
    args
}
pub enum Change {
    Append,
    Command,
    Revision,
    LeadingJson,
}
pub struct Mutation {
    pub word: &'static [u8],
    pub misuse: Misuse,
    pub form: Form,
    pub change: Change,
}
pub const MUTATIONS: &[Mutation] = &[
    Mutation { word: b"--unknown", misuse: Misuse::Option, form: Form::Text, change: Change::Append },
    Mutation { word: b"--repo", misuse: Misuse::Value, form: Form::JsonLines, change: Change::Append },
    Mutation { word: b"-\xff", misuse: Misuse::Option, form: Form::Text, change: Change::Append },
    Mutation { word: b"\xff", misuse: Misuse::Command, form: Form::JsonLines, change: Change::Command },
    Mutation { word: b"bad~revision", misuse: Misuse::Revision, form: Form::Text, change: Change::Revision },
    Mutation { word: b"--json", misuse: Misuse::Command, form: Form::JsonLines, change: Change::LeadingJson },
];
#[must_use]
pub fn mutated(invocation: &Invocation, row: &Mutation) -> Vec<OsString> {
    let mut input = invocation.clone();
    input.form = row.form;
    let mut args = arguments(&input);
    let word = OsString::from_vec(row.word.to_vec());
    match row.change {
        Change::Append => args.push(word),
        Change::Command => return std::iter::once(word).chain(args.into_iter().skip(1)).collect(),
        Change::LeadingJson => return std::iter::once(word).chain(args).collect(),
        Change::Revision => {
            *args.iter_mut().find(|word| word.as_bytes().starts_with(b"branch")).expect("drawn revision") = word;
        }
    }
    args
}
pub fn parse(args: &[OsString]) -> Result<Invocation, Startup> {
    hearth::argv::invocation(args, None, None, Path::new(CWD))
}
#[must_use]
pub fn refusal(row: &Mutation) -> Startup {
    Startup::Usage { misuse: row.misuse, word: row.word.into() }
}

/// Build one seed's random input state for each independent property.
pub fn each(mut check: impl FnMut(u64, &mut Rng)) {
    for seed in CASES {
        check(seed, &mut Rng::new(SEED + seed));
    }
}
