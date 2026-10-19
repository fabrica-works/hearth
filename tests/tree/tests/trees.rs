use hearth_lib::encode::Digest;
use hearth_protocol_tree::{
    frame::Unreadable,
    limits::{Limits, TreeLimits},
    tree::{self, ChangesDigest, Entry, Item, Op, TreeDigest, path_valid, tree_order},
};
use hearth_tree_world::{check, fixtures, generate, mutate, oracle};
use skein_lib::Rng;
use std::cmp::Ordering;

const SEEDS: u64 = 48;

fn file_of(seed: u64) -> (Vec<Item>, Box<[u8]>) {
    let mut rng = Rng::new(0x7EE0_5000 + seed);
    let items = generate::tree_file(&mut rng, 1 + seed % 40);
    let file = check::write_tree(&items, &fixtures::limits());
    (items, file)
}

#[test]
fn every_generated_tree_file_reads_back_to_what_was_written_and_meets_the_oracle() {
    let limits = fixtures::limits();
    for seed in 0..SEEDS {
        let (items, file) = file_of(seed);
        assert!(oracle::items_ok(&items), "the generator makes tree files, seed {seed}");
        assert_eq!(check::read_tree(&file, &limits).as_deref(), Ok(items.as_slice()), "seed {seed}");
        let body: Vec<u8> = items.iter().flat_map(oracle::item_bytes).collect();
        let mut expected = tree::header(u64::try_from(body.len()).expect("a file's size")).to_vec();
        expected.extend_from_slice(&body);
        assert_eq!(file.as_ref(), expected, "the header and the items' bytes, seed {seed}");
    }
}

#[test]
fn the_tree_and_change_set_digests_of_a_generated_file_are_the_oracles() {
    for seed in 0..SEEDS {
        let (items, _) = file_of(seed);
        let Some(Item::End(end)) = items.last() else { panic!("the end item is last") };
        let entries: Vec<&Entry> =
            items.iter().filter_map(|item| if let Item::Entry(entry, _) = item { Some(entry) } else { None }).collect();
        assert_eq!(u64::try_from(entries.len()).expect("small"), end.counts.entries, "seed {seed}");
        let mut digest = TreeDigest::new(end.counts.entries);
        for entry in &entries {
            digest.entry(entry);
        }
        assert_eq!(digest.finish(), end.tree, "the tree digest, seed {seed}");
        let mut digest = ChangesDigest::new(end.parent_tree, end.tree, end.counts.puts + end.counts.deletes);
        let mut last: Option<&Entry> = None;
        for item in &items {
            match item {
                Item::Entry(entry, _) => last = Some(entry),
                Item::Change { op: Op::Put, path } => digest.change(Op::Put, path, last),
                Item::Change { op: Op::Delete, path } => digest.change(Op::Delete, path, None),
                Item::End(_) => {}
            }
        }
        assert_eq!(digest.finish(), end.changes, "the change set's digest, seed {seed}");
    }
}

#[test]
fn no_cut_of_a_generated_tree_file_is_read() {
    let limits = fixtures::limits();
    for seed in 0..SEEDS / 8 {
        let (_, file) = file_of(seed);
        for cut in 0..file.len() {
            let error = check::read_tree(&file[..cut], &limits).expect_err("a cut file is no file");
            assert!(
                matches!(error, Unreadable::Short | Unreadable::Length { .. }),
                "seed {seed}, cut {cut}: {error:?}"
            );
        }
    }
}

fn reorder(items: &[Item], rng: &mut Rng, kind: &str) -> Option<Vec<Item>> {
    let mut items = items.to_vec();
    let at = usize::try_from(rng.below(items.len() as u64)).expect("an index");
    match kind {
        "swap" => {
            let other = usize::try_from(rng.below(items.len() as u64)).expect("an index");
            items.swap(at, other);
        }
        "drop" => {
            items.remove(at);
        }
        "twice" => items.insert(at, items[at].clone()),
        _ => return None,
    }
    Some(items)
}

