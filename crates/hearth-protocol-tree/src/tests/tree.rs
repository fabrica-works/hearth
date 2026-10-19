use super::fixtures::{SHIPPED, hex};
use crate::{
    decode::Malformed,
    frame::{Refused, Unreadable},
    layer::Counts,
    limits::{Limits, TreeLimits},
    tree::{
        self, ChangesDigest, End, Entry, Item, Kernel, Local, Next, Op, Payload, Reader, TreeDigest, Xattr,
    },
};
use core::cmp::Ordering;
use hearth_lib::encode::{Digest, Encoder, Item as Value};
use skein_lib::{Decimal, List, Writer, bytes};

fn d(byte: u8) -> Digest {
    Digest([byte; 32])
}
fn text(value: &[u8]) -> Box<[u8]> {
    bytes::copy_of(value)
}
fn entry(path: &[u8], mode: u32, payload: Payload) -> Entry {
    Entry { path: text(path), mode, uid: 0, gid: 0, xattrs: Box::from([]), payload }
}
fn owned(entry: Entry, uid: u32, gid: u32, xattrs: &[(&[u8], &[u8])]) -> Entry {
    let mut result = List::with_capacity(u32::try_from(xattrs.len()).expect("few attributes"));
    for (name, value) in xattrs {
        result.push(Xattr { name: text(name), value: text(value) }).expect("room");
    }
    Entry { uid, gid, xattrs: result.into_boxed(), ..entry }
}
fn local() -> Local {
    Local { inode: 5, ctime: (1_760_000_000, 123), mtime: (1_760_000_001, 456) }
}
fn root() -> Entry {
    entry(b"", 0o755, Payload::Directory)
}
fn dir(path: &[u8]) -> Entry {
    entry(path, 0o755, Payload::Directory)
}
/// 3.4's eight kinds in tree order, with owners and attributes.
fn each_kind() -> Box<[Entry]> {
    let capability: &[u8] = b"\x01\x00\x00\x02";
    Box::from([
        root(),
        entry(b"bin", 0o777, Payload::Symlink { target: text(b"usr/bin") }),
        dir(b"dev"),
        entry(b"dev/null", 0o666, Payload::Char { major: 1, minor: 3 }),
        owned(entry(b"dev/sda", 0o660, Payload::Block { major: 8, minor: 0 }), 0, 6, &[]),
        owned(dir(b"etc"), 0, 0, &[(b"user.note", b"x")]),
        entry(b"etc/motd", 0o644, Payload::File { size: 12, content: d(7) }),
        entry(b"etc/motd.link", 0o644, Payload::Hardlink { path: text(b"etc/motd") }),
        dir(b"run"),
        owned(entry(b"run/pipe", 0o600, Payload::Fifo), 1000, 1000, &[]),
        owned(
            entry(b"run/sock", 0o755, Payload::Socket),
            65_535,
            65_535,
            &[(b"security.capability", capability), (b"user.a", b"")],
        ),
    ])
}
fn tree_digest(entries: &[Entry]) -> Digest {
    let mut digest = TreeDigest::new(u64::try_from(entries.len()).expect("few entries"));
    for entry in entries {
        digest.entry(entry);
    }
    digest.finish()
}
fn digest_of_hex(text: &str) -> Digest {
    Digest(hex(text).as_ref().try_into().expect("a digest"))
}
fn changes(parent: Option<Digest>, tree: Digest, list: &[(Op, &[u8], Option<&Entry>)]) -> Digest {
    let mut digest = ChangesDigest::new(parent, tree, u64::try_from(list.len()).expect("few changes"));
    for (op, path, entry) in list {
        digest.change(*op, path, *entry);
    }
    digest.finish()
}

// The expected digests and bytes come from an independent encoder, a few lines
// of Python written from 3.1 alone, which also reproduces 3.2's vector.
#[test]
fn the_empty_layers_root_entry_hashes_to_store_md_3_2s_vector() {
    assert_eq!(
        tree_digest(&[root()]),
        digest_of_hex("6022d1fbc6ad38d08d2ae02cefc21e7e6af5a35e6be9d115f0ba267fdb0d005a"),
        "3.2's tree vector"
    );
}

#[test]
fn an_entry_of_each_kind_with_owners_attributes_and_a_hard_link_hashes_to_its_vector() {
    assert_eq!(
        tree_digest(&each_kind()),
        digest_of_hex("f978e648f5076c33ac539aceb325722412dc4e33f175edf03ee2969de25a0013"),
        "eleven entries"
    );
}

#[test]
fn a_change_set_hashes_to_its_vectors_with_a_parent_without_one_and_empty() {
    let all = each_kind();
    let tree = tree_digest(&all);
    let motd = all.get(6).expect("the file");
    assert_eq!(
        changes(
            Some(d(0xaa)),
            tree,
            &[(Op::Put, b"etc/motd", Some(motd)), (Op::Delete, b"old", None), (Op::Delete, b"old/file", None)]
        ),
        digest_of_hex("7f02f9884ff06d3433525e873406437c518a6379a0b45c401742526adc1fc04b"),
        "a put and two deletes"
    );
    assert_eq!(
        changes(None, tree, &[(Op::Put, b"", Some(&root()))]),
        digest_of_hex("d53561d3e2136c9dd03a9a799e1ec2e58d42ae99db1912cf81776bc6f25f530d"),
        "no parent hashes none"
    );
    assert_eq!(
        changes(Some(d(0xaa)), tree, &[]),
        digest_of_hex("214ffafd4f0d84272cce12d981a0fd91ebd767a66b78f9ea68379c06fb100083"),
        "no change"
    );
}

