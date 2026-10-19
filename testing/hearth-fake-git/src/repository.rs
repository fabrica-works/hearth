//! Bare repositories, resolution and recursive tree listings (domain/testing.md, 4.2).
use crate::objects::{Commit, Entry, Mode, Oid, Tag, Type, hash};
use alloc::{boxed::Box, collections::BTreeMap};
use hearth_fake_machine::linux::{Ids, Item, Machine};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Resolved {
    Commit(Oid),
    Missing,
    NotACommit { kind: Type },
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Listed {
    pub mode: Mode,
    pub oid: Oid,
    pub size: Option<u64>,
    pub path: Box<[u8]>,
}
#[derive(Debug)]
pub struct Repository {
    objects: BTreeMap<Oid, (Type, Box<[u8]>)>,
    refs: BTreeMap<Box<[u8]>, Oid>,
    listings: BTreeMap<Oid, Box<[Listed]>>,
    head: Box<[u8]>,
}
impl Repository {
    #[must_use]
    pub fn new(head: &[u8]) -> Self {
        Self { objects: BTreeMap::new(), refs: BTreeMap::new(), listings: BTreeMap::new(), head: head.into() }
    }
    fn store(&mut self, kind: Type, bytes: Vec<u8>) -> Oid {
        let oid = hash(kind, &bytes);
        self.objects.insert(oid, (kind, bytes.into()));
        oid
    }
    pub fn commit(&mut self, commit: &Commit) -> Oid {
        let listed: Vec<_> = commit
            .tree
            .0
            .iter()
            .map(|(path, entry)| {
                let (mode, oid, size) = match entry {
                    Entry::Blob { mode, bytes } => {
                        (*mode, self.store(Type::Blob, bytes.to_vec()), Some(bytes.len() as u64))
                    }
                    Entry::Submodule(oid) => (Mode::Submodule, *oid, None),
                };
                Listed { mode, oid, size, path: path.clone() }
            })
            .collect();
        let root = self.tree(&listed, b"");
        let mut bytes = b"tree ".to_vec();
        bytes.extend_from_slice(&root.hex());
        bytes.push(b'\n');
        for parent in &commit.parents {
            bytes.extend_from_slice(b"parent ");
            bytes.extend_from_slice(&parent.hex());
            bytes.push(b'\n');
        }
        commit.author.write("author", &mut bytes);
        commit.committer.write("committer", &mut bytes);
        bytes.push(b'\n');
        bytes.extend_from_slice(&commit.message);
        let oid = self.store(Type::Commit, bytes);
        self.listings.insert(oid, listed.into());
        self.set_ref(&self.head.clone(), oid);
        oid
    }
    fn tree(&mut self, listed: &[Listed], prefix: &[u8]) -> Oid {
        let mut entries = BTreeMap::<Vec<u8>, (bool, u32, Oid)>::new();
        for item in listed {
            let Some(rest) = item.path.strip_prefix(prefix) else {
                continue;
            };
            if let Some(slash) = rest.iter().position(|b| *b == b'/') {
                let name = &rest[..slash];
                if !entries.contains_key(name) {
                    let mut child = prefix.to_vec();
                    child.extend_from_slice(name);
                    child.push(b'/');
                    let oid = self.tree(listed, &child);
                    entries.insert(name.to_vec(), (true, 0o40000, oid));
                }
            } else {
                entries.insert(rest.to_vec(), (false, item.mode.number(), item.oid));
            }
        }
        let mut entries: Vec<_> = entries.into_iter().collect();
        entries.sort_by(|(a, (at, ..)), (b, (bt, ..))| {
            a.iter().copied().chain(at.then_some(b'/')).cmp(b.iter().copied().chain(bt.then_some(b'/')))
        });
        let mut bytes = Vec::new();
        for (name, (_, mode, oid)) in entries {
            bytes.extend_from_slice(format!("{mode:o} ").as_bytes());
            bytes.extend_from_slice(&name);
            bytes.push(0);
            bytes.extend_from_slice(&oid.0);
        }
        self.store(Type::Tree, bytes)
    }
    pub fn tag(&mut self, tag: &Tag) -> Oid {
        assert!(matches!(self.object(tag.commit), Some((Type::Commit, _))), "tag a stored commit");
        let mut bytes = b"object ".to_vec();
        bytes.extend_from_slice(&tag.commit.hex());
        bytes.extend_from_slice(b"\ntype commit\ntag ");
        bytes.extend_from_slice(&tag.name);
        bytes.push(b'\n');
        tag.tagger.write("tagger", &mut bytes);
        bytes.push(b'\n');
        bytes.extend_from_slice(&tag.message);
        let oid = self.store(Type::Tag, bytes);
        self.set_ref(&[b"refs/tags/", tag.name.as_ref()].concat(), oid);
        oid
    }
    pub fn set_ref(&mut self, name: &[u8], to: Oid) {
        self.refs.insert(name.into(), to);
    }
    #[must_use]
    pub fn resolve(&self, revision: &[u8]) -> Resolved {
        let revision = if revision == b"HEAD" { self.head.as_ref() } else { revision };
        let direct = Oid::from_hex(revision).filter(|oid| self.objects.contains_key(oid));
        let by_ref = [b"".as_slice(), b"refs/", b"refs/tags/", b"refs/heads/", b"refs/remotes/"]
            .into_iter()
            .find_map(|prefix| self.refs.get([prefix, revision].concat().as_slice()).copied())
            .or_else(|| self.refs.get([b"refs/remotes/", revision, b"/HEAD"].concat().as_slice()).copied());
        let mut oid = direct.or(by_ref);
        if oid.is_none() && (4..40).contains(&revision.len()) && revision.iter().all(u8::is_ascii_hexdigit) {
            let prefix: Vec<_> = revision.iter().map(u8::to_ascii_lowercase).collect();
            let mut found = self.objects.keys().filter(|oid| oid.hex().starts_with(&prefix));
            oid = found.next().copied();
            assert!(found.next().is_none(), "no ambiguous hex prefix in M1");
        }
        let Some(mut oid) = oid else {
            return Resolved::Missing;
        };
        for _ in 0..=self.objects.len() {
            let Some((kind, bytes)) = self.object(oid) else {
                return Resolved::Missing;
            };
            match kind {
                Type::Commit => return Resolved::Commit(oid),
                Type::Tag => oid = Oid::from_hex(&bytes[7..47]).expect("stored tag's object"),
                Type::Blob | Type::Tree => return Resolved::NotACommit { kind },
            }
        }
        unreachable!("stored tags have no cycle")
    }
    #[must_use]
    pub fn list(&self, commit: Oid) -> Option<Box<[Listed]>> {
        self.listings.get(&commit).cloned()
    }
    #[must_use]
    pub fn object(&self, oid: Oid) -> Option<(Type, &[u8])> {
        self.objects.get(&oid).map(|(kind, bytes)| (*kind, bytes.as_ref()))
    }
}
#[derive(Debug, Default)]
pub struct Repositories(pub BTreeMap<Box<[u8]>, Repository>);
impl Repositories {
    pub fn lay(&mut self, machine: &mut Machine, path: &[u8], owner: Ids, repository: Repository) {
        assert!(path.starts_with(b"/") && !path.ends_with(b"/"), "absolute git-dir");
        for (i, byte) in path.iter().enumerate().skip(1) {
            if *byte == b'/' && machine.facts(&path[..i]).is_none() {
                machine.lay(&[Item::directory(&path[..i], 0o755)]);
            }
        }
        let head = [b"ref: ", repository.head.as_ref(), b"\n"].concat();
        machine.lay(&[
            Item::directory(path, 0o755).owner(owner),
            Item::file(&[path, b"/HEAD"].concat(), 0o644, &head).owner(owner),
        ]);
        assert!(self.0.insert(path.into(), repository).is_none(), "one repository at a path");
    }
}