#[test]
fn entries_out_of_order_are_refused_whatever_their_neighbours() {
    let limits = fixtures::limits();
    let mut refused = 0;
    for seed in 0..SEEDS {
        let (items, _) = file_of(seed);
        for at in 1..items.len() {
            let (Item::Entry(before, _), Item::Entry(after, _)) = (&items[at - 1], &items[at]) else { continue };
            let mut swapped = items.clone();
            swapped.swap(at - 1, at);
            let file = check::write_tree(&swapped, &limits);
            assert_eq!(
                check::read_tree(&file, &limits),
                Err(Unreadable::Order(b"path")),
                "{:?} after {:?}, seed {seed}",
                before.path,
                after.path
            );
            refused += 1;
        }
    }
    assert!(refused > 100, "entries were swapped: {refused}");
}

#[test]
fn a_change_for_another_path_is_refused() {
    let limits = fixtures::limits();
    let mut refused = 0;
    for seed in 0..SEEDS {
        let (items, _) = file_of(seed);
        let paths: Vec<_> = items
            .iter()
            .filter_map(|item| if let Item::Entry(entry, _) = item { Some(entry.path.clone()) } else { None })
            .collect();
        for (at, item) in items.iter().enumerate() {
            let Item::Change { op: Op::Put, path } = item else { continue };
            for other in paths.iter().filter(|other| *other != path).take(3) {
                let mut changed = items.clone();
                changed[at] = Item::Change { op: Op::Put, path: other.clone() };
                let file = check::write_tree(&changed, &limits);
                assert_eq!(check::read_tree(&file, &limits), Err(Unreadable::Order(b"change")), "seed {seed}");
                refused += 1;
            }
            let mut moved = items.clone();
            let change = moved.remove(at);
            moved.insert(at + 1, change);
            if matches!(moved[at], Item::End(_)) {
                continue;
            }
            let file = check::write_tree(&moved, &limits);
            assert!(check::read_tree(&file, &limits).is_err(), "a put one item late, seed {seed}");
        }
    }
    assert!(refused > 100, "changes were redirected: {refused}");
}

#[test]
fn an_item_past_its_bound_is_refused_and_one_at_it_is_not() {
    let limits = fixtures::limits();
    for seed in 0..SEEDS {
        let (items, file) = file_of(seed);
        let largest =
            items.iter().map(|item| tree::item(item, &limits).expect("writable").len() - 5).max().expect("items");
        let at = |item: u32| Limits { tree: TreeLimits { item, ..limits.tree }, ..limits };
        let bound = u32::try_from(largest).expect("small");
        assert_eq!(check::read_tree(&file, &at(bound)).as_deref(), Ok(items.as_slice()), "at the bound, seed {seed}");
        assert_eq!(check::read_tree(&file, &at(bound - 1)), Err(Unreadable::Limit(b"item")), "past it, seed {seed}");
        let refused = items.iter().map(|item| tree::item(item, &at(bound - 1))).filter(Result::is_err).count();
        assert!(refused >= 1, "the writer refuses what the reader will not take, seed {seed}");
    }
}

#[test]
fn bytes_after_the_end_are_refused_inside_the_header_and_outside_it() {
    let limits = fixtures::limits();
    for seed in 0..SEEDS {
        let (items, file) = file_of(seed);
        let length = u64::try_from(file.len() - 24).expect("small");
        let mut longer = file.to_vec();
        longer.push(0);
        assert_eq!(
            check::read_tree(&longer, &limits),
            Err(Unreadable::Length { length, after: length + 1 }),
            "a byte after the body, seed {seed}"
        );
        let mut inside = items.clone();
        inside.push(items[0].clone());
        let both = check::write_tree(&inside, &limits);
        let Item::Entry(_, _) = &items[0] else { panic!("the root first") };
        let extra = tree::item(&items[0], &limits).expect("writable").len();
        let total = u64::try_from(both.len() - 24).expect("small");
        assert_eq!(
            check::read_tree(&both, &limits),
            Err(Unreadable::Length { length: total, after: total - u64::try_from(extra).expect("small") }),
            "an item after the end, inside the header's length, seed {seed}"
        );
    }
}