#[test]
#[should_panic(expected = "an encoding is finished with its one value whole")]
fn a_tree_digest_told_more_entries_than_it_is_fed_panics() {
    let mut digest = TreeDigest::new(2);
    digest.entry(&root());
    assert_eq!(digest.finish(), d(0), "finishing panics before this");
}

#[test]
#[should_panic(expected = "an encoding holds one value")]
fn a_tree_digest_fed_more_entries_than_it_was_told_panics() {
    let mut digest = TreeDigest::new(1);
    digest.entry(&root());
    digest.entry(&root());
}

#[test]
#[should_panic(expected = "a put carries the child's entry, and a delete none")]
fn a_put_without_its_entry_panics() {
    ChangesDigest::new(None, d(1), 1).change(Op::Put, b"a", None);
}

#[test]
#[should_panic(expected = "a put carries the entry at its path")]
fn a_put_with_another_paths_entry_panics() {
    ChangesDigest::new(None, d(1), 1).change(Op::Put, b"b", Some(&entry(b"a", 0o644, Payload::Fifo)));
}

fn written(item: &Item) -> Box<[u8]> {
    tree::item(item, &SHIPPED).expect("a bounded item")
}
fn counts() -> Counts {
    Counts { bytes: 12, deletes: 2, entries: 11, files: 1, hashed: 12, puts: 1 }
}
fn an_end(parent_tree: Option<Digest>, boot: Box<[Kernel]>) -> Item {
    let tree = digest_of_hex("f978e648f5076c33ac539aceb325722412dc4e33f175edf03ee2969de25a0013");
    Item::End(End { key: d(0x11), parent_tree, tree, changes: d(0x22), counts: counts(), boot })
}
pub(super) fn a_kernel(version: &[u8]) -> Kernel {
    Kernel { version: text(version), kernel: d(1), modules: d(2), initramfs: d(3) }
}
/// An end item that needs the decoder's depth and the longest key.
pub(super) fn an_end_with_a_kernel() -> Item {
    an_end(Some(d(0xaa)), Box::from([a_kernel(b"6.12.1-amd64")]))
}

#[test]
fn items_are_written_as_tag_length_and_canonical_value_byte_for_byte() {
    let put = Item::Change { op: Op::Put, path: text(b"etc/motd") };
    let delete = Item::Change { op: Op::Delete, path: text(b"old") };
    assert_eq!(
        written(&Item::Entry(root(), local())),
        hex(concat!(
            "010000007f06000000000000000206000000000000000704000000000000000003000000000000000103000000000000",
            "01ed03000000000000000003000000000000000007000000000000000000060000000000000005030000000000000005",
            "030000000068e7780003000000000000007b030000000068e778010300000000000001c8",
        )),
        "the root entry and its local part"
    );
    assert_eq!(
        written(&put),
        hex("02 00000023 06 0000000000000002 03 0000000000000001 04 0000000000000008 6574632f6d6f7464"),
        "a put"
    );
    assert_eq!(
        written(&delete),
        hex("02 0000001e 06 0000000000000002 03 0000000000000002 04 0000000000000003 6f6c64"),
        "a delete"
    );
    assert_eq!(
        written(&an_end_with_a_kernel()),
        hex(concat!(
            "0300000248070000000000000006040000000000000004626f6f74060000000000000001070000000000000004040000",
            "000000000009696e697472616d6673050303030303030303030303030303030303030303030303030303030303030303",
            "0400000000000000066b65726e656c050101010101010101010101010101010101010101010101010101010101010101",
            "0400000000000000076d6f64756c65730502020202020202020202020202020202020202020202020202020202020202",
            "0204000000000000000776657273696f6e04000000000000000c362e31322e312d616d64363404000000000000000763",
            "68616e676573052222222222222222222222222222222222222222222222222222222222222222040000000000000006",
            "636f756e7473070000000000000006040000000000000005627974657303000000000000000c04000000000000000764",
            "656c65746573030000000000000002040000000000000007656e747269657303000000000000000b0400000000000000",
            "0566696c657303000000000000000104000000000000000668617368656403000000000000000c040000000000000004",
            "707574730300000000000000010400000000000000036b65790511111111111111111111111111111111111111111111",
            "1111111111111111111104000000000000000b706172656e745f7472656505aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa0400000000000000047472656505f978e648f5076c33ac539aceb325722412dc4e",
            "33f175edf03ee2969de25a0013",
        )),
        "an end item with a kernel"
    );
    assert_eq!(
        written(&an_end(None, Box::from([]))),
        hex(concat!(
            "0300000166070000000000000006040000000000000004626f6f74060000000000000000040000000000000007636861",
            "6e676573052222222222222222222222222222222222222222222222222222222222222222040000000000000006636f",
            "756e7473070000000000000006040000000000000005627974657303000000000000000c04000000000000000764656c",
            "65746573030000000000000002040000000000000007656e747269657303000000000000000b04000000000000000566",
            "696c657303000000000000000104000000000000000668617368656403000000000000000c0400000000000000047075",
            "74730300000000000000010400000000000000036b657905111111111111111111111111111111111111111111111111",
            "111111111111111104000000000000000b706172656e745f74726565000400000000000000047472656505f978e648f5",
            "076c33ac539aceb325722412dc4e33f175edf03ee2969de25a0013",
        )),
        "an end item with no parent and no kernel"
    );
}

