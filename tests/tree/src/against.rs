//! A walk against a parent, checked against the oracle: the child's tree digest and change set
//! meet the oracle's over the two trees, whichever entries the walk read (domain/store.md, 3.5,
//! 4.3 and 5.4).
use crate::{
    check, expected,
    fs::Fs,
    helper::{self, Helper, Walked},
    oracle,
};
use hearth_domain_tree::{
    limits::Limits,
    vocabulary::{Answer, Counts, Request},
};
use hearth_lib::encode::Digest;
use hearth_protocol_tree::{
    limits as protocol,
    tree::{self, Entry, Item, Local, Op},
};
use skein_lib::sha256;
use std::collections::BTreeMap;

pub const PARENT_KEY: Digest = Digest([0x41; 32]);
pub const KEY: Digest = Digest([0x42; 32]);

/// The tree file of a walk of `fs` with no parent.
#[must_use]
pub fn parent_file(fs: &Fs, limits: &Limits) -> Walked {
    let walked = helper::walk(fs, limits, PARENT_KEY);
    assert!(matches!(walked.answer, Answer::Walked { .. }), "the parent walks: {:?}", walked.answer);
    walked
}
/// Walks `child` against the file.
#[must_use]
pub fn against(child: &Fs, file: &[u8], limits: &Limits) -> Walked {
    helper::walk_with(Helper::new(child).against(file), limits, KEY)
}

/// Applying the change items to the parent's entries gives the child's.
#[must_use]
pub fn applied(parent: &[(Entry, Local)], items: &[Item]) -> Vec<Entry> {
    let mut entries: BTreeMap<Vec<u8>, Entry> =
        parent.iter().map(|(entry, _)| (entry.path.to_vec(), entry.clone())).collect();
    let mut last: Option<&Entry> = None;
    for item in items {
        match item {
            Item::Entry(entry, _) => last = Some(entry),
            Item::Change { op: Op::Put, path } => {
                let entry = last.expect("a put follows its entry");
                assert_eq!(&*entry.path, &**path, "a put follows its own entry");
                entries.insert(path.to_vec(), entry.clone());
            }
            Item::Change { op: Op::Delete, path } => {
                assert!(entries.remove(&path.to_vec()).is_some(), "a delete is of a path the parent had: {path:?}");
            }
            Item::End(_) => {}
        }
    }
    let mut all: Vec<Entry> = entries.into_values().collect();
    all.sort_by(|a, b| oracle::order(&a.path, &b.path));
    all
}

/// Walks `child` against the walk of `parent`, and checks what came against the oracle: the
/// digests, the counts, the file, its digest, and the change set applied to the parent.
pub fn checked(parent: &Fs, child: &Fs, limits: &Limits, what: &str) -> Walked {
    let first = parent_file(parent, limits);
    let walked = against(child, &first.file, limits);
    let Answer::Walked { tree, changes, tree_file, counts } = &walked.answer else {
        let (path, cause) = walked.failed();
        panic!("{what}: the walk failed at a path of {} bytes: {cause:?}", path.len())
    };
    let items = expected::items_against(child, parent, KEY);
    let Some(Item::End(end)) = items.last() else { panic!("the end item is last") };
    assert_eq!(*tree, end.tree, "{what}: the tree digest is the oracle's over the child");
    assert_eq!(*changes, end.changes, "{what}: the change set's digest is the oracle's over the two trees");
    assert_eq!(helper::tree_counts(counts), end.counts, "{what}: the counts");
    assert!(oracle::items_ok(&items), "{what}: the oracle lists a tree file");
    let read = check::read_tree(&walked.file, &protocol::SHIPPED).unwrap_or_else(|error| panic!("{what}: {error:?}"));
    assert_eq!(read.as_ref(), items.as_slice(), "{what}: the file reads back to what the oracle lists");
    let size = u64::try_from(walked.file.len()).expect("a size");
    assert_eq!(tree_file.size, size, "{what}: the file's size");
    assert_eq!(tree_file.sha256, Digest(sha256::digest(&walked.file)), "{what}: the file's SHA-256");
    let body: Vec<u8> = items.iter().flat_map(oracle::item_bytes).collect();
    let mut file = tree::header(u64::try_from(body.len()).expect("a size")).to_vec();
    file.extend_from_slice(&body);
    assert_eq!(walked.file, file, "{what}: the file's bytes");
    let (before, _) = expected::entries(parent);
    let (after, _) = expected::entries(child);
    let gives = applied(&before, &read);
    let wanted: Vec<Entry> = after.into_iter().map(|(entry, _)| entry).collect();
    assert_eq!(gives, wanted, "{what}: the change items applied to the parent's entries give the child's");
    // The counts the design gives (6.2): the tree's, the change set's, the bytes read.
    assert_eq!(
        end.counts.puts + end.counts.deletes,
        changes_in(&read),
        "{what}: puts and deletes are the change items"
    );
    walked
}
#[must_use]
pub fn changes_in(items: &[Item]) -> u64 {
    items.iter().filter(|item| matches!(item, Item::Change { .. })).count() as u64
}
#[must_use]
pub fn counted(walked: &Walked) -> Counts {
    match &walked.answer {
        Answer::Walked { counts, .. } => *counts,
        Answer::Failed { .. } => panic!("the walk failed: {:?}", walked.answer),
    }
}
#[must_use]
pub fn hashes(walked: &Walked) -> usize {
    walked.requests.iter().filter(|request| matches!(request, Request::HashFile { .. })).count()
}
