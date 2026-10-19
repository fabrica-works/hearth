//! The walk against a parent: the child's tree digest and change set meet the oracle's over the
//! two trees, whichever entries the walk read (domain/store.md, 3.5, 4.3 and 5.4).
use hearth_domain_tree::{
    limits::{Limits, SHIPPED},
    vocabulary::{Answer, Cause, Fault, Kind, Request},
};
use hearth_lib::encode::Digest;
use hearth_protocol_tree::{
    limits as protocol,
    tree::{self, Item, Local},
};
use hearth_tree_world::{
    against::{KEY, against, checked, counted, hashes, parent_file},
    check, edits, expected,
    fs::{Fs, Node},
    helper::{self, Helper, Tamper, Walked},
    layers, oracle,
};
use skein_lib::Rng;

#[test]
fn a_walk_with_nothing_changed_is_no_change_with_the_parents_tree_digest_and_nothing_hashed() {
    for seed in 0..24 {
        let mut rng = Rng::new(0x7EE0_8100 + seed);
        let parent = layers::layer(&mut rng, seed * 5);
        let first = parent_file(&parent, &SHIPPED);
        let walked = checked(&parent, &parent.clone(), &SHIPPED, &format!("seed {seed}"));
        let (Answer::Walked { tree: before, .. }, Answer::Walked { tree, changes, counts, .. }) =
            (&first.answer, &walked.answer)
        else {
            panic!("walked")
        };
        assert_eq!(tree, before, "seed {seed}: the parent's tree digest");
        assert_eq!((counts.puts, counts.deletes, counts.hashed), (0, 0, 0), "seed {seed}");
        let empty = oracle::changes_digest(Some(*before), *before, &[]);
        assert_eq!(*changes, empty, "seed {seed}: the change set of no change");
        assert_eq!(hashes(&walked), 0, "seed {seed}: no file is hashed");
        let attributes = walked.requests.iter().filter(|request| matches!(request, Request::Attributes { .. })).count();
        assert_eq!(attributes, 0, "seed {seed}: no entry is read");
        // The tree file holds every entry, each with the local index the parent's had.
        let (found, _) = expected::entries(&parent);
        let read = check::read_tree(&walked.file, &protocol::SHIPPED).expect("a tree file");
        let locals: Vec<Local> =
            read.iter().filter_map(|item| if let Item::Entry(_, local) = item { Some(*local) } else { None }).collect();
        assert_eq!(locals, found.iter().map(|(_, local)| *local).collect::<Vec<_>>(), "seed {seed}");
    }
}

#[test]
fn the_empty_layer_against_itself_is_one_entry_and_no_change() {
    let walked = checked(&Fs::new(), &Fs::new(), &SHIPPED, "the empty layer");
    let counts = counted(&walked);
    assert_eq!((counts.entries, counts.puts, counts.deletes, counts.hashed), (1, 0, 0, 0));
}

#[test]
fn generated_edits_walk_to_the_oracles_digests_and_file() {
    let mut seen = std::collections::BTreeSet::new();
    for seed in 0..40 {
        let mut rng = Rng::new(0x7EE0_8200 + seed);
        let parent = layers::layer(&mut rng, 1 + seed % 40 * 3);
        let mut child = parent.clone();
        for edit in edits::edit(&mut rng, &mut child, 1, 6) {
            seen.insert(format!("{edit:?}"));
        }
        checked(&parent, &child, &SHIPPED, &format!("seed {seed}"));
    }
    for edit in edits::ALL {
        assert!(seen.contains(&format!("{edit:?}")), "{edit:?} was applied by some seed: {seen:?}");
    }
}

#[test]
fn each_edit_alone_walks_to_the_oracles_digests_and_file() {
    for edit in edits::ALL {
        let mut applied = 0;
        for seed in 0..10 {
            let mut rng = Rng::new(0x7EE0_8300 + seed);
            let parent = layers::layer(&mut rng, 30 + seed);
            let mut child = parent.clone();
            if edits::apply(&mut rng, &mut child, edit) {
                applied += 1;
                checked(&parent, &child, &SHIPPED, &format!("{edit:?}, seed {seed}"));
            }
        }
        assert!(applied > 3, "{edit:?} found a path to edit in most layers: {applied}");
    }
}