#[test]
fn a_tree_files_header_is_kind_3s_with_the_bodys_length() {
    assert_eq!(
        tree::header(0x0102_0304_0506),
        hex("6865617274680072 0003 0001 00000000 0000010203040506").as_ref(),
        "6.1's header"
    );
}

// Files.

fn joined(parts: &[&[u8]]) -> Box<[u8]> {
    let mut size = 0_usize;
    for part in parts {
        size = size.checked_add(part.len()).expect("a fixture fits");
    }
    let mut writer = Writer::new(size);
    for part in parts {
        writer.put(part).expect("room");
    }
    writer.finish()
}
/// The header for the items' total length, then the items.
fn file(parts: &[&[u8]]) -> Box<[u8]> {
    let body = joined(parts);
    let head = tree::header(u64::try_from(body.len()).expect("a fixture fits"));
    joined(&[&head, &body])
}
/// A file whose header names `extra` bytes more than the items hold.
fn file_longer_by(parts: &[&[u8]], extra: u64) -> Box<[u8]> {
    let body = joined(parts);
    let head = tree::header(u64::try_from(body.len()).expect("a fixture fits").checked_add(extra).expect("small"));
    joined(&[&head, &body])
}
fn ent(path: &[u8]) -> Box<[u8]> {
    written(&Item::Entry(dir(path), local()))
}
fn put(path: &[u8]) -> Box<[u8]> {
    written(&Item::Change { op: Op::Put, path: text(path) })
}
fn delete(path: &[u8]) -> Box<[u8]> {
    written(&Item::Change { op: Op::Delete, path: text(path) })
}
fn end() -> Box<[u8]> {
    written(&an_end(None, Box::from([])))
}

/// Reads a whole file from its bytes, feeding the front of what remains.
fn read(bytes: &[u8], limits: &Limits) -> Result<Box<[Item]>, Unreadable> {
    let mut reader = Reader::new(limits);
    let head = <&[u8; 24]>::try_from(bytes.get(..24).ok_or(Unreadable::Short)?).expect("24 bytes");
    let length = reader.header(head)?;
    let mut result = List::with_capacity(u32::try_from(bytes.len()).expect("a fixture fits"));
    let mut at = 24_usize;
    for _ in 0..bytes.len() {
        match reader.next(bytes.get(at..).expect("within the file"))? {
            Next::Item(item, used) => {
                at = at.checked_add(usize::try_from(used).expect("small")).expect("a fixture fits");
                let last = reader.ended();
                result.push(item).expect("room");
                if last {
                    return Ok(result.into_boxed());
                }
            }
            Next::Need(_) => {
                let after = u64::try_from(bytes.len().checked_sub(24).expect("a header")).expect("small");
                return Err(Unreadable::Length { length, after });
            }
        }
    }
    unreachable!("each item takes a byte")
}
fn refused(parts: &[&[u8]], expected: Unreadable, what: &str) {
    assert_eq!(read(&file(parts), &SHIPPED), Err(expected), "{what}");
}
fn accepted(parts: &[&[u8]], what: &str) {
    assert!(read(&file(parts), &SHIPPED).is_ok(), "{what}");
}

#[test]
fn a_file_reads_back_item_by_item_to_what_was_written() {
    let all = each_kind();
    let mut items = List::with_capacity(32);
    for (number, entry) in all.iter().enumerate() {
        items.push(Item::Entry(entry.clone(), local())).expect("room");
        if number == 6 {
            items.push(Item::Change { op: Op::Put, path: entry.path.clone() }).expect("room");
        }
        if number == 8 {
            items.push(Item::Change { op: Op::Delete, path: text(b"run/old") }).expect("room");
        }
    }
    items.push(an_end_with_a_kernel()).expect("room");
    let items = items.into_boxed();
    let mut parts = List::with_capacity(32);
    for item in &items {
        parts.push(written(item)).expect("room");
    }
    let mut refs = List::with_capacity(32);
    for part in &parts {
        refs.push(part.as_ref()).expect("room");
    }
    assert_eq!(read(&file(refs.as_slice()), &SHIPPED), Ok(items), "every item as written");
}

#[test]
fn the_reader_names_the_bytes_an_item_needs_before_it_reads_it() {
    let first = written(&Item::Entry(root(), local()));
    let size = u32::try_from(first.len()).expect("small");
    let bytes = file(&[&first, &end()]);
    let mut reader = Reader::new(&SHIPPED);
    reader.header(<&[u8; 24]>::try_from(bytes.get(..24).expect("a header")).expect("24")).expect("a header");
    let body = bytes.get(24..).expect("a body");
    assert_eq!(reader.next(body.get(..0).expect("none")), Ok(Next::Need(5)), "no head yet");
    assert_eq!(reader.next(body.get(..4).expect("four")), Ok(Next::Need(5)), "a short head");
    assert_eq!(reader.next(body.get(..5).expect("five")), Ok(Next::Need(size)), "the head names the whole item");
    assert_eq!(reader.next(body.get(..first.len() - 1).expect("short")), Ok(Next::Need(size)), "a cut value");
    assert_eq!(reader.next(body), Ok(Next::Item(Item::Entry(root(), local()), size)), "the bytes it took");
    assert!(!reader.ended(), "the end is still to come");
    let rest = body.get(first.len()..).expect("the end item");
    let Ok(Next::Item(last, _)) = reader.next(rest) else { panic!("the end item") };
    assert_eq!(last, an_end(None, Box::from([])), "the end item");
    assert!(reader.ended(), "now ended");
    assert_eq!(reader.next(rest), Err(Unreadable::Record), "nothing follows the end");
}

