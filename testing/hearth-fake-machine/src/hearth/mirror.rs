//! Snapshot suites and their unsigned indices (testing.md, 3.6).
use super::universe::Universe;
use alloc::{boxed::Box, collections::BTreeMap, format, string::String, vec::Vec};
use core::fmt::Write;
use skein_lib::{Rng, sha256};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Timestamp([u8; 16]);
impl Timestamp {
    #[must_use]
    pub fn new(bytes: &[u8]) -> Option<Self> {
        let bytes: [u8; 16] = bytes.try_into().ok()?;
        for (place, byte) in bytes.iter().enumerate() {
            if match place {
                8 => *byte != b'T',
                15 => *byte != b'Z',
                _ => !byte.is_ascii_digit(),
            } {
                return None;
            }
        }
        Some(Self(bytes))
    }
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Release {
    pub origin: Box<[u8]>,
    pub label: Box<[u8]>,
    pub suite: Box<[u8]>,
    pub codename: Box<[u8]>,
    pub version: Option<Box<[u8]>>,
}
impl Release {
    #[must_use]
    pub fn debian(suite: &[u8], codename: &[u8]) -> Self {
        Self {
            origin: b"Debian".as_slice().into(),
            label: b"Debian".as_slice().into(),
            suite: suite.into(),
            codename: codename.into(),
            version: None,
        }
    }
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Suite {
    release: Release,
    architecture: Box<[u8]>,
    components: Box<[(Box<[u8]>, Universe)]>,
    indices: Box<[Box<[u8]>]>,
    in_release: Box<[u8]>,
}
impl Suite {
    #[must_use]
    pub fn new(release: Release, architecture: &[u8], components: Box<[(Box<[u8]>, Universe)]>) -> Self {
        let indices: Box<[Box<[u8]>]> = components.iter().map(|(_, u)| u.index()).collect();
        let mut out = Vec::new();
        for (name, value) in [
            ("Origin", &release.origin),
            ("Label", &release.label),
            ("Suite", &release.suite),
            ("Codename", &release.codename),
        ] {
            field(&mut out, name, value);
        }
        if let Some(value) = &release.version {
            field(&mut out, "Version", value);
        }
        field(&mut out, "Architectures", architecture);
        let mut names = Vec::new();
        for (place, (name, _)) in components.iter().enumerate() {
            if place != 0 {
                names.push(b' ');
            }
            names.extend_from_slice(name);
        }
        field(&mut out, "Components", &names);
        out.extend_from_slice(b"SHA256:\n");
        for ((component, _), index) in components.iter().zip(&indices) {
            let mut hex = String::new();
            for byte in sha256::digest(index) {
                write!(&mut hex, "{byte:02x}").expect("writing to a string");
            }
            out.extend_from_slice(format!(" {hex} {} ", index.len()).as_bytes());
            out.extend_from_slice(component);
            out.extend_from_slice(b"/binary-");
            out.extend_from_slice(architecture);
            out.extend_from_slice(b"/Packages\n");
        }
        Self { release, architecture: architecture.into(), components, indices, in_release: out.into_boxed_slice() }
    }
    #[must_use]
    pub const fn release(&self) -> &Release {
        &self.release
    }
    #[must_use]
    pub fn universe(&self, component: &[u8]) -> Option<&Universe> {
        self.components.iter().find(|(name, _)| &**name == component).map(|(_, u)| u)
    }
    #[must_use]
    pub fn index(&self, component: &[u8]) -> Option<&[u8]> {
        self.components
            .iter()
            .position(|(name, _)| &**name == component)
            .and_then(|place| self.indices.get(place))
            .map(AsRef::as_ref)
    }
    #[must_use]
    pub fn in_release(&self) -> &[u8] {
        &self.in_release
    }
}
fn field(out: &mut Vec<u8>, name: &str, value: &[u8]) {
    out.extend_from_slice(name.as_bytes());
    out.extend_from_slice(b": ");
    out.extend_from_slice(value);
    out.push(b'\n');
}
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Mirror {
    snapshots: BTreeMap<Timestamp, BTreeMap<Box<[u8]>, Suite>>,
}
impl Mirror {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add(&mut self, at: Timestamp, suite: Suite) {
        self.snapshots.entry(at).or_default().insert(suite.release.codename.clone(), suite);
    }
    pub fn bump(&mut self, rng: &mut Rng, at: Timestamp, per_mille: u32) {
        let Some((latest, suites)) = self.snapshots.last_key_value() else {
            return;
        };
        assert!(at > *latest, "a bump is later than the latest snapshot");
        let mut bumped = BTreeMap::new();
        for (name, suite) in suites {
            let mut components = suite.components.clone();
            for (_, universe) in &mut components {
                let mut packages = universe.packages().to_vec();
                for package in &mut packages {
                    if rng.chance(per_mille) {
                        package.version = [package.version.as_ref(), b"+b1"].concat().into_boxed_slice();
                        package.size =
                            package.size.checked_add(rng.between(1, 4096)).expect("drawn archive size fits u64");
                        package.sha256 = sha256::digest(&[package.name.as_ref(), package.version.as_ref()].concat());
                    }
                }
                *universe = Universe::new(packages.into_boxed_slice());
            }
            bumped.insert(name.clone(), Suite::new(suite.release.clone(), &suite.architecture, components));
        }
        self.snapshots.insert(at, bumped);
    }
    #[must_use]
    pub fn suite(&self, archive: &[u8], at: Timestamp, codename: &[u8]) -> Option<&Suite> {
        if archive != b"debian" {
            return None;
        }
        self.snapshots.range(..=at).next_back()?.1.get(codename)
    }
}