#[test]
fn a_touch_is_read_again_and_is_no_change() {
    let mut parent = Fs::new();
    parent.add(b"f", Node::file(2, b"contents"));
    parent.add(b"g", Node::file(3, b"other"));
    let mut child = parent.clone();
    let at = child.lookup(b"f").expect("f");
    child.touched(at);
    let walked = checked(&parent, &child, &SHIPPED, "touch");
    let counts = counted(&walked);
    assert_eq!((counts.puts, counts.deletes, counts.hashed), (0, 0, 8));
    assert_eq!(hashes(&walked), 1, "f is read; g is not");
}

#[test]
fn an_inode_number_reused_with_a_new_change_time_is_read_and_a_new_entry() {
    let mut parent = Fs::new();
    parent.add(b"f", Node::file(2, b"old"));
    let mut child = parent.clone();
    child.remove(b"f");
    let mut node = Node::file(2, b"new!");
    node.ctime = child.tick();
    child.add(b"f", node);
    let walked = checked(&parent, &child, &SHIPPED, "reuse");
    let counts = counted(&walked);
    assert_eq!((counts.puts, counts.deletes, counts.hashed), (1, 0, 4));
    // The same contents under a reused inode: read again, and no change.
    let mut same = parent.clone();
    same.remove(b"f");
    let mut node = Node::file(2, b"old");
    node.ctime = same.tick();
    same.add(b"f", node);
    let walked = checked(&parent, &same, &SHIPPED, "reuse, same contents");
    assert_eq!((counted(&walked).puts, counted(&walked).hashed), (0, 3));
}

#[test]
fn an_unchanged_directory_is_descended_into_and_a_changed_file_beneath_it_is_a_put() {
    let mut parent = Fs::new();
    parent.add(b"d", Node::new(Kind::Directory, 2));
    parent.add(b"d/f", Node::file(3, b"one"));
    parent.add(b"d/g", Node::file(4, b"two"));
    let mut child = parent.clone();
    let f = child.lookup(b"d/f").expect("d/f");
    child.node_mut(f).content = b"three".to_vec();
    child.touched(f);
    let walked = checked(&parent, &child, &SHIPPED, "a changed file");
    let counts = counted(&walked);
    assert_eq!((counts.puts, counts.deletes, counts.hashed, counts.entries), (1, 0, 5, 4));
    assert_eq!(hashes(&walked), 1);
}

#[test]
fn a_deleted_directory_lists_its_whole_subtree_as_deletes() {
    let mut parent = Fs::new();
    parent.add(b"a", Node::file(2, b"a"));
    parent.add(b"d", Node::new(Kind::Directory, 3));
    parent.add(b"d/x", Node::new(Kind::Directory, 4));
    parent.add(b"d/x/y", Node::file(5, b"y"));
    parent.add(b"d/z", Node::symlink(6, b"a"));
    parent.add(b"e", Node::file(7, b"e"));
    let mut child = parent.clone();
    child.remove(b"d");
    let walked = checked(&parent, &child, &SHIPPED, "a deleted directory");
    let counts = counted(&walked);
    assert_eq!((counts.puts, counts.deletes, counts.hashed, counts.entries), (0, 4, 0, 3));
    let mut gone = Fs::new();
    gone.add(b"a", Node::file(2, b"a"));
    let walked = checked(&parent, &gone, &SHIPPED, "a deleted directory and the tail");
    assert_eq!(counted(&walked).deletes, 5);
}