#[test]
fn the_tree_order_of_3_4_is_the_order_entries_are_read_in() {
    accepted(&[&ent(b""), &ent(b"a"), &ent(b"a/b"), &ent(b"a-b"), &ent(b"b"), &end()], "a, a/b, a-b");
    refused(&[&ent(b"a"), &ent(b"a-b"), &ent(b"a/b"), &end()], Unreadable::Order(b"path"), "a/b after a-b");
    refused(&[&ent(b"a"), &ent(b"a"), &end()], Unreadable::Order(b"path"), "a path twice");
    refused(&[&ent(b"b"), &ent(b"a"), &end()], Unreadable::Order(b"path"), "b before a");
    refused(&[&ent(b"a"), &ent(b""), &end()], Unreadable::Order(b"path"), "the root last");
}

#[test]
fn a_put_follows_its_own_entry_and_a_delete_stands_alone() {
    accepted(&[&ent(b"a"), &put(b"a"), &ent(b"b"), &end()], "a put after its entry");
    accepted(&[&ent(b"a"), &delete(b"b"), &ent(b"c"), &end()], "a delete between entries");
    accepted(&[&ent(b"a"), &put(b"a"), &delete(b"b"), &delete(b"b/c"), &end()], "deletes after a put");
    accepted(&[&delete(b"a"), &end()], "a delete alone");
    refused(&[&ent(b"a"), &put(b"b"), &end()], Unreadable::Order(b"change"), "a put for another path");
    refused(&[&put(b"a"), &end()], Unreadable::Order(b"change"), "a put with no entry");
    refused(&[&ent(b"a"), &put(b"a"), &put(b"a"), &end()], Unreadable::Order(b"change"), "a put twice");
    refused(&[&ent(b"a"), &ent(b"b"), &put(b"a"), &end()], Unreadable::Order(b"change"), "a put after the next entry");
    refused(&[&ent(b"a"), &delete(b"a"), &end()], Unreadable::Order(b"change"), "a delete at an entry's path");
    refused(&[&ent(b"b"), &delete(b"a"), &end()], Unreadable::Order(b"change"), "a delete before the last entry");
    refused(&[&delete(b"b"), &delete(b"b"), &end()], Unreadable::Order(b"change"), "a delete twice");
    refused(&[&delete(b"c"), &ent(b"b"), &end()], Unreadable::Order(b"path"), "an entry before a delete");
}

#[test]
fn the_end_item_is_last_and_the_body_ends_exactly_there() {
    let (first, last) = (ent(b"a"), end());
    accepted(&[&first, &last], "an entry and the end");
    let trailing = file(&[&first, &last, &ent(b"b")]);
    let body = u64::try_from(trailing.len() - 24).expect("small");
    let taken = body - u64::try_from(ent(b"b").len()).expect("small");
    assert_eq!(
        read(&trailing, &SHIPPED),
        Err(Unreadable::Length { length: body, after: taken }),
        "an item after the end"
    );
    let slack = file_longer_by(&[&first, &last], 1);
    let held = u64::try_from(slack.len() - 24).expect("small");
    assert_eq!(
        read(&slack, &SHIPPED),
        Err(Unreadable::Length { length: held + 1, after: held }),
        "a header that names a byte more than the end item leaves"
    );
    refused(&[&first], Unreadable::Missing(b"end"), "a body that ends before the end item");
    let size = u64::try_from(first.len()).expect("small");
    assert_eq!(
        read(&file_longer_by(&[&first], 10), &SHIPPED),
        Err(Unreadable::Length { length: size + 10, after: size }),
        "a file that stops before the header's length"
    );
}

#[test]
fn an_item_longer_than_the_body_has_left_is_refused_before_its_value() {
    let item = ent(b"a");
    let size = u64::try_from(item.len()).expect("small");
    assert_eq!(
        read(&joined(&[&tree::header(size - 1), &item]), &SHIPPED),
        Err(Unreadable::Length { length: size - 1, after: size }),
        "the item passes the body's end"
    );
    let short = joined(&[&tree::header(3), &[1, 0, 0]]);
    assert_eq!(read(&short, &SHIPPED), Err(Unreadable::Length { length: 3, after: 5 }), "no room for a head");
}

#[test]
fn an_item_past_its_bound_is_refused_from_its_head_alone() {
    let limits = Limits { tree: TreeLimits { item: 100, ..SHIPPED.tree }, ..SHIPPED };
    let item = ent(b"a");
    assert!(item.len() > 105, "the fixture is past the bound");
    let mut reader = Reader::new(&limits);
    reader.header(&tree::header(1_000)).expect("a header");
    assert_eq!(reader.next(item.get(..5).expect("a head")), Err(Unreadable::Limit(b"item")), "from five bytes");
    assert_eq!(reader.next(&item), Err(Unreadable::Limit(b"item")), "and the reader stays refused");
    assert_eq!(read(&file(&[&item, &end()]), &limits), Err(Unreadable::Limit(b"item")), "in a whole file");
}

