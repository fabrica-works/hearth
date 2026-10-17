use crate::resolve::Selected;
use hearth_fake_machine::hearth::universe::{Alternative, Universe};

pub const PORT: u16 = 3142;
pub const ARCHIVE: &[u8] = b"20261001T030405Z";
pub const SOURCES: &[u8] = b"deb http://snapshot.debian.org/archive/debian/20261001T030405Z/ forky main\n";
pub fn args(proxy: Option<u16>) -> Box<[Box<[u8]>]> {
    let mut args: Vec<Box<[u8]>> = [
        b"--verbose".as_slice(),
        b"--simulate",
        b"--mode=chrootless",
        b"--variant=apt",
        b"--architectures=amd64",
        b"--aptopt=Acquire::Check-Valid-Until \"false\"",
        b"--aptopt=APT::Install-Recommends \"false\"",
    ]
    .into_iter()
    .map(Box::from)
    .collect();
    if let Some(port) = proxy {
        args.push(format!("--aptopt=Acquire::http::Proxy \"http://127.0.0.1:{port}/\"").into_bytes().into());
    }
    args.extend(
        [b"--skip=cleanup/apt/lists".as_slice(), b"forky", b"/scratch/root", b"/scratch/sources.list"]
            .into_iter()
            .map(Box::from),
    );
    args.into()
}
pub fn env() -> Box<[Box<[u8]>]> {
    [b"LC_ALL=C".as_slice(), b"PATH=/usr/sbin:/usr/bin:/sbin:/bin", b"TMPDIR=/scratch"]
        .into_iter()
        .map(Box::from)
        .collect()
}
pub const BUILDER: hearth_fake_machine::linux::Ids = hearth_fake_machine::linux::Ids { user: 1000, group: 1000 };
pub fn builder(machine: &mut hearth_fake_machine::Machine) -> hearth_fake_machine::linux::Pid {
    machine.linux_mut().login(BUILDER, b"/scratch")
}
pub fn on_machine(proxy: Option<u16>) -> hearth_fake_machine::Machine {
    use hearth_fake_machine::{
        Machine,
        hearth::{
            Tool,
            mirror::{Mirror, Release, Suite, Timestamp},
        },
        linux::{Item, Sizes},
    };
    let mut machine = Machine::new(46, Sizes::tiny());
    machine.linux_mut().lay(&[
        Item::directory(b"/usr", 0o755),
        Item::directory(b"/usr/bin", 0o755),
        Item::directory(b"/scratch", 0o755).owner(BUILDER),
        Item::file(b"/scratch/sources.list", 0o644, SOURCES).owner(BUILDER),
    ]);
    machine.install(b"/usr/bin/mmdebstrap", Tool::Mmdebstrap);
    let mut mirror = Mirror::new();
    mirror.add(
        Timestamp::new(ARCHIVE).expect("fixture timestamp"),
        Suite::new(Release::debian(b"testing", b"forky"), b"amd64", Box::new([(b"main".as_slice().into(), debian())])),
    );
    machine.serve(mirror, proxy);
    machine
}
pub fn spawn_mmdebstrap(
    machine: &mut hearth_fake_machine::Machine,
    parent: hearth_fake_machine::linux::Pid,
    args: &[Box<[u8]>],
) -> (hearth_fake_machine::linux::Pid, [hearth_fake_machine::linux::Handle; 2]) {
    use hearth_fake_machine::linux::{At, How, Op, Pipe, Reply, Spawn, Way};
    let Reply::Opened(root) = machine
        .call(parent, Op::Open { at: At::Root, path: b"/".as_slice().into(), how: How::Directory })
        .expect("root opens")
    else {
        panic!("root handle");
    };
    let Reply::Spawned { child, pipes } = machine
        .call(
            parent,
            Op::Spawn(Spawn {
                program: b"/usr/bin/mmdebstrap".as_slice().into(),
                args: args.into(),
                env: env(),
                root,
                dir: b"scratch".as_slice().into(),
                pipes: Box::new([Pipe { child: 1, way: Way::Out }, Pipe { child: 2, way: Way::Out }]),
            }),
        )
        .expect("mmdebstrap spawns")
    else {
        panic!("spawned child");
    };
    machine.call(parent, Op::Close { handle: root }).expect("close root");
    (child, pipes.into_vec().try_into().expect("two output pipes"))
}
pub fn drain_both(
    machine: &mut hearth_fake_machine::Machine,
    parent: hearth_fake_machine::linux::Pid,
    stdout: hearth_fake_machine::linux::Handle,
    stderr: hearth_fake_machine::linux::Handle,
) -> Vec<u8> {
    use hearth_fake_machine::linux::{Op, Reply};
    let mut bytes = Vec::new();
    for pipe in [stdout, stderr] {
        loop {
            match machine.call(parent, Op::Take { pipe, most: 4096 }).expect("drain pipe") {
                Reply::Took(part) => bytes.extend_from_slice(&part),
                Reply::Empty | Reply::Ended => break,
                reply @ (Reply::Opened(_)
                | Reply::Read(_)
                | Reply::Done
                | Reply::Stat(_)
                | Reply::Listed(_)
                | Reply::Spawned { .. }
                | Reply::Sent(_)
                | Reply::Running
                | Reply::Exited(_)) => panic!("pipe answers bytes or end: {reply:?}"),
            }
        }
    }
    bytes
}
pub fn lines(bytes: &[u8]) -> impl Iterator<Item = &[u8]> {
    bytes.split(|byte| *byte == b'\n')
}
pub fn args_for(names: &[&[u8]], proxy: Option<u16>) -> Box<[Box<[u8]>]> {
    let mut args = args(proxy).into_vec();
    if !names.is_empty() {
        let mut include = b"--include=".to_vec();
        for (place, name) in names.iter().enumerate() {
            if place != 0 {
                include.push(b',');
            }
            include.extend_from_slice(name);
        }
        args.insert(5, include.into());
    }
    args.into()
}
pub struct Run {
    pub machine: hearth_fake_machine::Machine,
    pub parent: hearth_fake_machine::linux::Pid,
    pub child: hearth_fake_machine::linux::Pid,
    pub printed: Vec<u8>,
}
pub fn run_on(mut machine: hearth_fake_machine::Machine, names: &[&[u8]], proxy: Option<u16>, sources: &[u8]) -> Run {
    use hearth_fake_machine::linux::{At, How, Op, Reply};
    let parent = builder(&mut machine);
    let Reply::Opened(file) = machine
        .call(parent, Op::Open { at: At::Root, path: b"/scratch/sources.list".as_slice().into(), how: How::Write })
        .expect("sources open")
    else {
        panic!("source file handle");
    };
    machine.call(parent, Op::Truncate { file, length: 0 }).expect("reset sources");
    machine.call(parent, Op::Write { file, at: 0, bytes: sources.into() }).expect("sources written");
    machine.call(parent, Op::Close { handle: file }).expect("close sources");
    let (child, [stdout, stderr]) = spawn_mmdebstrap(&mut machine, parent, &args_for(names, proxy));
    let mut process =
        crate::process::Mmdebstrap::start(&mut machine, child, crate::process::Faults { stall_after: None });
    while process.step(&mut machine) == crate::process::Progress::Working {}
    let printed = drain_both(&mut machine, parent, stdout, stderr);
    Run { machine, parent, child, printed }
}
pub fn run(names: &[&[u8]]) -> Run {
    run_on(on_machine(Some(PORT)), names, Some(PORT), SOURCES)
}
pub const PACKAGES: &[u8] = b"/scratch/root/var/lib/apt/lists/snapshot.debian.org_archive_debian_20261001T030405Z_dists_forky_main_binary-amd64_Packages";