#[test]
fn a_kind_change_at_a_path_is_a_put_and_what_was_beneath_it_is_deleted() {
    let mut parent = Fs::new();
    parent.add(b"x", Node::new(Kind::Directory, 2));
    parent.add(b"x/y", Node::file(3, b"y"));
    let mut child = parent.clone();
    child.remove(b"x");
    let mut node = Node::file(9, b"now a file");
    node.ctime = child.tick();
    child.add(b"x", node);
    let walked = checked(&parent, &child, &SHIPPED, "directory to file");
    assert_eq!((counted(&walked).puts, counted(&walked).deletes), (1, 1));
    let walked = checked(&child, &parent, &SHIPPED, "file to directory");
    assert_eq!((counted(&walked).puts, counted(&walked).deletes), (2, 0));
}

#[test]
fn links_added_and_removed_read_both_paths_and_the_first_path_in_tree_order_leads() {
    let mut parent = Fs::new();
    let at = parent.add(b"m", Node::file(2, b"shared"));
    parent.add(b"other", Node::file(3, b"o"));
    let mut child = parent.clone();
    child.link(b"a", at);
    child.changed(at);
    let walked = checked(&parent, &child, &SHIPPED, "a link added before the file");
    assert_eq!(counted(&walked).puts, 2, "a is new, m is a hard link now");
    assert_eq!(hashes(&walked), 1, "the first path is hashed, the later one is not");
    // And lost again.
    let walked = checked(&child, &parent, &SHIPPED, "a link removed");
    assert_eq!((counted(&walked).puts, counted(&walked).deletes), (1, 1), "m is a file again, a is gone");
}

#[test]
fn wide_and_deep_layers_walk_against_themselves_and_their_edits() {
    let mut rng = Rng::new(0x7EE0_8400);
    let parent = layers::wide_and_deep(&mut rng, 800, 120);
    let walked = checked(&parent, &parent.clone(), &SHIPPED, "wide and deep, unchanged");
    assert_eq!(hashes(&walked), 0);
    let mut child = parent.clone();
    edits::edit(&mut rng, &mut child, 5, 12);
    checked(&parent, &child, &SHIPPED, "wide and deep, edited");
}

#[test]
fn pages_and_batches_do_not_change_the_walk() {
    let mut rng = Rng::new(0x7EE0_8500);
    let parent = layers::layer(&mut rng, 120);
    let mut child = parent.clone();
    edits::edit(&mut rng, &mut child, 6, 10);
    let whole = checked(&parent, &child, &SHIPPED, "whole");
    for (page, batch) in [(1, 1), (2, 300), (7, 2_000), (1_024, 1)] {
        let limits = Limits { page, batch, ..SHIPPED };
        let paged = checked(&parent, &child, &limits, &format!("page {page}, batch {batch}"));
        assert_eq!(paged.file, whole.file, "page {page}, batch {batch}");
    }
}

#[test]
fn the_end_item_copies_the_parents_kernels() {
    let mut rng = Rng::new(0x7EE0_8600);
    let parent = layers::layer(&mut rng, 20);
    let first = parent_file(&parent, &SHIPPED);
    let mut items = check::read_tree(&first.file, &protocol::SHIPPED).expect("a tree file").into_vec();
    let Some(Item::End(end)) = items.last_mut() else { panic!("the end item") };
    end.boot = Box::new([
        tree::Kernel {
            version: b"6.12.1".as_slice().into(),
            kernel: Digest([1; 32]),
            modules: Digest([2; 32]),
            initramfs: Digest([3; 32]),
        },
        tree::Kernel {
            version: b"6.12.9".as_slice().into(),
            kernel: Digest([4; 32]),
            modules: Digest([5; 32]),
            initramfs: Digest([6; 32]),
        },
    ]);
    let boot = end.boot.clone();
    let file = check::write_tree(&items, &protocol::SHIPPED);
    let walked = against(&parent, &file, &SHIPPED);
    assert!(matches!(walked.answer, Answer::Walked { .. }), "{:?}", walked.answer);
    let read = check::read_tree(&walked.file, &protocol::SHIPPED).expect("a tree file");
    let Some(Item::End(end)) = read.last() else { panic!("the end item") };
    assert_eq!(end.boot, boot, "copied unchanged");
    assert_eq!(end.parent_tree, helper::end_of(&file, &protocol::SHIPPED).tree.into());
}

