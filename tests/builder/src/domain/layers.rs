//! The layers of the world's store, and the tree helper's fake over them (domain/store.md, 5.3 and
//! 11): a layer is a directory beneath `layers/`, a snapshot is the fake's copy of one, and a freeze
//! is the sync. The fake's digests are stand-ins for the walk's (h05): deterministic from the files.
use crate::{
    files::Files,
    fixtures::{WORLD_OWNER, hex_digest},
};
use hearth_domain_builder::build::Step;
use hearth_domain_store::vocabulary::{Cause, Counts, Event, Helped, Helper, Record, Renaming, TreeFile, Walk};
use hearth_lib::encode::Digest;
use hearth_protocol_builder::translate;
use hearth_protocol_tree::{self as tree, limits::SHIPPED};
use skein_io::{
    file,
    kernel::{Error, Kind, OpenHow},
};
use skein_sim::{Ask, Handle};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Default)]
pub(super) struct Layers {
    seed: u64,
    /// The layer the last snapshot made, whose steps the world runs.
    staging: Option<Digest>,
    /// The helper requests answered so far.
    helped: usize,
    /// The helper request, counted from zero, that fails after doing its work.
    failing: Option<usize>,
}
impl Layers {
    pub fn new(seed: u64) -> Self {
        Self { seed, staging: None, helped: 0, failing: None }
    }
    /// The `nth` helper request, counted from zero, fails: it does its work, then answers `Failed`.
    pub fn fail_helper(&mut self, nth: usize) {
        self.failing = Some(nth);
    }
    /// The empty layer's subvolume, with the uuid the builder reads after making it (4.1).
    pub fn create(&self, files: &Files, key: Digest) -> Event {
        let made = ensure(files).and_then(|()| make_directory(files, &layer(key, ".staging")));
        match made {
            Ok(()) => Event::Created { uuid: self.uuid(key) },
            Err(_) => Event::Failed,
        }
    }
    pub fn rename(files: &Files, key: Digest, rename: Renaming) -> Event {
        let (from, to) = match rename {
            Renaming::Tree => (layer(key, ".tree.tmp"), layer(key, ".tree")),
            Renaming::Publish => (layer(key, ".staging"), layer(key, "")),
        };
        done(rename_within(files, &from, &to))
    }
    pub fn write(files: &mut Files, record: &Record) -> Event {
        let bytes = tree::layer::write(&translate::store::tree_layer(record), &SHIPPED).expect("a record in bounds");
        let stored = files.answer(file::Request::Store {
            owner: WORLD_OWNER,
            root: files.roots.store.token,
            path: layer(record.layer.key, ".record").into_bytes().into(),
            bytes,
            expected: file::Expect::Any,
            no_follow: true,
        });
        if matches!(stored, file::Event::Stored { .. }) { Event::Done } else { Event::Failed }
    }
    /// A staging layer's directory, once emptied (7.3, step 5).
    pub fn remove_staging(files: &Files, key: Digest) -> Event {
        let removed = within(files, &layer(key, ".staging"), |directory, name| remove(files, directory, name, true));
        done(removed)
    }
    /// `<key>.tree.tmp`, if the walk wrote one.
    pub fn remove_tree_tmp(files: &Files, key: Digest) -> Event {
        let removed = within(files, &layer(key, ".tree.tmp"), |directory, name| remove(files, directory, name, false));
        done(match removed {
            Err(Error::NotFound) => Ok(()),
            Ok(()) | Err(_) => removed,
        })
    }
    pub fn helper(&mut self, files: &Files, helper: Helper) -> Event {
        let nth = self.helped;
        self.helped = nth.saturating_add(1);
        let answer = match helper {
            Helper::Snapshot { parent, staging } => {
                self.staging = Some(staging);
                self.snapshot(files, parent, staging)
            }
            Helper::Walk { layer: key, parent } => walk(files, key, parent).map(Helped::Walked),
            Helper::Hash { layer: key } => walk(files, key, None).map(Helped::Hashed),
            // A freeze is the sync, and the fake keeps no read-only flag (btrfs comes with h08).
            Helper::Freeze { .. } => files.call(Ask::Sync { file: files.store_root() }).map(|_| Helped::Frozen),
            Helper::Empty { layer: key } => {
                empty(files, &layer(key, ".staging")).map(|removed| Helped::Emptied { removed })
            }
            Helper::Thaw { .. } | Helper::Apply { .. } | Helper::Delete { .. } => {
                unreachable!("no task sends {helper:?} yet")
            }
        };
        let answer = answer.unwrap_or(Helped::Failed { cause: Cause::NotFound });
        Event::Helped {
            answer: if self.failing == Some(nth) { Helped::Failed { cause: Cause::NoSpace } } else { answer },
        }
    }
    fn snapshot(&self, files: &Files, parent: Digest, staging: Digest) -> Result<Helped, Error> {
        let record = read(files, &layer(parent, ".record"))?;
        let parent_uuid = tree::layer::read(&record, &SHIPPED).map_err(|_| Error::NotFound)?.subvolume.uuid;
        copy(files, &layer(parent, ""), &layer(staging, ".staging"))?;
        Ok(Helped::Snapshotted { uuid: self.uuid(staging), parent_uuid })
    }
    /// The seeded files a step leaves in the layer being made: a few small ones, in a directory
    /// of the step's name.
    pub fn run(&self, files: &Files, step: Step) {
        let staging = self.staging.expect("a step runs in a staged layer");
        let name: String =
            format!("{step:?}").chars().filter(char::is_ascii_alphanumeric).collect::<String>().to_lowercase();
        let base = format!("{}/{name}", layer(staging, ".staging"));
        make_directory(files, &base).expect("the step's directory");
        for index in 0..3_u8 {
            let seeded = [&self.seed.to_le_bytes()[..], name.as_bytes(), &[index]].concat();
            let content = hex_digest(skein_lib::sha256::digest(&seeded));
            write(files, &format!("{base}/file{index}"), content.as_bytes()).expect("the step's file");
        }
    }
    /// The uuid the fake helper gives a layer's subvolume: drawn from the seed and the key.
    fn uuid(&self, key: Digest) -> [u8; 16] {
        let drawn = skein_lib::sha256::digest(&[&self.seed.to_le_bytes()[..], &key.0].concat());
        drawn[..16].try_into().expect("sixteen of thirty-two bytes")
    }
}
fn remove(files: &Files, directory: Handle, name: &str, is_directory: bool) -> Result<(), Error> {
    files.call(Ask::Remove { dir: directory, name: name.as_bytes().into(), directory: is_directory }).map(|_| ())
}
/// Removes everything beneath the directory at `path`, deepest first; how many entries went.
fn empty(files: &Files, path: &str) -> Result<u64, Error> {
    let mut removed = 0_u64;
    for (name, kind) in entries(files, path)? {
        let child = format!("{path}/{name}");
        let is_directory = kind == Kind::Directory;
        if is_directory {
            removed = removed.saturating_add(empty(files, &child)?);
        }
        within(files, &child, |directory, entry| remove(files, directory, entry, is_directory))?;
        removed = removed.saturating_add(1);
    }
    Ok(removed)
}
fn done(result: Result<(), Error>) -> Event {
    if result.is_ok() { Event::Done } else { Event::Failed }
}
/// `layers/<key><suffix>`, beneath the store's root.
fn layer(key: Digest, suffix: &str) -> String {
    format!("layers/{}{suffix}", hex_digest(key.0))
}
/// `layers/` is made with the first layer (domain/store.md, 2.4).
fn ensure(files: &Files) -> Result<(), Error> {
    match files.call(Ask::MakeDirectory { dir: files.store_root(), name: b"layers".as_slice().into(), mode: 0o700 }) {
        Ok(_) | Err(Error::Exists) => Ok(()),
        Err(error) => Err(error),
    }
}
/// Runs `act` on the directory a path beneath the store's root is in, with the path's last name.
fn within<T>(files: &Files, path: &str, act: impl FnOnce(Handle, &str) -> Result<T, Error>) -> Result<T, Error> {
    let (directory, name) = path.rsplit_once('/').unwrap_or((".", path));
    let handle = files.open(files.store_root(), directory.as_bytes(), OpenHow::Directory)?;
    let result = act(handle, name);
    files.close(handle);
    result
}
fn make_directory(files: &Files, path: &str) -> Result<(), Error> {
    within(files, path, |directory, name| {
        files.call(Ask::MakeDirectory { dir: directory, name: name.as_bytes().into(), mode: 0o700 }).map(|_| ())
    })
}
fn rename_within(files: &Files, from: &str, to: &str) -> Result<(), Error> {
    let (directory, from) = from.rsplit_once('/').expect("a path in layers/");
    let (_, to) = to.rsplit_once('/').expect("a path in layers/");
    let handle = files.open(files.store_root(), directory.as_bytes(), OpenHow::Directory)?;
    let result = files
        .call(Ask::Rename { from_dir: handle, from: from.as_bytes().into(), to_dir: handle, to: to.as_bytes().into() })
        .and_then(|_| files.call(Ask::Sync { file: handle }));
    files.close(handle);
    result.map(|_| ())
}
fn write(files: &Files, path: &str, bytes: &[u8]) -> Result<(), Error> {
    let file = files.open(files.store_root(), path.as_bytes(), OpenHow::Create { mode: Some(0o644) })?;
    let result =
        files.call(Ask::Write { file, at: 0, bytes: bytes.into() }).and_then(|_| files.call(Ask::Sync { file }));
    files.close(file);
    result.map(|_| ())
}
fn read(files: &Files, path: &str) -> Result<Box<[u8]>, Error> {
    match files.load(WORLD_OWNER, files.store_root(), path.as_bytes(), u32::MAX, false)? {
        file::Event::Loaded { bytes, .. } => Ok(bytes),
        file::Event::TooLarge { .. }
        | file::Event::Failed { .. }
        | file::Event::Cancelled { .. }
        | file::Event::Opened { .. }
        | file::Event::Made { .. }
        | file::Event::Refused { .. }
        | file::Event::Conflict { .. }
        | file::Event::Scanned { .. }
        | file::Event::Stated { .. }
        | file::Event::Read { .. }
        | file::Event::Written { .. }
        | file::Event::Cut { .. }
        | file::Event::Synced { .. }
        | file::Event::Renamed { .. }
        | file::Event::Listed { .. }
        | file::Event::Removed { .. }
        | file::Event::Closed { .. }
        | file::Event::Locked { .. }
        | file::Event::Busy { .. }
        | file::Event::Stored { .. } => unreachable!("Load loads"),
    }
}
/// The entries of a directory beneath the store's root, by name.
fn entries(files: &Files, path: &str) -> Result<BTreeMap<String, Kind>, Error> {
    let handle = files.open(files.store_root(), path.as_bytes(), OpenHow::Directory)?;
    let listed = files.list(handle, 4096);
    files.close(handle);
    Ok(listed?
        .iter()
        .map(|entry| (String::from_utf8(entry.name.to_vec()).expect("world names are text"), entry.kind))
        .collect())
}
/// The fake's snapshot: a copy of the directory `from` as `to`.
fn copy(files: &Files, from: &str, to: &str) -> Result<(), Error> {
    make_directory(files, to)?;
    for (name, kind) in entries(files, from)? {
        let (from, to) = (format!("{from}/{name}"), format!("{to}/{name}"));
        match kind {
            Kind::Directory => copy(files, &from, &to)?,
            Kind::File => write(files, &to, &read(files, &from)?)?,
            Kind::Symlink | Kind::Other => unreachable!("the world's layers hold directories and files"),
        }
    }
    Ok(())
}
/// One entry of the fake's tree file: its line, and a file's size.
struct Line {
    text: String,
    size: Option<u64>,
}
/// The tree of the directory `from`, by path.
fn survey(files: &Files, from: &str, path: &str, into: &mut BTreeMap<String, Line>) -> Result<(), Error> {
    for (name, kind) in entries(files, from)? {
        let (from, path) = (format!("{from}/{name}"), format!("{path}/{name}"));
        match kind {
            Kind::Directory => {
                into.insert(path.clone(), Line { text: format!("d {path}\n"), size: None });
                survey(files, &from, &path, into)?;
            }
            Kind::File => {
                let bytes = read(files, &from)?;
                let sha = hex_digest(skein_lib::sha256::digest(&bytes));
                into.insert(
                    path.clone(),
                    Line { text: format!("f {path} {} {sha}\n", bytes.len()), size: Some(bytes.len() as u64) },
                );
            }
            Kind::Symlink | Kind::Other => unreachable!("the world's layers hold directories and files"),
        }
    }
    Ok(())
}
/// The walk or the hash of a layer being made, against its parent's tree file: writes
/// `<key>.tree.tmp`, the listing of the tree, and gives the digests and counts of what it found.
fn walk(files: &Files, key: Digest, parent: Option<Digest>) -> Result<Walk, Error> {
    let mut tree = BTreeMap::from([(".".to_owned(), Line { text: "d .\n".to_owned(), size: None })]);
    survey(files, &layer(key, ".staging"), ".", &mut tree)?;
    let before: BTreeMap<String, String> = match parent {
        Some(parent) => {
            let listing = String::from_utf8(read(files, &layer(parent, ".tree"))?.to_vec()).expect("a listing");
            listing
                .lines()
                .map(|line| {
                    let path = line.split(' ').nth(1).expect("a path").to_owned();
                    (path, format!("{line}\n"))
                })
                .collect()
        }
        None => BTreeMap::new(),
    };
    let listing: String = tree.values().map(|line| line.text.as_str()).collect();
    let puts: Vec<&Line> =
        tree.iter().filter(|(path, line)| before.get(*path) != Some(&line.text)).map(|(_, line)| line).collect();
    let present: BTreeSet<&String> = tree.keys().collect();
    let deletes: Vec<&String> = before.keys().filter(|path| !present.contains(path)).collect();
    let changes: String =
        puts.iter().map(|line| line.text.clone()).chain(deletes.iter().map(|path| format!("- {path}\n"))).collect();
    let sizes = |lines: &mut dyn Iterator<Item = &Line>| lines.filter_map(|line| line.size).sum::<u64>();
    let counts = Counts {
        bytes: sizes(&mut tree.values()),
        deletes: deletes.len() as u64,
        entries: tree.len() as u64,
        files: tree.values().filter(|line| line.size.is_some()).count() as u64,
        hashed: sizes(&mut puts.iter().copied()),
        puts: puts.len() as u64,
    };
    let bytes = listing.as_bytes();
    let stored = write(files, &layer(key, ".tree.tmp"), bytes);
    stored?;
    let digest = |tag: &str, text: &str| Digest(skein_lib::sha256::digest(format!("{tag}\n{text}").as_bytes()));
    Ok(Walk {
        tree: digest("tree", &listing),
        changes: digest("changes", &changes),
        tree_file: TreeFile { sha256: Digest(skein_lib::sha256::digest(bytes)), size: bytes.len() as u64 },
        counts,
    })
}