#[test]
fn items_changed_in_order_or_in_value_read_as_the_oracle_says() {
    let limits = fixtures::limits();
    let (mut accepted, mut rejected, mut unwritable) = (0, 0, 0);
    for seed in 0..SEEDS * 8 {
        let mut rng = Rng::new(0x7EE0_6000 + seed);
        let (items, _) = file_of(seed % SEEDS);
        let kinds = ["swap", "drop", "twice", "path", "mode"];
        let kind = kinds[usize::try_from(rng.below(kinds.len() as u64)).expect("an index")];
        let mut changed = reorder(&items, &mut rng, kind).unwrap_or_else(|| items.clone());
        if kind == "path" || kind == "mode" {
            let at = usize::try_from(rng.below(changed.len() as u64)).expect("an index");
            let paths: [&[u8]; 9] = [b"", b"a", b"a/b", b"a//b", b"/a", b"a/", b"..", b"a\0b", b"zz"];
            let path: Box<[u8]> = paths[usize::try_from(rng.below(9)).expect("an index")].into();
            match &mut changed[at] {
                Item::Entry(entry, _) if kind == "path" => entry.path = path,
                Item::Entry(entry, _) => match rng.below(3) {
                    0 => entry.mode = 0o10000,
                    1 => entry.uid = 65_536,
                    _ => entry.gid = 70_000,
                },
                Item::Change { path: own, .. } if kind == "path" => *own = path,
                Item::Change { .. } | Item::End(_) => {}
            }
        }
        let mut parts = Vec::new();
        let mut writable = true;
        for item in &changed {
            let written = tree::item(item, &limits);
            assert_eq!(
                written.is_ok(),
                oracle::item_ok(item),
                "the writer and the oracle agree on {item:?}, seed {seed}"
            );
            match written {
                Ok(bytes) => parts.push(bytes),
                Err(_) => writable = false,
            }
        }
        if !writable {
            unwritable += 1;
            continue;
        }
        let mut file = tree::header(parts.iter().map(|part| part.len() as u64).sum()).to_vec();
        for part in &parts {
            file.extend_from_slice(part);
        }
        let read = check::read_tree(&file, &limits);
        assert_eq!(read.is_ok(), oracle::items_ok(&changed), "{kind}, seed {seed}: {:?}", read.as_ref().err());
        if let Ok(read) = read {
            assert_eq!(read.as_ref(), changed.as_slice(), "{kind}, seed {seed}");
            accepted += 1;
        } else {
            rejected += 1;
        }
    }
    assert!(
        accepted > 20 && rejected > 100 && unwritable > 20,
        "{accepted} read, {rejected} refused, {unwritable} unwritable"
    );
}

#[test]
fn a_mutated_tree_file_is_refused_or_is_what_the_oracle_takes_in_its_one_encoding() {
    let limits = fixtures::limits();
    for seed in 0..SEEDS {
        let mut rng = Rng::new(0x7EE0_7000 + seed);
        let (_, file) = file_of(seed);
        for changed in mutate::frames(&mut rng, &file) {
            if let Ok(items) = check::read_tree(&changed, &limits) {
                assert!(oracle::items_ok(&items), "seed {seed}");
                assert_eq!(check::write_tree(&items, &limits).as_ref(), changed.as_slice(), "seed {seed}");
            }
        }
    }
}

fn path_from(rng: &mut Rng) -> Vec<u8> {
    let alphabet: &[u8] = b"ab-. /0\0";
    let length = rng.between(0, 8);
    (0..length).map(|_| alphabet[usize::try_from(rng.below(alphabet.len() as u64)).expect("an index")]).collect()
}