#[test]
fn an_item_of_an_unknown_tag_is_refused() {
    for found in [0_u8, 4, 9, 255] {
        refused(&[&[found, 0, 0, 0, 0]], Unreadable::Tag { found }, "an unknown tag");
    }
}

#[test]
fn the_header_is_checked_in_order_and_the_bodys_bound_first() {
    let mut reader = Reader::new(&SHIPPED);
    assert_eq!(reader.next(&[]), Err(Unreadable::Missing(b"header")), "items before the header");
    let rows: &[(&str, Unreadable)] = &[
        ("0065617274680072 0003 0001 00000000 0000000000000000", Unreadable::Magic),
        ("6865617274680072 0002 0001 00000000 0000000000000000", Unreadable::Kind { found: 2 }),
        ("6865617274680072 0003 0002 00000000 0000000000000000", Unreadable::Version { found: 2 }),
        ("6865617274680072 0003 0001 00000001 0000000000000000", Unreadable::Reserved),
        ("6865617274680072 0003 0001 00000000 0000000040000001", Unreadable::Bound { length: 1_073_741_825 }),
    ];
    for (header, expected) in rows {
        let mut reader = Reader::new(&SHIPPED);
        let header = <[u8; 24]>::try_from(hex(header).as_ref()).expect("24 bytes");
        assert_eq!(reader.header(&header), Err(*expected), "{header:?}");
        assert_eq!(reader.header(&header), Err(*expected), "and it stays refused");
    }
    let mut reader = Reader::new(&SHIPPED);
    assert_eq!(reader.header(&tree::header(1_073_741_824)), Ok(1_073_741_824), "a body at its bound");
    assert_eq!(reader.header(&tree::header(0)), Err(Unreadable::Record), "a second header");
}

// Hand-made items.

fn uint(number: u64) -> Value {
    Value::Uint(number)
}
fn name(value: &[u8]) -> Value {
    Value::Bytes(text(value))
}
fn values(parts: &[&[Value]]) -> Box<[Value]> {
    let mut result = List::with_capacity(256);
    for part in parts {
        for value in *part {
            result.push(value.clone()).expect("room");
        }
    }
    result.into_boxed()
}
fn raw(tag: u8, values: &[Value]) -> Box<[u8]> {
    let mut measure = Encoder::measuring();
    for value in values {
        measure.item(value);
    }
    let length = measure.measured();
    let mut writer = Encoder::writing(length);
    for value in values {
        writer.item(value);
    }
    let length = u32::try_from(length).expect("small");
    joined(&[&[tag], &length.to_be_bytes(), &writer.written()])
}
/// An entry item's values, of a directory unless the payload says otherwise.
fn entry_values(
    path: &[u8],
    kind: u64,
    mode: u64,
    owners: (u64, u64),
    attributes: &[Value],
    payload: &[Value],
) -> Box<[Value]> {
    let count = u64::try_from(attributes.len() / 2).expect("small");
    values(&[
        &[
            Value::List(2),
            Value::List(7),
            name(path),
            uint(kind),
            uint(mode),
            uint(owners.0),
            uint(owners.1),
            Value::Map(count),
        ],
        attributes,
        payload,
        &[Value::List(5), uint(5), uint(1), uint(2), uint(3), uint(4)],
    ])
}
fn plain(path: &[u8], kind: u64, mode: u64, owners: (u64, u64), payload: &[Value]) -> Box<[u8]> {
    raw(1, &entry_values(path, kind, mode, owners, &[], payload))
}
fn directory(path: &[u8]) -> Box<[u8]> {
    plain(path, 1, 0o755, (0, 0), &[Value::None])
}

#[test]
fn a_path_that_is_no_path_makes_an_entry_unreadable() {
    let long = [b'a'; 4_097];
    accepted(&[&directory(&long[..4_096]), &end()], "an entry at the longest path");
    refused(&[&directory(&long), &end()], Unreadable::Limit(b"path"), "4097 bytes");
    for path in [&b"/"[..], b"/a", b"a/", b"a//b", b".", b"..", b"a/.", b"a/..", b"./a", b"a\0", b"\0"] {
        refused(&[&directory(path), &end()], Unreadable::Shape(b"path"), "an invalid path");
        refused(
            &[&raw(2, &[Value::List(2), uint(1), name(path)]), &end()],
            Unreadable::Shape(b"path"),
            "an invalid change path",
        );
    }
}

