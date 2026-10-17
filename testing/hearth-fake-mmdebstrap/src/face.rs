//! Render's simulate vector and apt's list names (render.md, 5.2 and 5.3).
use crate::resolve::{Failure, Field, Variant};
use hearth_fake_machine::hearth::mirror::Timestamp;
use std::fmt::Write;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Simulate {
    pub variant: Variant,
    pub architecture: Box<[u8]>,
    pub include: Box<[Box<[u8]>]>,
    pub proxy: Option<u16>,
    pub suite: Box<[u8]>,
    pub scratch: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Source {
    pub uri: Box<[u8]>,
    pub archive: Box<[u8]>,
    pub at: Timestamp,
    pub suite: Box<[u8]>,
    pub components: Box<[Box<[u8]>]>,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Inst {
    pub name: Box<[u8]>,
    pub version: Box<[u8]>,
    pub architecture: Box<[u8]>,
    pub origins: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Outcome {
    Resolved(Box<[Inst]>),
    Unreachable,
    NotFound { line: u32 },
    Failed { essential: Box<[Inst]>, providers: Box<[Inst]>, failure: Failure },
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Printed {
    pub stdout: Box<[u8]>,
    pub stderr: Box<[u8]>,
    pub exit: u8,
}
/// Accept precisely the invocation hearth writes, with no inherited environment.
#[must_use]
pub fn check(args: &[Box<[u8]>], env: &[Box<[u8]>]) -> Simulate {
    let mut args = args.iter().map(AsRef::as_ref);
    for fixed in [b"--verbose".as_slice(), b"--simulate", b"--mode=chrootless", b"--variant=apt"] {
        assert_eq!(args.next(), Some(fixed), "render.md 5.2's ordered vector");
    }
    let architecture =
        args.next().expect("architecture").strip_prefix(b"--architectures=").expect("architecture option");
    assert!(!architecture.is_empty(), "one architecture");
    let mut include = Vec::new();
    let mut next = args.next().expect("apt options");
    while let Some(names) = next.strip_prefix(b"--include=") {
        for name in names.split(|byte| *byte == b',') {
            assert!(!name.is_empty(), "a nonempty include name");
            include.push(Box::from(name));
        }
        next = args.next().expect("apt options after includes");
    }
    assert_eq!(next, b"--aptopt=Acquire::Check-Valid-Until \"false\"");
    assert_eq!(
        args.next(),
        Some(b"--aptopt=APT::Install-Recommends \"false\"".as_slice()),
        "APT::Install-Recommends must be false"
    );
    next = args.next().expect("skip or proxy");
    let proxy = next.strip_prefix(b"--aptopt=Acquire::http::Proxy \"http://127.0.0.1:").map(|port| {
        let port = port.strip_suffix(b"/\"").expect("loopback proxy URL");
        let port = core::str::from_utf8(port).expect("ASCII port").parse().expect("u16 port");
        next = args.next().expect("skip after proxy");
        port
    });
    assert_eq!(next, b"--skip=cleanup/apt/lists");
    let suite = args.next().expect("suite").into();
    let scratch: Box<[u8]> = args.next().expect("root").strip_suffix(b"/root").expect("scratch root").into();
    assert!(scratch.starts_with(b"/") && !scratch.ends_with(b"/"), "absolute scratch");
    assert_eq!(args.next(), Some([scratch.as_ref(), b"/sources.list"].concat().as_slice()));
    assert_eq!(args.next(), None, "no further arguments");
    let expected: Box<[Box<[u8]>]> = Box::new([
        b"LC_ALL=C".as_slice().into(),
        b"PATH=/usr/sbin:/usr/bin:/sbin:/bin".as_slice().into(),
        [b"TMPDIR=".as_slice(), &scratch].concat().into(),
    ]);
    assert_eq!(env, expected.as_ref(), "the complete environment of render.md 5.2");
    Simulate {
        variant: Variant::Apt,
        architecture: architecture.into(),
        include: include.into(),
        proxy,
        suite,
        scratch,
    }
}
/// Decode the source lines hearth writes; their order is the source preference.
#[must_use]
pub fn sources(text: &[u8]) -> Box<[Source]> {
    text.split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| {
            let mut fields = line.split(|byte| *byte == b' ');
            assert_eq!(fields.next(), Some(b"deb".as_slice()));
            let uri = fields.next().expect("source URI");
            let path = uri.strip_prefix(b"http://snapshot.debian.org/archive/").expect("snapshot archive");
            let mut path = path.split(|byte| *byte == b'/');
            let archive = path.next().expect("archive").into();
            let at = Timestamp::new(path.next().expect("timestamp")).expect("snapshot timestamp");
            assert_eq!(path.next(), Some(b"".as_slice()), "URI ends with slash");
            assert_eq!(path.next(), None);
            let suite = fields.next().expect("suite").into();
            let components = fields.map(Box::from).collect();
            Source { uri: uri.into(), archive, at, suite, components }
        })
        .collect()
}
/// Apt's uncompressed list name for one source-relative path.
#[must_use]
pub fn list_name(source: &Source, path: &[u8]) -> Box<[u8]> {
    [source.uri.strip_prefix(b"http://").expect("HTTP source"), b"dists/", &source.suite, b"/", path]
        .concat()
        .into_iter()
        .map(|byte| if byte == b'/' { b'_' } else { byte })
        .collect()
}

fn text(bytes: &[u8]) -> &str {
    core::str::from_utf8(bytes).expect("Debian's ASCII fields")
}
fn insts(out: &mut String, packages: &[Inst]) {
    for package in packages {
        writeln!(
            out,
            "Inst {} ({} {} [{}])",
            text(&package.name),
            text(&package.version),
            text(&package.origins),
            text(&package.architecture)
        )
        .expect("string write");
    }
}
/// The recorded mmdebstrap 1.5.7 / apt 3.0.3 lines, with the run's fields.
#[must_use]
pub fn print(simulate: &Simulate, sources: &[Source], outcome: &Outcome) -> Printed {
    let mut out = String::from(
        "I: skipping emulation check in chrootless mode\nI: finding correct signed-by value...\nI: automatically chosen format: directory\nI: running apt-get update...\n",
    );
    match outcome {
        Outcome::Resolved(packages) => {
            out.push_str("Reading package lists...\nI: simulate downloading packages with apt...\n");
            insts(&mut out, packages);
            out.push_str("I: skipping cleanup/apt/lists as requested\nI: cleaning apt cache...\nI: simulate creating tarball...\nI: success in 0 seconds\n");
        }
        Outcome::Unreachable => {
            let proxy = simulate.proxy.expect("the recorded unreachable proxy");
            for (line, source) in sources.iter().enumerate() {
                let uri = text(&source.uri).trim_end_matches('/');
                for _ in 0..3 {
                    writeln!(out, "Ign:{} {uri} {} InRelease", line + 1, text(&source.suite)).expect("string write");
                }
                writeln!(out, "Err:{} {uri} {} InRelease\n  Could not connect to 127.0.0.1:{proxy} (127.0.0.1). - connect (111: Connection refused)\n  Unable to connect to 127.0.0.1:{proxy}:", line + 1, text(&source.suite)).expect("string write");
            }
            out.push_str("Reading package lists...\n");
            for source in sources {
                writeln!(
                    out,
                    "E: Failed to fetch {}dists/{}/InRelease  Unable to connect to 127.0.0.1:{proxy}:",
                    text(&source.uri),
                    text(&source.suite)
                )
                .expect("string write");
            }
            out.push_str("E: Some index files failed to download. They have been ignored, or old ones used instead.\n");
            update_failed(&mut out);
        }
        Outcome::NotFound { line } => {
            let source = &sources[usize::try_from(*line - 1).expect("source fits")];
            let uri = text(&source.uri).trim_end_matches('/');
            let suite = text(&source.suite);
            writeln!(out, "Ign:{line} {uri} {suite} InRelease\nErr:{} {uri} {suite} Release", line + 1)
                .expect("string write");
            if let Some(proxy) = simulate.proxy {
                writeln!(out, "  404  Not Found [IP: 127.0.0.1 {proxy}]").expect("string write");
            }
            out.push_str("Reading package lists...\n");
            writeln!(out, "E: The repository '{uri} {suite} Release' does not have a Release file.")
                .expect("string write");
            update_failed(&mut out);
        }
        Outcome::Failed { essential, providers, failure } => {
            insts(&mut out, essential);
            match failure {
                Failure::Unknown(name) => {
                    writeln!(out, "E: Unable to locate package {}", text(name)).expect("string write");
                }
                Failure::NoCandidate(name) => {
                    writeln!(out, "Package {} is a virtual package provided by:", text(name)).expect("string write");
                    for provider in providers.iter().rev() {
                        writeln!(out, "  {} {}", text(&provider.name), text(&provider.version)).expect("string write");
                    }
                    writeln!(out, "\nE: Package '{}' has no installation candidate", text(name)).expect("string write");
                }
                Failure::Unsatisfiable(broken) => {
                    out.push_str("The following packages have unmet dependencies:\n");
                    let mut broken = broken.iter().collect::<Vec<_>>();
                    broken.sort_by(|a, b| a.package.cmp(&b.package));
                    for relation in broken {
                        assert_eq!(relation.field, Field::Conflicts, "only Conflicts has a recorded unmet block");
                        write!(out, " {} : Conflicts: {}", text(&relation.package), text(&relation.relation))
                            .expect("string write");
                        if let Some(version) = &relation.against {
                            write!(out, " but {} is to be installed", text(version)).expect("string write");
                        }
                        out.push('\n');
                    }
                    out.push_str("E: Unable to correct problems, you have held broken packages.\n");
                }
            }
            write!(out, "E: setup failed: E: apt-get --yes -oDPkg::Chroot-Directory= -oDPkg::Options::=--force-not-root -oDPkg::Options::=--force-script-chrootless -oDPkg::Options::=--root={}/root -oDPkg::Options::=--log={}/root/var/log/dpkg.log -oAPT::Get::Simulate=true install -oAPT::Status-Fd=<$fd> -oDpkg::Use-Pty=false", text(&simulate.scratch), text(&simulate.scratch)).expect("string write");
            for name in &simulate.include {
                write!(out, " {}", text(name)).expect("string write");
            }
            out.push_str(" apt failed: process exited with 100 and error in console output\n");
        }
    }
    let exit = if matches!(outcome, Outcome::Resolved(_)) {
        0
    } else {
        out.push_str("W: hooklistener errored out: E: received eof on socket\n\nI: main() received signal PIPE: waiting for setup...\nE: mmdebstrap failed to run\n");
        25
    };
    Printed { stdout: Box::default(), stderr: out.into_bytes().into(), exit }
}
fn update_failed(out: &mut String) {
    out.push_str("E: apt-get update --error-on=any -oAPT::Status-Fd=<$fd> -oDpkg::Use-Pty=false failed: process exited with 100 and error in console output\n");
}
