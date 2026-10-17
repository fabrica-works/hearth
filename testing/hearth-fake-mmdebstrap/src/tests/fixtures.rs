use crate::resolve::Selected;
use hearth_fake_machine::hearth::universe::{Alternative, Universe};

/// Invented stanzas in the kept form, with the pin's relation shapes.
pub const DEBIAN: &[u8] = b"Package: base-files
Version: 14.2
Essential: yes
Pre-Depends: awk
Architecture: amd64
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
Pre-Depends: libc6 (>= 2.38)
Architecture: amd64
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
Provides: awk
Architecture: amd64
Section: admin
Priority: required
Filename: pool/mawk.deb
Size: 1
SHA256: 0000000000000000000000000000000000000000000000000000000000000000

Package: gawk
Version: 1:5.3.2-1
Provides: awk
Architecture: amd64
Section: admin
Priority: standard
Filename: pool/gawk.deb
Size: 1
SHA256: 0000000000000000000000000000000000000000000000000000000000000000

Package: debconf
Version: 1.5.91
Provides: debconf-2.0
Architecture: amd64
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
Depends: base-passwd (>= 3.6.1) | adduser, debconf | debconf-2.0
Architecture: amd64
Section: admin
Priority: optional
Filename: pool/apt.deb
Size: 1
SHA256: 0000000000000000000000000000000000000000000000000000000000000000

Package: exim4-daemon-light
Version: 4.98.2-1
Provides: mail-transport-agent
Conflicts: mail-transport-agent
Architecture: amd64
Section: admin
Priority: optional
Filename: pool/exim4-daemon-light.deb
Size: 1
SHA256: 0000000000000000000000000000000000000000000000000000000000000000

Package: postfix
Version: 3.10.4-1
Provides: mail-transport-agent
Conflicts: mail-transport-agent
Architecture: amd64
Section: admin
Priority: optional
Filename: pool/postfix.deb
Size: 1
SHA256: 0000000000000000000000000000000000000000000000000000000000000000

Package: linux-image-amd64
Version: 6.16.12-1
Depends: dracut (>= 110) | initramfs-tools (>= 0.120+deb8u2) | linux-initramfs-tool
Architecture: amd64
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
Provides: linux-initramfs-tool
Conflicts: dracut, initramfs-tools
Architecture: amd64
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