#[test]
fn an_entry_whose_fields_are_out_of_range_or_of_the_wrong_shape_is_unreadable() {
    let none = [Value::None];
    refused(&[&plain(b"a", 1, 0o10000, (0, 0), &none), &end()], Unreadable::Limit(b"mode"), "a mode past 0o7777");
    accepted(&[&plain(b"a", 1, 0o7777, (65_535, 65_535), &none), &end()], "the largest mode and owners");
    refused(&[&plain(b"a", 1, 0, (65_536, 0), &none), &end()], Unreadable::Limit(b"uid"), "a uid past 65535");
    refused(&[&plain(b"a", 1, 0, (0, 65_536), &none), &end()], Unreadable::Limit(b"gid"), "a gid past 65535");
    refused(
        &[&plain(b"a", 1, 0, (0, 1_099_511_627_776), &none), &end()],
        Unreadable::Limit(b"gid"),
        "a gid past a u32",
    );
    for kind in [0, 9, 255] {
        refused(&[&plain(b"a", kind, 0, (0, 0), &none), &end()], Unreadable::Shape(b"kind"), "an unknown kind");
    }
    refused(
        &[&plain(b"a", 1, 0, (0, 0), &[uint(0)]), &end()],
        Unreadable::Shape(b"payload"),
        "a directory with a payload",
    );
    refused(&[&plain(b"a", 7, 0, (0, 0), &[uint(0)]), &end()], Unreadable::Shape(b"payload"), "a fifo with a payload");
    refused(
        &[&plain(b"a", 8, 0, (0, 0), &[uint(0)]), &end()],
        Unreadable::Shape(b"payload"),
        "a socket with a payload",
    );
    refused(&[&plain(b"a", 2, 0, (0, 0), &none), &end()], Unreadable::Shape(b"payload"), "a file with none");
    refused(
        &[&plain(b"a", 2, 0, (0, 0), &[Value::List(3), uint(1), Value::Digest(d(1)), uint(1)]), &end()],
        Unreadable::Limit(b"payload"),
        "a file with three",
    );
    refused(
        &[&plain(b"a", 2, 0, (0, 0), &[Value::List(2), Value::Digest(d(1)), uint(1)]), &end()],
        Unreadable::Shape(b"size"),
        "a file with its fields swapped",
    );
    refused(&[&plain(b"a", 3, 0, (0, 0), &[uint(1)]), &end()], Unreadable::Shape(b"target"), "a link with a number");
    refused(
        &[&plain(b"a", 3, 0, (0, 0), &[Value::Bytes(text(&[b'x'; 4_097]))]), &end()],
        Unreadable::Limit(b"target"),
        "a target past 4096",
    );
    accepted(&[&plain(b"a", 3, 0, (0, 0), &[Value::Bytes(text(&[b'x'; 4_096]))]), &end()], "a target of 4096");
    accepted(&[&plain(b"a", 3, 0, (0, 0), &[name(b"../any/thing/\0")]), &end()], "a target is as stored");
    refused(
        &[&plain(b"a", 4, 0, (0, 0), &[name(b"/x")]), &end()],
        Unreadable::Shape(b"payload"),
        "a hard link to a bad path",
    );
    accepted(&[&plain(b"b", 4, 0, (0, 0), &[name(b"a")]), &end()], "a hard link");
    refused(
        &[&plain(b"a", 5, 0, (0, 0), &[Value::List(2), uint(1), uint(4_294_967_296)]), &end()],
        Unreadable::Limit(b"minor"),
        "a minor past a u32",
    );
    accepted(&[&plain(b"a", 6, 0, (0, 0), &[Value::List(2), uint(1), uint(1)]), &end()], "a block device");
}

/// `count` name and value pairs: a prefix and three digits, and `value` bytes.
fn pairs(count: usize, prefix: &[u8], value: usize) -> Box<[Value]> {
    let mut result = List::with_capacity(512);
    for number in 0..count {
        let decimal = Decimal::of(u64::try_from(number).expect("small"));
        let zeros = [b'0'; 3];
        let padding = zeros.get(decimal.as_bytes().len()..).expect("at most three digits");
        result.push(Value::Bytes(joined(&[prefix, padding, decimal.as_bytes()]))).expect("room");
        result.push(Value::Bytes(bytes::zeroed(value))).expect("room");
    }
    result.into_boxed()
}
fn with(attributes: &[Value]) -> Box<[u8]> {
    raw(1, &entry_values(b"a", 1, 0o755, (0, 0), attributes, &[Value::None]))
}

#[test]
fn an_entrys_attributes_are_bounded_and_in_order() {
    accepted(&[&with(&pairs(64, b"user.", 1)), &end()], "64 attributes");
    refused(&[&with(&pairs(65, b"user.", 1)), &end()], Unreadable::Limit(b"xattrs"), "65 attributes");
    accepted(&[&with(&pairs(1, &[b'n'; 252], 65_536)), &end()], "a name of 255 and a value of 65536");
    refused(&[&with(&pairs(1, &[b'n'; 253], 1)), &end()], Unreadable::Limit(b"key"), "a name of 256");
    refused(&[&with(&pairs(1, b"user.", 65_537)), &end()], Unreadable::Limit(b"value"), "a value of 65537");
    // Two names the wrong way round, and the same twice, as bytes: the encoder will not make them.
    let two = written(&entry_item(owned(dir(b"a"), 0, 0, &[(b"b", b""), (b"c", b"")])));
    let at = bytes::find(&two, b"\x04\x00\x00\x00\x00\x00\x00\x00\x01b").expect("the first name") + 9;
    let mut swapped = bytes::copy_of(&two);
    swapped[at] = b'd';
    refused(&[&swapped, &end()], Unreadable::Body(Malformed::Order), "names out of order");
    swapped[at] = b'c';
    refused(&[&swapped, &end()], Unreadable::Body(Malformed::Order), "a name twice");
}

/// The item with its value cut by `by` bytes at the end, its length made good.
fn cut(item: &[u8], by: usize) -> Box<[u8]> {
    let value = item.get(5..item.len().checked_sub(by).expect("a value to cut")).expect("a value to cut");
    let length = u32::try_from(value.len()).expect("small");
    joined(&[&item[..1], &length.to_be_bytes(), value])
}
/// The item with bytes after its value, its length made good.
fn extended(item: &[u8], extra: &[u8]) -> Box<[u8]> {
    let value = joined(&[&item[5..], extra]);
    let length = u32::try_from(value.len()).expect("small");
    joined(&[&item[..1], &length.to_be_bytes(), &value])
}