/// Invented stanzas in the kept form, with the pin's relation shapes.
pub const DEBIAN: &[u8] = b"Package: base-files
Version: 14.2
Essential: yes
Architecture: amd64
Pre-Depends: awk
Section: admin
Priority: required
Filename: pool/base-files.deb
Size: 1
SHA256: 0000000000000000000000000000000000000000000000000000000000000000

Package: base-passwd
Version: 3.6.7
Essential: yes
Architecture: amd64
Section: admin
Priority: required
Filename: pool/base-passwd.deb
Size: 1
SHA256: 0000000000000000000000000000000000000000000000000000000000000000

Package: dpkg
Version: 1.22.21
Essential: yes
Architecture: amd64
Pre-Depends: libc6 (>= 2.38)
Section: admin
Priority: required
Filename: pool/dpkg.deb
Size: 1
SHA256: 0000000000000000000000000000000000000000000000000000000000000000

Package: libc6
Version: 2.41-12
Architecture: amd64
Section: admin
Priority: required
Filename: pool/libc6.deb
Size: 1
SHA256: 0000000000000000000000000000000000000000000000000000000000000000

Package: mawk
Version: 1.3.4.20260302-2
Architecture: amd64
Provides: awk
Section: admin
Priority: required
Filename: pool/mawk.deb
Size: 1
SHA256: 0000000000000000000000000000000000000000000000000000000000000000

Package: gawk
Version: 1:5.3.2-1
Architecture: amd64
Provides: awk
Section: admin
Priority: standard
Filename: pool/gawk.deb
Size: 1
SHA256: 0000000000000000000000000000000000000000000000000000000000000000