#[test]
fn tree_order_is_a_total_order_that_agrees_with_the_component_wise_oracle() {
    let mut rng = Rng::new(0x7EE0_8000);
    let paths: Vec<Vec<u8>> = (0..200).map(|_| path_from(&mut rng)).collect();
    for a in &paths {
        for b in &paths {
            let order = tree_order(a, b);
            assert_eq!(order, oracle::order(a, b), "{a:?} against {b:?}");
            assert_eq!(tree_order(b, a), order.reverse(), "antisymmetric: {a:?}, {b:?}");
            assert_eq!(order == Ordering::Equal, a == b, "equal only when the same bytes: {a:?}, {b:?}");
        }
    }
    for _ in 0..20_000 {
        let pick = |rng: &mut Rng| &paths[usize::try_from(rng.below(paths.len() as u64)).expect("an index")];
        let (a, b, c) = (pick(&mut rng), pick(&mut rng), pick(&mut rng));
        if tree_order(a, b) != Ordering::Greater && tree_order(b, c) != Ordering::Greater {
            assert_ne!(tree_order(a, c), Ordering::Greater, "transitive: {a:?}, {b:?}, {c:?}");
        }
    }
    let mut ours = paths.clone();
    ours.sort_by(|a, b| tree_order(a, b));
    let mut theirs = paths.clone();
    theirs.sort_by(|a, b| oracle::order(a, b));
    assert_eq!(ours, theirs, "sorting by either gives one sequence");
}

#[test]
fn a_path_is_valid_exactly_when_the_oracle_says_so() {
    let mut rng = Rng::new(0x7EE0_9000);
    for _ in 0..20_000 {
        let path = path_from(&mut rng);
        assert_eq!(path_valid(&path), oracle::valid(&path), "{path:?}");
    }
    for length in [4_094, 4_095, 4_096, 4_097, 4_098] {
        for fill in [b'a', b'/'] {
            let mut path = vec![b'a'; length];
            path[length / 2] = fill;
            assert_eq!(path_valid(&path), oracle::valid(&path), "{length} bytes with {}", char::from(fill));
        }
    }
}

#[test]
fn a_reader_fed_bytes_that_are_no_tree_file_refuses_them_without_panicking() {
    let limits = fixtures::limits();
    for seed in 0..SEEDS * 8 {
        let mut rng = Rng::new(0x7EE0_A000 + seed);
        let (_, file) = file_of(seed % SEEDS);
        let mut bytes = file.to_vec();
        for _ in 0..rng.between(1, 16) {
            let at = usize::try_from(rng.below(bytes.len() as u64)).expect("an index");
            bytes[at] = u8::try_from(rng.below(256)).expect("a byte");
        }
        let _: Result<Box<[Item]>, Unreadable> = check::read_tree(&bytes, &limits);
        let noise: Vec<u8> = (0..rng.between(0, 400)).map(|_| u8::try_from(rng.below(256)).expect("a byte")).collect();
        let mut garbage = tree::header(u64::try_from(noise.len()).expect("small")).to_vec();
        garbage.extend_from_slice(&noise);
        let _: Result<Box<[Item]>, Unreadable> = check::read_tree(&garbage, &limits);
    }
}

#[test]
fn the_root_entry_of_the_empty_layer_hashes_to_store_md_3_2s_vector() {
    let entry = Entry {
        path: Box::from([]),
        mode: 0o755,
        uid: 0,
        gid: 0,
        xattrs: Box::from([]),
        payload: tree::Payload::Directory,
    };
    let text = "6022d1fbc6ad38d08d2ae02cefc21e7e6af5a35e6be9d115f0ba267fdb0d005a";
    let mut expected = [0_u8; 32];
    for (at, byte) in expected.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[at * 2..at * 2 + 2], 16).expect("a hex pair");
    }
    let mut digest = TreeDigest::new(1);
    digest.entry(&entry);
    assert_eq!(digest.finish(), Digest(expected), "the code under test");
    assert_eq!(oracle::tree_digest(&[&entry]), Digest(expected), "and the oracle");
}