#[test]
fn an_item_with_too_few_too_many_or_mistaken_values_is_unreadable() {
    let local = [Value::List(5), uint(5), uint(1), uint(2), uint(3), uint(4)];
    let head =
        [Value::List(2), Value::List(7), name(b"a"), uint(1), uint(0), uint(0), uint(0), Value::Map(0), Value::None];
    let short_local = [Value::List(4), uint(5), uint(1), uint(2), uint(3)];
    let late_nanoseconds = [Value::List(5), uint(5), uint(1), uint(4_294_967_296), uint(3), uint(4)];
    let whole = raw(1, &values(&[&head, &local]));
    refused(&[&raw(1, &[uint(1)]), &end()], Unreadable::Shape(b"entry"), "an entry that is a number");
    refused(&[&raw(1, &[Value::List(1), uint(1)]), &end()], Unreadable::Shape(b"entry"), "an entry of one");
    refused(&[&raw(1, &values(&[&head, &short_local])), &end()], Unreadable::Shape(b"local"), "a short local");
    refused(
        &[&raw(1, &values(&[&head, &late_nanoseconds])), &end()],
        Unreadable::Limit(b"ctime"),
        "nanoseconds past a u32",
    );
    refused(&[&extended(&whole, &[0]), &end()], Unreadable::Body(Malformed::Trailing), "a value after the local");
    refused(&[&cut(&whole, 1), &end()], Unreadable::Body(Malformed::Truncated), "a value that stops short");
    refused(&[&cut(&whole, 30), &end()], Unreadable::Body(Malformed::Truncated), "a value that stops shorter");
    refused(&[&raw(2, &[Value::List(2), uint(3), name(b"a")]), &end()], Unreadable::Shape(b"op"), "an operation of 3");
    refused(&[&raw(2, &[Value::List(2), uint(0), name(b"a")]), &end()], Unreadable::Shape(b"op"), "an operation of 0");
    refused(
        &[&raw(2, &[Value::List(3), uint(1), name(b"a"), Value::None]), &end()],
        Unreadable::Limit(b"change"),
        "a change of three",
    );
    refused(
        &[&raw(2, &[Value::List(2), name(b"a"), uint(1)]), &end()],
        Unreadable::Shape(b"op"),
        "a change the wrong way round",
    );
}

fn end_values(without: &[u8], boot: &[Value], parent: &[Value]) -> Box<[Value]> {
    let counts = [
        Value::Map(6),
        name(b"bytes"),
        uint(0),
        name(b"deletes"),
        uint(0),
        name(b"entries"),
        uint(1),
        name(b"files"),
        uint(0),
        name(b"hashed"),
        uint(0),
        name(b"puts"),
        uint(1),
    ];
    let keep = if without.is_empty() { 6_u64 } else { 5 };
    let mut result = List::with_capacity(256);
    result.push(Value::Map(keep)).expect("room");
    let fields: [(&[u8], Box<[Value]>); 6] = [
        (b"boot", boot.into()),
        (b"changes", Box::from([Value::Digest(d(3))])),
        (b"counts", counts.into()),
        (b"key", Box::from([Value::Digest(d(1))])),
        (b"parent_tree", parent.into()),
        (b"tree", Box::from([Value::Digest(d(2))])),
    ];
    for (key, value) in fields {
        if key != without {
            result.push(name(key)).expect("room");
            for item in &*value {
                result.push(item.clone()).expect("room");
            }
        }
    }
    result.into_boxed()
}
fn boot(count: u64, versions: &[&[u8]]) -> Box<[Value]> {
    let mut result = List::with_capacity(256);
    result.push(Value::List(count)).expect("room");
    for version in versions {
        for value in &*kernel_values(version) {
            result.push(value.clone()).expect("room");
        }
    }
    result.into_boxed()
}
fn kernel_values(version: &[u8]) -> Box<[Value]> {
    Box::from([
        Value::Map(4),
        name(b"initramfs"),
        Value::Digest(d(3)),
        name(b"kernel"),
        Value::Digest(d(1)),
        name(b"modules"),
        Value::Digest(d(2)),
        name(b"version"),
        name(version),
    ])
}

