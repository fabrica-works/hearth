//! Seeded package universes with a Debian-shaped core (testing.md, 3.6).
use super::universe::{Alternative, Package, Priority, Provide, Relation, Universe};
use alloc::{boxed::Box, format, vec::Vec};
use skein_lib::{Rng, sha256};
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Draw<'a> {
    pub packages: u32,
    pub names: &'a [&'a [u8]],
}
impl Universe {
    #[must_use]
    pub fn draw(rng: &mut Rng, draw: &Draw<'_>) -> Self {
        let mut packages = Vec::new();
        for (name, version, priority, essential) in CORE {
            let mut p = package(name, version, *priority, rng);
            p.essential = *essential;
            match *name {
                b"base-files" => p.pre_depends = Box::new([Box::from([alternative(b"awk", None)])]),
                b"mawk" | b"gawk" => {
                    p.provides = Box::new([Provide { name: b"awk".as_slice().into(), version: None }]);
                    p.depends = Box::new([Box::from([alternative(b"libc6", Some(b"2.41"))])]);
                }
                b"dpkg" => p.depends = Box::new([Box::from([alternative(b"libc6", Some(b"2.41"))])]),
                b"debconf" => {
                    p.provides = Box::new([Provide { name: b"debconf-2.0".as_slice().into(), version: None }]);
                }
                b"apt" => {
                    p.depends = Box::new([
                        Box::from([alternative(b"base-passwd", Some(b"3.6.1")), alternative(b"adduser", None)]),
                        Box::from([alternative(b"debconf", None), alternative(b"debconf-2.0", None)]),
                        Box::from([alternative(b"libc6", Some(b"2.41"))]),
                    ]);
                }
                b"base-passwd" | b"libc6" | b"adduser" => {}
                _ => unreachable!("the fixed core's names"),
            }
            packages.push(p);
        }
        for k in 0..draw.packages {
            let name = format!("pkg-{k}");
            let version = format!("{}.0-1", rng.between(1, 20));
            let mut p = package(name.as_bytes(), version.as_bytes(), Priority::Optional, rng);
            let mut groups = Vec::new();
            for _ in 0..rng.between(1, 3) {
                let prior = earlier(rng, &packages);
                let mut group = vec![alternative(&prior.name, Some(&prior.version))];
                if rng.chance(200) {
                    let other = earlier(rng, &packages);
                    group.push(alternative(&other.name, Some(&other.version)));
                }
                groups.push(group.into_boxed_slice());
            }
            p.depends = groups.into_boxed_slice();
            if rng.chance(250) {
                let prior = earlier(rng, &packages);
                p.pre_depends = Box::new([Box::from([alternative(&prior.name, Some(&prior.version))])]);
            }
            if rng.below(6) == 0 {
                p.provides =
                    Box::new([Provide { name: format!("virtual-{k}").into_bytes().into_boxed_slice(), version: None }]);
            }
            if rng.chance(100) {
                p.architecture = b"all".as_slice().into();
            }
            packages.push(p);
        }
        for name in draw.names {
            packages.push(package(name, b"1.0-1", Priority::Optional, rng));
        }
        Self::new(packages.into_boxed_slice())
    }
}
const CORE: &[(&[u8], &[u8], Priority, bool)] = &[
    (b"base-files", b"14", Priority::Required, true),
    (b"base-passwd", b"3.6.8", Priority::Required, true),
    (b"dpkg", b"1.22.0", Priority::Required, true),
    (b"libc6", b"2.41-12", Priority::Required, false),
    (b"mawk", b"1.3.4", Priority::Required, false),
    (b"gawk", b"1:5.3.2-1", Priority::Optional, false),
    (b"debconf", b"1.5.91", Priority::Required, false),
    (b"adduser", b"3.153", Priority::Important, false),
    (b"apt", b"3.1.8", Priority::Required, false),
];
fn package(name: &[u8], version: &[u8], priority: Priority, rng: &mut Rng) -> Package {
    Package {
        name: name.into(),
        version: version.into(),
        architecture: b"amd64".as_slice().into(),
        priority,
        essential: false,
        section: b"utils".as_slice().into(),
        installed_size: Some(rng.between(1, 4096)),
        provides: Box::default(),
        depends: Box::default(),
        pre_depends: Box::default(),
        conflicts: Box::default(),
        filename: [b"pool/main/".as_slice(), name, b".deb"].concat().into_boxed_slice(),
        size: rng.between(1, 65536),
        sha256: sha256::digest(&[name, version].concat()),
    }
}
fn alternative(name: &[u8], version: Option<&[u8]>) -> Alternative {
    Alternative { name: name.into(), qualifier: None, version: version.map(|v| (Relation::LaterOrEqual, v.into())) }
}
fn earlier<'a>(rng: &mut Rng, packages: &'a [Package]) -> &'a Package {
    let place = rng.below(u64::try_from(packages.len()).expect("universe length fits u64"));
    packages.get(usize::try_from(place).expect("place fits usize")).expect("a prior package")
}