Package: debconf
Version: 1.5.91
Architecture: amd64
Provides: debconf-2.0
Section: admin
Priority: required
Filename: pool/debconf.deb
Size: 1
SHA256: 0000000000000000000000000000000000000000000000000000000000000000

Package: adduser
Version: 3.152
Architecture: amd64
Section: admin
Priority: important
Filename: pool/adduser.deb
Size: 1
SHA256: 0000000000000000000000000000000000000000000000000000000000000000

Package: apt
Version: 3.1.8
Architecture: amd64
Depends: base-passwd (>= 3.6.1) | adduser, debconf | debconf-2.0
Section: admin
Priority: optional
Filename: pool/apt.deb
Size: 1
SHA256: 0000000000000000000000000000000000000000000000000000000000000000

Package: exim4-daemon-light
Version: 4.98.2-1
Architecture: amd64
Provides: mail-transport-agent
Conflicts: mail-transport-agent
Section: admin
Priority: optional
Filename: pool/exim4-daemon-light.deb
Size: 1
SHA256: 0000000000000000000000000000000000000000000000000000000000000000

Package: postfix
Version: 3.10.4-1
Architecture: amd64
Provides: mail-transport-agent
Conflicts: mail-transport-agent
Section: admin
Priority: optional
Filename: pool/postfix.deb
Size: 1
SHA256: 0000000000000000000000000000000000000000000000000000000000000000

Package: linux-image-amd64
Version: 6.16.12-1
Architecture: amd64
Depends: dracut (>= 110) | initramfs-tools (>= 0.120+deb8u2) | linux-initramfs-tool
Section: admin
Priority: optional
Filename: pool/linux-image-amd64.deb
Size: 1
SHA256: 0000000000000000000000000000000000000000000000000000000000000000

Package: dracut
Version: 059-4
Architecture: amd64
Section: admin
Priority: optional
Filename: pool/dracut.deb
Size: 1
SHA256: 0000000000000000000000000000000000000000000000000000000000000000

Package: initramfs-tools
Version: 0.150
Architecture: amd64
Section: admin
Priority: optional
Filename: pool/initramfs-tools.deb
Size: 1
SHA256: 0000000000000000000000000000000000000000000000000000000000000000

Package: tiny-initramfs
Version: 0.1-8
Architecture: amd64
Provides: linux-initramfs-tool
Conflicts: dracut, initramfs-tools
Section: admin
Priority: optional
Filename: pool/tiny-initramfs.deb
Size: 1
SHA256: 0000000000000000000000000000000000000000000000000000000000000000

";
pub fn debian() -> Universe {
    Universe::read(DEBIAN).expect("invented kept stanzas read")
}
pub fn names(universes: &[&Universe], selected: &[Selected]) -> Vec<String> {
    let mut names: Vec<_> = selected
        .iter()
        .map(|s| {
            let package = &universes[usize::try_from(s.universe).expect("source fits")].packages()
                [usize::try_from(s.package).expect("place fits")];
            String::from_utf8(package.name.to_vec()).expect("fixture names are ASCII")
        })
        .collect();
    names.sort();
    names
}
pub fn includes(names: &[&[u8]]) -> Box<[Box<[u8]>]> {
    names.iter().copied().map(Box::from).collect()
}
pub fn missing_groups(count: u32) -> Universe {
    let mut package = debian().package(b"base-files").expect("fixture package").clone();
    package.pre_depends = Box::default();
    package.depends = (0..count)
        .map(|i| {
            Box::from([Alternative {
                name: format!("missing-{i}").into_bytes().into_boxed_slice(),
                qualifier: None,
                version: None,
            }])
        })
        .collect();
    Universe::new(Box::new([package]))
}
pub fn dependency_on_awk(version: Option<&[u8]>) -> Universe {
    use hearth_fake_machine::hearth::universe::Relation;
    let universe = debian();
    let mut base = universe.package(b"base-files").expect("fixture base").clone();
    base.pre_depends = Box::new([Box::new([Alternative {
        name: Box::from(&b"awk"[..]),
        qualifier: None,
        version: Some((Relation::LaterOrEqual, Box::from(&b"2"[..]))),
    }])]);
    let mut mawk = universe.package(b"mawk").expect("fixture provider").clone();
    for provide in &mut mawk.provides {
        provide.version = version.map(Box::from);
    }
    Universe::new(Box::new([base, mawk]))
}
pub fn conflict_with_initramfs() -> Universe {
    let universe = debian();
    let mut packages = universe.packages().to_vec();
    let dracut = packages.iter_mut().find(|p| p.name.as_ref() == b"dracut").expect("fixture dracut");
    dracut.version = Box::from(&b"110"[..]);
    dracut.conflicts = Box::new([Alternative { name: Box::from(&b"postfix"[..]), qualifier: None, version: None }]);
    Universe::new(packages.into_boxed_slice())
}
