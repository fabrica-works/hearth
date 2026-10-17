# Session 09: `hearth-protocol-tree` (M1): the frame, 3.1's decoder, kinds 1, 4 and 5

`crates/hearth-protocol-tree`, a lib-role machine on skein-lib and
hearth-lib, reads and writes the records of M1's store without layers
(store.md, 2.4): `store.record` (kind 1), resolutions (4) and cache files'
records (5), in a frame (6.1), through 3.1's decoder of hearth 01's `Item`s.
Its machine world and worst cases are hearth 39's. Nothing calls it here:
hearth 23 connects it (the store's io; hearth 33 its kinds 4 and 5), hearth
31 its kind 4.

## Read first

hearth's store.md 2.4, 3.1, 3.2, 5.3, 6.1, 6.4 to 6.6, 6.8, 8.2 step 6;
render.md 4.2 (`packages`), 5.1, 5.3, 5.4, 8.3; testing.md 2.4.
`hearth/01-lib.md`: its Choices, and 01.1's Shape and sketches.
code-organization.md 1 to 4, 6, 7; README, Rules ("hearth's first
landings", "Format versions", "Shared files", "Test time"); decisions.md 9,
10; `findings/skein-io.md` 3; `findings/skein-testing.md` 2, 3. skein's
programming-model.md 3, 7, 8, 10 and testing-strategy.md 2.1, 2.4, 6, by
number only.

## What the design asks

- **The frame** (store.md, 6.1): the length "checked against the kind's bound
  first"; "reserved u32 zero"; past a bound or limit, a record "of a version
  the builder cannot read (6.8)".
- **Decoding** (3.1): "every length and count is checked against a limit
  before anything is set aside, and nesting uses an explicit stack"; "One
  value has one encoding"; render.md, 8.3: "no recursion and no unbounded loop".
- **Kinds 1 and 4** (6.6, 6.4; render.md, 5.3): no `range` without layers
  (2.4); `inputs` "as the packages part holds them"; the closure "by name
  then architecture"; "Every requested package must be in the closure".
- **Versions** (6.8): "a release reads its own version of each kind and the
  previous one".

## Choices this plan makes

- **Read and written whole:** hearth 23 and 33 `Load` a record (at most
  `frame::largest`) and `Store` it, one buffer (`findings/skein-io.md`, 3);
  the kinds are functions over bytes. M2's streamed tree files (6.3) add a
  machine.
- **A pull decoder of 01's `Item`s:** one item a call (a map's key as
  `Item::Bytes`), `Malformed` for each shape 01's encoder asserts; each
  kind's reader pulls its fields' items in key order: no tree, no recursion.
  **A strict frame,** version 1 only (M1 has no previous one).
- **Every field 6.4 to 6.6 give at version 1, `range` included:** M1 writes
  none (2.4) but reads one, so an M2 store's record is never "written again"
  without it (6.8). Nobody extends these kinds (code-organization.md, 3).
- **`inputs` are kept as items:** each of the four values as read, in
  pre-order, only its first item's kind checked; hearth 33's scan recomputes
  the key as `H::new(b"resolution", 4)` with each through `Encoder::item`, so
  the packages part has one encoding, render's (hearth 17). The rest is typed.
- **Orders are checked on read and refused on write:** the closure by name
  then architecture, none repeated; the request a set; each `source` a
  line's index. Whatever this crate writes reads back.
- **`made.costs` is `{ fetched, took }`,** bytes and ns (render.md, 5.5: "each
  run's duration and bytes"). **Kind 4's bounds 6.1 leaves:** a line's `uri`
  4,096 ("a path"); `suite`, `components` 256 ("a name"). **An index is
  `list [ path, size, sha256 ]`,** as store.md 6.4, not render.md 5.4's map.
  **The InRelease's entry in `indices` is `[ "InRelease", its size, its
  SHA-256 ]`,** its path relative to the suite's directory; hearth 31 writes
  it, the decoder takes any bytes. **A blob's `oid` is git's hex text,** as
  hearth 02's listing carries it. (store.md 6.4, 6.1; render.md 5.4.)
- **The decoder's own bounds,** which no valid record reaches: depth 8 (the
  deepest field, a line's index, is at 5; never past 01's `DEPTH`); a key of
  64 bytes; 8,256 items for the inputs (the request's 8,192 names, render.md
  5.3, and 64); an `oid` of 64 bytes (render.md, 2.2). (store.md 6.1.)

## Rules that bind it

- **No recursion and no unbounded loop** (render.md, 8.3): each `for` runs
  over a count already checked against a limit, or the stack's depth;
  nesting is a `skein_lib::Stack` of `limits.depth` slots. Clippy cannot see
  it: unchecked, review only.
- **Format versions** (README, Rules): the frame and kinds 1, 4, 5 at
  version 1 are 09.1's (`Kind::version`); nothing here moves them.
- **Literals and the shipped profile:** expected bytes are store.md 3.2's
  vectors, or computed once by hand from 3.1's table; limits come from
  `limits::SHIPPED`, another named beside it with its reason.

## Increments

### 09.1 The crate, the frame and 3.1's decoder

Scope: the crate; the frame (6.1), checked and written; 3.1's decoder of
01's `Item`s; `src/limits.rs`, whose `SHIPPED` holds 6.1's defaults and the
plan's. 09.1 owns the frame and kinds 1, 4 and 5 at version 1.
Needs: hearth 01.1's value model (`hearth_lib::encode::{Item, Digest,
Encoder}`).
Files: the root `Cargo.toml` and `Cargo.lock` (shared: README, "Shared
files"): `members` gains the crate, `[workspace.dependencies]` gains
`hearth-lib = { path = "crates/hearth-lib" }` unless landed; new `crates/hearth-protocol-tree/Cargo.toml` (hearth-lib's, plus
`hearth-lib.workspace = true`), `src/{lib.rs, limits.rs, frame.rs,
decode.rs, tests.rs}` and `src/tests/{fixtures.rs, frame.rs}`. No users at
the base.
Shape:
- `lib.rs` as 01's (`no_std` but in tests, `forbid(unsafe_code)`).
  `frame::body` checks with a `Reader`, in order: 24 bytes (`Short`),
  `MAGIC`, the kind asked for, `Kind::version`, reserved zero, the length
  against `Kind::bound` (`Bound`), then against the bytes after the header
  (`Length`). `frame::write`: `Writer::new(24 + len)`, or `Refused::Bound`;
  `largest` is 24 plus the bound.
- `Decoder::item`, one item a call, with no loop but a `settle` of its own, as
  01's (a bounded `for` over the depth): a tag `00` to `07` (else `Tag`); a uint's 8 bytes or a digest's 32;
  a byte string's length checked against `longest` (`limits.key` where a key
  belongs: `Long`) and the bytes left (`Truncated`) before `bytes::copy_of`;
  a count checked against the bytes left (`Truncated`), then pushed (`Deep`
  when full); where a key belongs, bytes (`Key`) above the previous key,
  kept as a copy as 01 keeps it (`Order`); once the value is whole, `None`,
  or `Trailing` if bytes remain.
Test shape: unit tests, each under 20 ms; the lints hold in tests (no `Vec`,
no `matches!`). New `src/tests/fixtures.rs`: `SHIPPED`; `hex(&str) ->
Box<[u8]>`, spaces skipped, as 01's; `items(body, &Limits) ->
Result<Box<[Item]>, Malformed>`, the decoder drained into a `List`. The
reference test, in `src/tests/frame.rs` (the frame's and decoder's tests):

```rust
use super::fixtures::{SHIPPED, hex};
use crate::frame::{self, Kind};
#[test]
fn a_framed_body_has_the_header_store_md_gives_byte_for_byte() {
    let body = hex("07 0000000000000001 04 0000000000000001 61 03 0000000000000001");
    let framed = frame::write(Kind::Store, &body, &SHIPPED).expect("28 bytes are within 4 KiB");
    let expected = hex(concat!("6865617274680072 0001 0001 00000000 000000000000001c ", // magic, kind 1, version 1, reserved, 28
        "07 0000000000000001 04 0000000000000001 61 03 0000000000000001"));
    assert_eq!(framed, expected, "store.md, 6.1's header around 3.2's map {{ \"a\": uint 1 }}");
}
```

The others: `store_md_3_2s_vectors_decode_to_their_items` (its two: `[List(2),
Bytes("hearth/1"), Bytes("empty")]` and `[Map(1), Bytes("a"), Uint(1)]`);
`a_length_past_the_kinds_bound_is_refused_before_the_body`
(a kind 1 header claiming 4,097, no body: `Bound`, not `Length`);
`a_header_unlike_store_md_6_1s_makes_the_frame_unreadable` (rows: 23 bytes,
the magic, kind 5 read as 1, version 2, reserved 1, the length +1 and -1); `a_body_past_the_kinds_bound_is_refused_when_framed`;
`bytes_that_are_not_one_value_of_store_md_3_1_are_malformed` (rows: tag
`08`, a key as a uint, a repeated key, decreasing keys, a trailing byte, a
count of `u64::MAX` in 9 bytes, a body ending inside a uint);
`nesting_one_past_the_depth_limit_is_too_deep`;
`a_bytes_item_past_its_longest_is_long`.
Proof: unit layer, no higher than the tests: nothing calls this machine
until hearth 23 and 33, whose store io read and write `store.record` and
cache files' records through it (31: resolutions). Run once: `heavy
cargo nextest run -p hearth-protocol-tree -E
'test(a_framed_body_has_the_header_store_md_gives_byte_for_byte)'`; it shows
a body framed byte for byte as 6.1 gives it. Bound: 0.02 s.
Don't: build a tree or recurse; read a stream with demands; compute `H` (01.2
is not a Need); give kinds 2, 3 or 6 to 9 a body; depend on anything but
skein-lib and hearth-lib.
Size: at most 280 lines.
Removes: nothing.

### 09.2 Kinds 1 and 5

Scope: `store.record` (6.6) and `cache/<sha256>.record` (6.5), read and
written; a crate-private reader of a map's fields in key order, shared by
the three kinds; `Unreadable`'s field variants and `Refused::Limit`.
Needs: nothing beyond 09.1.
Files: new `crates/hearth-protocol-tree/src/{fields.rs, store.rs,
cache_file.rs}`; `src/lib.rs`, `src/frame.rs`; `src/tests.rs`,
`src/tests/fixtures.rs`, new `src/tests/records.rs`.
Shape:
- `fields.rs` pulls from a `Decoder`: `map`, `key`, `uint`, `id` (32 bits,
  else `Limit`), `bytes(limit)` (`Long` becomes `Limit`), `digest`,
  `list(limit)`, `wall`, `duration`. A record's reader takes `Map(n)`, then
  up to `n` keys, meeting the kind's names in the sketches' key order: a
  name reached fills its slot; a key that names no field is `Unknown`
  (store.md 6.1, 6.8); a required slot left empty is `Missing`; a value of
  another kind is `Shape`.
- Writing checks each limit (`Refused::Limit`), then runs one `encode(&record,
  &mut Encoder)` through `Encoder::measuring()`, then `Encoder::writing(len)`,
  then `frame::write` (01's "One encoder, three sinks"). `Origin` is a map
  whose first key is `kind`; a name other than `fetch`, `index`, `blob` or
  `rendered` is `Shape(b"origin")`.
Test shape: the same fixtures, plus `frame_of(kind, &[Item]) -> Box<[u8]>`,
which frames items written by hearth-lib's encoder, for records this crate's
writer refuses. The reference test, in `src/tests/records.rs`:

```rust
use {super::fixtures::{SHIPPED, hex}, crate::store::{self, Store}, skein_lib::Wall};
#[test]
fn a_store_record_encodes_its_fields_in_key_order_byte_for_byte() {
    let record = Store { layout: 1, made: Wall::from_nanos(1_760_000_000_000_000_000), range: None, uid: 1000 };
    let expected = hex(concat!(
        "6865617274680072 0001 0001 00000000 000000000000004c 07 0000000000000003 ",
        "04 0000000000000006 6c61796f7574 03 0000000000000001 ", // layout: 1
        "04 0000000000000004 6d616465 03 186cc6acd4b00000 ",      // made
        "04 0000000000000003 756964 03 00000000000003e8",         // uid: 1000; no range (2.4)
    ));
    assert_eq!(store::write(&record, &SHIPPED), Ok(expected), "store.md, 6.6's fields, keys in byte order");
}
```

The others: `a_cache_file_record_encodes_its_fields_in_key_order_byte_for_byte`
(kind 5's header and 6.5's body, so an M2 builder reads this store as its
own, 2.4: `CacheFile { at: 1_760_000_000_000_000_000 ns, origin: Rendered {
path: b"tasks/core/tree/etc/motd.j2" }, sha256: <the empty file's>, size: 0,
took: 1.5 ms }`, 224 bytes after the header, worked out once from 3.1); `a_store_record_with_a_range_reads_back_with_it`;
`each_origin_of_a_cache_file_reads_back_as_its_variant` (a table of the
four); `a_body_that_is_one_value_but_no_map_is_no_record` (rows: `list []`
and `uint 1`, framed as kinds 1 and 5 by `frame_of`: `Unreadable::Record`);
`an_origin_of_another_kind_is_unreadable` (`copied`);
`a_record_without_a_required_field_names_it_missing` (no `took`);
`a_field_its_version_does_not_name_makes_it_unreadable`;
`a_uri_past_4096_bytes_is_unreadable`;
`writing_a_cache_file_past_a_limit_is_refused` (a `rendered` path of 4,097
bytes).
Proof: unit layer, for 09.1's reason; hearth 23 and 33 first write these
kinds. Run once: `heavy cargo nextest run -p hearth-protocol-tree -E
'test(a_store_record_encodes_its_fields_in_key_order_byte_for_byte)'`; it
shows M1's `store.record` as 6.6 names it, keys in byte order, no `range`.
Bound: 0.02 s.
Don't: decide what a `layout` other than 1 means (store.md 6.8: it is
written again, hearth 23's step 2; read here as a uint); compare
`sha256` with the file's name (the io's); validate UTF-8.
Size: at most 250 lines.
Removes: nothing.

### 09.3 Kind 4

Scope: `resolutions/<key>.record` (6.4; render.md, 5.4), read and written:
`inputs` as items, `sources`, the closure, `digest` and `key` as written,
`made`; its limits (8,192 packages, 8 lines, 16 indices a line, the path and
name limits) and orders (`Unreadable::Order`, `Refused::Order`).
Needs: nothing beyond 09.2.
Files: new `crates/hearth-protocol-tree/src/resolution.rs`; `src/lib.rs`,
`src/frame.rs`, `src/fields.rs` (`value`); `src/tests/fixtures.rs`,
`src/tests/records.rs`.
Shape:
- `fields::value` copies one whole value's items into a `List<Item>` of
  `limits.resolution.inputs`, counting the items still owed (1, then `n` more
  for `List(n)` and `2n` for `Map(n)`), never past the list's capacity
  (`Limit`). Inputs: `apt` and `sources` start with `Map`; `request` is a
  `List` of names of at most `name` bytes, strictly increasing
  (`Order(b"request")`); `variant` is bytes of at most `name`.
- A line is `List(4)`, an index `List(3)`, a package `List(7)`, each count
  checked against its limit before `List::with_capacity`. The closure runs
  by name then architecture (`Order(b"closure")`); each `source` must be
  below the line count (`Limit(b"source")`), checked once `sources`, later
  in key order, is read.
Test shape: the same fixtures, plus `a_resolution(packages: u32)` (packages
`p00000` and on, on `amd64`; one line of two indices, `InRelease` and
`main/binary-amd64/Packages`; a `forky` `main` part's inputs) and
`resolution_items(…)`, the same as items, for what the writer refuses. The tests, in `src/tests/records.rs`:
`a_resolution_reads_back_as_it_was_written_inputs_included`
(`a_resolution(2)` written, then read, equals itself: the key's values come
back item for item); `a_closure_past_its_package_limit_is_unreadable` and
`writing_a_closure_past_its_package_limit_is_refused` (`packages: 4` beside
the shipped 8,192, a closure of 5: five show the bound as 8,193 do, at a
unit test's cost); `a_closure_out_of_name_then_architecture_order_is_unreadable`
(rows: names descending, one name's architectures descending, a package
repeated); `a_package_whose_source_names_no_line_is_unreadable`;
`a_resolution_field_one_past_its_limit_is_unreadable` (6.1's and the plan's
limits, a row each, on `resolution_items`: 9 lines, 17 indices, an index's
path, a filename and a `uri` of 4,097 bytes, a package's name, version and
architecture and a `suite` of 257; each `Limit` naming its field) and
`writing_a_resolution_field_one_past_its_limit_is_refused` (the same rows on
`a_resolution(2)`: `Refused::Limit`); `a_request_that_is_not_a_set_is_unreadable`;
`an_input_of_another_kind_than_its_own_is_unreadable` (a uint `variant`);
`a_resolution_frame_starts_with_kind_4s_header` (`a_resolution(2)` written:
its first 16 bytes are `6865617274680072 0004 0001 00000000`, the next 8 the
length of the bytes after them).
Proof: unit layer, for 09.1's reason; hearth 31 records resolutions and
hearth 33's scan reads them. Run once: `heavy cargo nextest run -p
hearth-protocol-tree -E
'test(a_resolution_reads_back_as_it_was_written_inputs_included)'`; it shows
the key's four values back item for item, which the scan recomputes the key
from. Bound: 0.02 s.
Don't: type the inputs past their first item; compute or check `key` or
`digest` (store.md 6.4: hearth 33's scan recomputes both, and deletes a
record whose key or digest does not recompute); check that the InRelease
is first; resolve a
`source` into a URI.
Size: at most 280 lines.
Removes: nothing.

## Interface sketches

No state between calls. Assumed: hearth 01.1's `hearth_lib::encode::{Item,
Digest, Encoder}`, `Item` deriving `Hash`. A variant's doc trails it.

```rust
// crates/hearth-protocol-tree/src/limits.rs (09.1; worst_case and held: hearth 39.1)
/// The records' configured limits (store.md, 6.1; programming-model.md, 7).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)] pub struct Limits { pub depth: u32, pub key: u32, pub store: StoreLimits, pub resolution: ResolutionLimits, pub cache_file: CacheFileLimits }
/// Kind 1's body.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)] pub struct StoreLimits { pub body: u32 }
/// Kind 4's body and the limits within it; `inputs` counts the four values' items.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)] pub struct ResolutionLimits { pub body: u32, pub packages: u32, pub lines: u32, pub indices: u32, pub path: u32, pub name: u32, pub inputs: u32 }
/// Kind 5's body, a URI or a path, and an object id.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)] pub struct CacheFileLimits { pub body: u32, pub path: u32, pub oid: u32 }
/// The shipped defaults: store.md 6.1's, and the plan's where 6.1 gives none.
pub const SHIPPED: Limits = Limits { depth: 8, key: 64, store: StoreLimits { body: 4_096 },
    resolution: ResolutionLimits { body: 4_194_304, packages: 8_192, lines: 8, indices: 16, path: 4_096, name: 256, inputs: 8_256 },
    cache_file: CacheFileLimits { body: 8_192, path: 4_096, oid: 64 } };
```

```rust
// crates/hearth-protocol-tree/src/frame.rs (09.1; field variants 09.2, Order 09.3) and src/decode.rs (09.1)
/// The frame's first eight bytes (store.md, 6.1).
pub const MAGIC: [u8; 8] = *b"hearth\0r";
/// The kinds of record M1 reads and writes.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)] pub enum Kind {
    Store,      // 1: `store.record` (6.6)
    Resolution, // 4: `resolutions/<key>.record` (6.4)
    CacheFile,  // 5: `cache/<sha256>.record` (6.5)
}
impl Kind {
    #[must_use] pub const fn number(self) -> u16;
    /// 1 for each kind: the version this release reads and writes. Owner: hearth 09.1.
    #[must_use] pub const fn version(self) -> u16;
    #[must_use] pub const fn bound(self, limits: &Limits) -> u32;
}
/// Why a record is of a version the builder cannot read (6.1, 6.8): the store treats it as absent.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)] pub enum Unreadable {
    Short,                              // fewer than the header's 24 bytes
    Magic,                              // the first eight bytes are not MAGIC
    Kind { found: u16 },                // another kind than the one read
    Version { found: u16 },             // a version this release does not read
    Reserved,                           // the reserved word is not zero
    Bound { length: u64 },              // the length past the bound, found before the body
    Length { length: u64, after: u64 }, // the length is not the bytes after the header
    Body(Malformed),                    // the body is not one value of 3.1
    Record,                             // a value, but not a map of named fields
    Missing(&'static [u8]),             // a field the record needs is absent
    Shape(&'static [u8]),               // a field holds another kind of value than its own
    Limit(&'static [u8]),               // a field past its limit
    Order(&'static [u8]),               // a field out of its record's order, or repeated
    Unknown,                            // a field the kind's version does not name (store.md 6.1)
}
/// Why a record is not written: a reader would refuse it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)] pub enum Refused {
    Bound { length: u64 }, // the body would pass the kind's bound
    Limit(&'static [u8]),  // a field would pass its limit
    Order(&'static [u8]),  // a field would be out of its record's order
}
/// The body of a frame of `kind`, its header checked in 09.1's order.
pub fn body<'a>(bytes: &'a [u8], kind: Kind, limits: &Limits) -> Result<&'a [u8], Unreadable>;
/// `body` framed as `kind` at its version.
pub fn write(kind: Kind, body: &[u8], limits: &Limits) -> Result<Box<[u8]>, Refused>;
/// The most bytes a file of `kind` may hold: the `max` of hearth 23's `Load`.
#[must_use] pub fn largest(kind: Kind, limits: &Limits) -> u64;

/// Why bytes are not one value of store.md, 3.1: a shape 01's encoder asserts, or a cut.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)] pub enum Malformed {
    Truncated, // the body ends inside a value, or a count or length passes the bytes left
    Trailing,  // bytes after the one value
    Tag(u8),   // a first byte that is none of the seven kinds'
    Deep,      // a container opened past the depth limit
    Key,       // a map's key not encoded as bytes
    Order,     // a map's key not above the one before it, bytewise
    Long,      // bytes, or a key, longer than the reader allows
}
/// One body's value, item by item in pre-order, its shape on a stack of `limits.depth`.
#[derive(Debug)] pub struct Decoder<'a> { reader: Reader<'a>, open: Stack<Open>, key: u32, whole: bool }
impl<'a> Decoder<'a> {
    #[must_use] pub fn new(body: &'a [u8], limits: &Limits) -> Decoder<'a>;
    /// The next item, bytes at most `longest` (a key: `limits.key`); `None` once whole and read to its end.
    pub fn item(&mut self, longest: u32) -> Result<Option<Item>, Malformed>;
}
```

```rust
// src/store.rs, src/cache_file.rs (09.2) and src/resolution.rs (09.3): what hearth 23 and 24 hold,
// field by field; map fields in key order, lists by position
/// `store.record`, kind 1, version 1 (store.md, 6.6).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)] pub struct Store { pub layout: u64, pub made: Wall, pub range: Option<Range>, pub uid: u32 }
/// `range`: the subordinate range, absent in a store without layers (2.4); M1 writes none.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)] pub struct Range { pub base: u32, pub count: u32 }
/// `cache/<sha256>.record`, kind 5, version 1 (store.md, 6.5).
#[derive(Clone, PartialEq, Eq, Hash, Debug)] pub struct CacheFile { pub at: Wall, pub origin: Origin, pub sha256: Digest, pub size: u64, pub took: Duration }
/// Where a cache file first came from: a map of its field and `kind`, its name.
#[derive(Clone, PartialEq, Eq, Hash, Debug)] pub enum Origin {
    Fetch { uri: Box<[u8]> },     // `fetch`: fetched from `uri` (build.md, 5.1)
    Index { resolution: Digest }, // `index`: kept by the resolution of this key (6.4)
    Blob { oid: Box<[u8]> },      // `blob`: a static file streamed from git, by its object id
    Rendered { path: Box<[u8]> }, // `rendered`: by its template's path in the revision
}
/// `resolutions/<key>.record`, kind 4, version 1 (store.md, 6.4); `key` and `digest` as written.
#[derive(Clone, PartialEq, Eq, Hash, Debug)] pub struct Resolution { pub closure: Box<[Package]>, pub digest: Digest, pub inputs: Inputs, pub key: Digest, pub made: Made, pub sources: Box<[Line]> }
/// The key's values as the packages part holds them (render.md, 4.2, 5.1): each value's items, as read.
#[derive(Clone, PartialEq, Eq, Hash, Debug)] pub struct Inputs { pub apt: Box<[Item]>, pub request: Box<[Item]>, pub sources: Box<[Item]>, pub variant: Box<[Item]> }
/// A source line, `list [ uri, suite, components, indices ]`, in the sources' order.
#[derive(Clone, PartialEq, Eq, Hash, Debug)] pub struct Line { pub uri: Box<[u8]>, pub suite: Box<[u8]>, pub components: Box<[Box<[u8]>]>, pub indices: Box<[Index]> }
/// An index kept in the cache, `list [ path, size, sha256 ]`, its line's `InRelease` first.
#[derive(Clone, PartialEq, Eq, Hash, Debug)] pub struct Index { pub path: Box<[u8]>, pub size: u64, pub sha256: Digest }
/// A package, `list [ name, architecture, version, source, filename, size, sha256 ]`; `source` a line's index.
#[derive(Clone, PartialEq, Eq, Hash, Debug)] pub struct Package { pub name: Box<[u8]>, pub architecture: Box<[u8]>, pub version: Box<[u8]>, pub source: u32, pub filename: Box<[u8]>, pub size: u64, pub sha256: Digest }
/// `made`: apt's version, when, the run's costs, mmdebstrap's version; outside the digest.
#[derive(Clone, PartialEq, Eq, Hash, Debug)] pub struct Made { pub apt: Box<[u8]>, pub at: Wall, pub costs: Costs, pub mmdebstrap: Box<[u8]> }
/// `costs`: bytes fetched, and the run's duration.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)] pub struct Costs { pub fetched: u64, pub took: Duration }
// store::{read, write}, and cache_file:: and resolution:: alike over their own record:
pub fn read(bytes: &[u8], limits: &Limits) -> Result<Store, Unreadable>;
pub fn write(record: &Store, limits: &Limits) -> Result<Box<[u8]>, Refused>;
```

## Questions to raise when reached

None.

## Done when

Each runs once, reading hearth's `origin/main`:
1. `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-09\.[1-3]'` prints 3.
2. In a worktree at `origin/main`, `heavy cargo nextest run -p
   hearth-protocol-tree` passes, its `Summary` naming at least 28 tests.
3. `git -C ~/src/rust/hearth grep -n -F 'hearth\0r' origin/main -- crates`
   prints one line, in `crates/hearth-protocol-tree/src/frame.rs`; in that
   worktree, `cargo tree -p hearth-protocol-tree -e normal --depth 1`
   prints three: `hearth-protocol-tree`, `hearth-lib`, `skein-lib`.