fn parent_failure(walked: &Walked, what: &str) {
    let (_, cause) = walked.failed();
    assert_eq!(cause, Cause::Parent, "{what}");
}

#[test]
fn a_damaged_parent_stream_fails_the_walk_and_never_gives_digests() {
    let mut rng = Rng::new(0x7EE0_8700);
    let parent = layers::layer(&mut rng, 25);
    let mut child = parent.clone();
    edits::edit(&mut rng, &mut child, 2, 5);
    let first = parent_file(&parent, &SHIPPED);
    let entries = expected::entries(&parent).0.len();
    for tamper in [Tamper::Tree, Tamper::Sha256, Tamper::Size] {
        let mut helper = Helper::new(&child).against(&first.file);
        helper.tamper(tamper);
        let walked = helper::walk_with(helper, &SHIPPED, KEY);
        parent_failure(&walked, &format!("{tamper:?}"));
        assert_eq!(walked.failed().0, b"", "{tamper:?}: at the end of the walk");
    }
    for at in 0..entries - 1 {
        for tamper in [Tamper::Swap(at), Tamper::Repeat(at)] {
            let mut helper = Helper::new(&child).against(&first.file);
            helper.tamper(tamper);
            let walked = helper::walk_with(helper, &SHIPPED, KEY);
            parent_failure(&walked, &format!("{tamper:?}"));
        }
    }
    // Damage in the file itself, found by the stream's own digest: a walk against a file that is
    // not the parent's record.
    let other = parent_file(&child, &SHIPPED);
    let mut helper = Helper::new(&child).against(&other.file);
    helper.tamper(Tamper::Sha256);
    parent_failure(&helper::walk_with(helper, &SHIPPED, KEY), "another file");
}

#[test]
fn an_io_failure_at_the_parent_stream_ends_the_walk_and_never_panics() {
    for seed in 0..12 {
        let mut rng = Rng::new(0x7EE0_8800 + seed);
        let parent = layers::layer(&mut rng, 20);
        let mut child = parent.clone();
        edits::edit(&mut rng, &mut child, 1, 5);
        let first = parent_file(&parent, &SHIPPED);
        let clean = against(&child, &first.file, &SHIPPED);
        assert!(matches!(clean.answer, Answer::Walked { .. }), "seed {seed}");
        for (at, request) in clean.requests.iter().enumerate() {
            if !matches!(request, Request::ParentNext) {
                continue;
            }
            for fault in [Fault::NotFound, Fault::Permission, Fault::NoSpace, Fault::Bound, Fault::Other] {
                let mut helper = Helper::new(&child).against(&first.file);
                helper.fault = Some((at, fault));
                let walked = helper::walk_with(helper, &SHIPPED, KEY);
                let (path, cause) = walked.failed();
                let wanted = clean.requests[at + 1..]
                    .iter()
                    .find_map(|later| if let Request::Stat { path } = later { Some(path.as_ref()) } else { None })
                    .unwrap_or(b"");
                assert_eq!(path, wanted, "seed {seed}, request {at}");
                assert_ne!(cause, Cause::Unexpected, "seed {seed}, request {at}");
                assert_eq!(walked.requests.len(), at + 1, "seed {seed}: nothing is asked after a failure");
            }
        }
    }
}

#[test]
fn every_edit_applied_to_one_layer_at_a_time_is_still_the_oracles() {
    // A layer of every kind, every edit on every path it has, one by one.
    let mut rng = Rng::new(0x7EE0_8900);
    let parent = layers::layer(&mut rng, 14);
    for edit in edits::ALL {
        for round in 0..6 {
            let mut child = parent.clone();
            let mut rng = Rng::new(0x7EE0_8A00 + round);
            if edits::apply(&mut rng, &mut child, edit) {
                checked(&parent, &child, &SHIPPED, &format!("{edit:?}, round {round}"));
            }
        }
    }
}