#[test]
fn an_end_item_with_a_missing_unknown_or_mistaken_field_is_unreadable() {
    let nothing = [Value::List(0)];
    let none = [Value::None];
    accepted(&[&raw(3, &end_values(b"", &nothing, &none))], "the fixture reads");
    let fields: [&'static [u8]; 6] = [b"boot", b"changes", b"counts", b"key", b"parent_tree", b"tree"];
    for field in fields {
        refused(&[&raw(3, &end_values(field, &nothing, &none))], Unreadable::Missing(field), "a field missing");
    }
    refused(
        &[&raw(3, &end_values(b"", &nothing, &[uint(1)]))],
        Unreadable::Shape(b"parent_tree"),
        "a parent that is a number",
    );
    refused(&[&raw(3, &[Value::List(0)])], Unreadable::Shape(b"end"), "an end that is a list");
    let unknown = values(&[&[Value::Map(1), name(b"zzz"), Value::None]]);
    refused(&[&raw(3, &unknown)], Unreadable::Unknown, "a field the version does not name");
    accepted(&[&raw(3, &end_values(b"", &boot(2, &[b"6.1", b"6.12"]), &none))], "two kernels in order");
    refused(
        &[&raw(3, &end_values(b"", &boot(2, &[b"6.12", b"6.1"]), &none))],
        Unreadable::Order(b"boot"),
        "kernels out of order",
    );
    refused(
        &[&raw(3, &end_values(b"", &boot(2, &[b"6.1", b"6.1"]), &none))],
        Unreadable::Order(b"boot"),
        "a kernel twice",
    );
    let versions: [&[u8]; 9] = [b"1", b"2", b"3", b"4", b"5", b"6", b"7", b"8", b"9"];
    accepted(&[&raw(3, &end_values(b"", &boot(8, &versions[..8]), &none))], "eight kernels");
    refused(&[&raw(3, &end_values(b"", &boot(9, &versions), &none))], Unreadable::Limit(b"boot"), "nine kernels");
    refused(
        &[&raw(3, &end_values(b"", &boot(1, &[&[b'v'; 256]]), &none))],
        Unreadable::Limit(b"version"),
        "a version of 256",
    );
    accepted(&[&raw(3, &end_values(b"", &boot(1, &[&[b'v'; 255]]), &none))], "a version of 255");
}

// What the writer refuses.

fn refuses(item: &Item, limits: &Limits, expected: Refused, what: &str) {
    assert_eq!(tree::item(item, limits), Err(expected), "{what}");
}
fn entry_item(entry: Entry) -> Item {
    Item::Entry(entry, local())
}

#[test]
fn the_writer_refuses_what_the_reader_would_not_take() {
    let limits = SHIPPED;
    refuses(&entry_item(dir(b"a//b")), &limits, Refused::Limit(b"path"), "an invalid path");
    refuses(&entry_item(dir(&[b'a'; 4_097])), &limits, Refused::Limit(b"path"), "a long path");
    refuses(&entry_item(entry(b"a", 0o10000, Payload::Fifo)), &limits, Refused::Limit(b"mode"), "a mode");
    refuses(&entry_item(owned(dir(b"a"), 65_536, 0, &[])), &limits, Refused::Limit(b"uid"), "a uid");
    refuses(&entry_item(owned(dir(b"a"), 0, 65_536, &[])), &limits, Refused::Limit(b"gid"), "a gid");
    refuses(
        &entry_item(entry(b"a", 0, Payload::Symlink { target: text(&[b'x'; 4_097]) })),
        &limits,
        Refused::Limit(b"target"),
        "a target",
    );
    refuses(
        &entry_item(entry(b"a", 0, Payload::Hardlink { path: text(b"/x") })),
        &limits,
        Refused::Limit(b"target"),
        "a link's path",
    );
    refuses(
        &entry_item(owned(dir(b"a"), 0, 0, &[(b"b", b""), (b"a", b"")])),
        &limits,
        Refused::Order(b"xattrs"),
        "names out of order",
    );
    refuses(
        &entry_item(owned(dir(b"a"), 0, 0, &[(b"a", b""), (b"a", b"")])),
        &limits,
        Refused::Order(b"xattrs"),
        "a name twice",
    );
    let name: &[u8] = &[b'n'; 256];
    let value = bytes::zeroed(65_537);
    refuses(&entry_item(owned(dir(b"a"), 0, 0, &[(name, b"")])), &limits, Refused::Limit(b"key"), "a long name");
    refuses(&entry_item(owned(dir(b"a"), 0, 0, &[(b"a", &value)])), &limits, Refused::Limit(b"value"), "a long value");
    let mut many = List::with_capacity(65);
    for number in 0..65_u8 {
        many.push(Xattr { name: text(&[b'u', number]), value: text(b"") }).expect("room");
    }
    let too_many = Entry { xattrs: many.into_boxed(), ..dir(b"a") };
    refuses(&entry_item(too_many), &limits, Refused::Limit(b"xattrs"), "65 attributes");
    refuses(
        &Item::Change { op: Op::Put, path: text(b"a/") },
        &limits,
        Refused::Limit(b"path"),
        "a change at a bad path",
    );
    let small = Limits { tree: TreeLimits { item: 100, ..limits.tree }, ..limits };
    refuses(&entry_item(dir(b"a")), &small, Refused::Limit(b"item"), "an item past the bound");
    let shallow = Limits { depth: 2, ..limits };
    refuses(&entry_item(dir(b"a")), &shallow, Refused::Limit(b"depth"), "a decoder too shallow for an entry");
    refuses(&an_end_with_a_kernel(), &shallow, Refused::Limit(b"depth"), "or for a kernel");
    assert!(tree::item(&an_end(None, Box::from([])), &shallow).is_ok(), "an end with no kernel needs two");
    let narrow = Limits { tree: TreeLimits { key: 10, ..limits.tree }, ..limits };
    refuses(&an_end_with_a_kernel(), &narrow, Refused::Limit(b"key"), "a key too short for parent_tree");
    let mut nine = List::with_capacity(9);
    for number in 0..9_u8 {
        nine.push(a_kernel(&[b'1' + number])).expect("room");
    }
    refuses(&an_end(None, nine.into_boxed()), &limits, Refused::Limit(b"kernels"), "nine kernels");
    refuses(
        &an_end(None, Box::from([a_kernel(b"2"), a_kernel(b"1")])),
        &limits,
        Refused::Order(b"boot"),
        "kernels out of order",
    );
    refuses(
        &an_end(None, Box::from([a_kernel(&[b'v'; 256])])),
        &limits,
        Refused::Limit(b"version"),
        "a version of 256",
    );
}
