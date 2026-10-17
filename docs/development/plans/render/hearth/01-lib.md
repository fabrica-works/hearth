# Session 01: `hearth-lib`: the canonical encoding, then `H` and the vectors

hearth's first crate and its one crate in the lib role (README.md, 4.2):
`crates/hearth-lib`, `no_std`, on skein-lib alone, with store.md's
canonical encoding (3.1) and `H` (3.2). 01.1 joins it to the workspace,
names skein by URL, and puts in place the value model, a stream of items
in pre-order, and its encoder, which asserts 3.1's shape as each item is
written into a count (to measure) or a sized `Writer` (a record's body).
01.2 adds `H` over skein-lib's streaming SHA-256 (`skein_lib::sha256`, on
skein's `main`), with the tag `hearth/1` and store.md 3.2's vectors.
hearth 09's decoder takes the value model, hearth 11's spec digest `H`.

## Read first

- hearth's store.md: 3.1, 3.2; 3.4's `entry` form; 6.1. README.md, 4.2;
  render.md, 4.6, 2.3's JSON depth and 4.4's exports part (`DEPTH`).
- skein's lib.md: 6 (`Writer`, `bytes::copy_of`), 6.2 (SHA-256);
  `findings/skein.md`, 1; programming-model.md: 3, 6.1, 6.2, 8, 10.
- This plan's README, Rules: "hearth's first landings", "Upstream first",
  "Format versions", "Shared files"; code-organization.md, 1, 3, 4, 6.
- hearth's root `Cargo.toml`, lines 13-17 (how skein is named);
  `clippy.toml`, whose lints hold in this crate's tests too.

## What the design asks

- **Seven kinds, eight first bytes** (store.md, 3.1): `none 00`, `false
  01`, `true 02`, `uint 03` (8 bytes, big-endian), `bytes 04` (the length
  as 8 bytes, then b), `digest 05` (32 bytes), `list 06` (the count, then
  each v), `map 07` (the count, then each entry's key as `bytes`, then v).
- **Keys and records:** "Map keys are byte strings, strictly increasing
  bytewise; a duplicate is not a map."; "`none` is a value, not an
  absence"; "An optional field is left out when absent."; "An enumeration
  is its variant's name, as `bytes`"; "A set is a list, without
  duplicates, in its members' order".
- **One encoding:** "One value has one encoding, and a decoder needs no
  lookahead."; the encoder and `H` live in `hearth-lib`, `no_std`, on
  skein-lib alone; "Decoding is parsing", a protocol machine's.
- **H** (3.2): `H(tag, v1, …, vn) = SHA-256( encode( list [ bytes
  "hearth/1", bytes tag, v1, …, vn ] ) )`; seven test vectors.

## Choices this plan makes

- **The value model is items in pre-order,** `encode::Item`: a
  container's head carries its count and its values follow, a map's as
  key, value, key, value. No value is a tree and nothing recurses
  (programming-model.md, 3 and 8). hearth 09's decoder yields the same
  `Item`s; code-organization.md, 3 makes the model hearth 01's alone.
- **One encoder, three sinks.** A record's encoding is one function over
  `&mut Encoder`, run on `Encoder::measuring()` and then on
  `Encoder::writing(len)` (programming-model.md, 8: "compute the
  message's length, `Writer::new(len)`, write the fields, `finish()`"),
  or once into `H::values()`.
- **Misuse is a bug, asserted** (programming-model.md, 10.4): the input
  is built from typed state, so each shape rule is an assertion with the
  Shape's message; hearth 09 refuses the same shapes, as values, in bytes
  from disk. Key order is checked against a copy of each open map's
  previous key (`bytes::copy_of`): one allocation a key.
- **`DEPTH = 64`,** a constant, not a configured limit, since it bounds
  the values hearth builds: an export's value has its declared type
  (hearth 20.1's `conform`), a type in a document at most 32 deep (render.md
  2.3), so at most 29 lists, though an evaluation may make 64 deep (2.3): a
  deeper one is refused before the part. With the part's own levels
  (`H("part:exports", …)`: 4: map task → map name → value) it is 33; 64 leaves
  room. Hearth 20.2 owns the records' JSON `DEPTH` (also 64). store.md, section 6's deepest record is 5.
- **The records' rules stay with the records:** a set is a `list` in the
  order its part names, an enumeration the `bytes` of its name, an
  absent field one fewer in its map's count, text `bytes`. Render's parts
  and the store's records apply them; 01.2's vectors pin them.
- **Names:** `Digest`, a newtype of the 32 bytes, so a digest is never
  `bytes`; modules `hearth_lib::encode` and `hearth_lib::hash`, no
  re-exports; `H`, the design's word.

## Rules that bind it

- **hearth's first landings** (README, Rules): 01.1 adds a crate, so it
  lands with `flock ~/src/rust/tmp/hearth/render/land-hearth.lock land`,
  never `LAND_PREBUILD= land`. The root `Cargo.toml`
  and `Cargo.lock` are shared: landing second, rebase, and keep the
  other session's dependency lines beside this crate's.
- **Upstream first** (README, Rules): lib's `sha256` is on skein's `main`
  (`0f093bb`), so 01.1's first build pins it and 01.2 needs no repin:
  `grep -o 'skein.git#[0-9a-f]*' Cargo.lock | sort -u` prints one commit,
  which `git -C ~/src/rust/skein merge-base --is-ancestor 0f093bb <it>`
  accepts. If it does not, 01.2 starts with a repin commit of its own,
  `heavy cargo update -p skein-lib`.
- **Format versions** (README, Rules): the tag `hearth/1` is 01.2's, at
  1; nothing here changes it or 3.1's encoding (store.md, 3.2). Test
  digests are store.md 3.2's literals, pasted (code-organization.md, 6).

## Increments

### 01.1 The crate, the value model and the encoder

Scope: the canonical encoding (store.md, 3.1): the value model; the
encoder into a count and a sized writer; 3.1's shape asserted as items
are written; the crate in the workspace, skein named by URL.
Needs: nothing.
Files: root `Cargo.toml`, `Cargo.lock` (shared: README, Rules, "Shared
files"); new `crates/hearth-lib/Cargo.toml`, `src/lib.rs`,
`src/encode.rs`, `src/tests.rs`, `src/tests/fixtures.rs`,
`src/tests/encode.rs`. No users: hearth has no crate at the base.
Shape:
- Root `Cargo.toml`: `"crates/*"` in `members`, if another session's
  landing has not put it there (decisions.md, 16: a glob for each
  root once it holds a crate; never one for an empty root); under
  `[workspace.dependencies]`, `skein-lib = { git =
  "https://github.com/forgetic/skein.git" }`, the comment's example line,
  which leaves the comment (its prose stays). `Cargo.lock`, made by the
  first build, is committed: it pins skein's `origin/main` of the day.
- The crate's `Cargo.toml` as skein-lib's: the four `.workspace = true`
  package keys, `skein-lib.workspace = true`, `[lints] workspace = true`.
  `src/lib.rs`: `#![cfg_attr(not(test), no_std)]`,
  `#![forbid(unsafe_code)]`, `extern crate alloc;`, `pub mod encode;`,
  `#[cfg(test)] mod tests;`.
- `hearth_lib::encode` (sketch): each method writes its item as 3.1's
  table does, the kind's byte, then `to_be_bytes` of a uint, length or
  count, then the bytes, into the sink: `Measure` adds their length
  (checked); `Write` puts them into `skein_lib::Writer::new(len)`
  (`expect("an encoder writes within the length it measured")`).
- The shape, on a `skein_lib::Stack<Open>` of `DEPTH`, each rule an
  assertion with this message:
  - a second value at the top: "an encoding holds one value". A
    container closes once its count is used (`settle`), so a value past
    it is its parent's, or this second value;
  - a key not above the map's previous one: "a map's keys strictly
    increase"; a value where a key belongs: "a map's entry starts with
    its key"; a key where its value belongs: "a map's key is followed by
    its value"; a key outside a map: "a key belongs to a map";
  - a head past `DEPTH` open: "a value nests at most DEPTH containers
    deep";
  - `measured` or `written` before the value is whole: "an encoding is
    finished with its one value whole" (and `unreachable!` on the other
    sink).

  A head takes its slot in its parent when placed; after a value ends,
  `settle` closes each container it completed (sketch).
- `item(&Item)` calls its variant's method; `Item::Bytes` where a key
  belongs is `key`, since a key's bytes are a `bytes` value's (3.1).

Test shape: `src/tests.rs` (`mod encode; mod fixtures;`); the crate's
fixtures module, `src/tests/fixtures.rs`, new (code-organization.md, 4):
`hex(text: &str) -> Box<[u8]>` (store.md's hex as written, spaces
skipped, into `bytes::zeroed`: no `Vec`, as the lints hold in tests);
`text(bytes: &[u8]) -> Item`; `put(encoder: &mut Encoder, items:
&[Item])`, each through `item`; `encode(items: &[Item]) -> Box<[u8]>`,
measured by one encoder, written by a second. No configured limits;
`encode::DEPTH` is the crate's. `src/tests/encode.rs`, each a unit test
under 20 ms; the reference test:

```rust
use super::fixtures::{encode, hex, text};
use crate::encode::Item;
#[test]
fn a_list_of_texts_encodes_as_store_md_writes_it() {
    let value = [Item::List(2), text(b"hearth/1"), text(b"empty")];
    let expected = hex("06 0000000000000002 04 0000000000000008 6865617274682f31 04 0000000000000005 656d707479");
    assert_eq!(encode(&value), expected, "store.md, 3.2: encode(list [bytes \"hearth/1\", bytes \"empty\"])");
}
```

The `_is_a_bug` tests drive an `Encoder::measuring()` by hand under
`#[should_panic(expected = "<the Shape's message>")]`, as skein-lib's
`finishing_short_is_a_bug` (`crates/skein-lib/src/tests/bytes.rs:233`).
- `a_map_key_is_encoded_as_a_whole_bytes_value`: `map { "a": uint 1 }`
  is 3.2's `07 0000000000000001 04 0000000000000001 61 03
  0000000000000001`.
- `each_scalar_kind_starts_with_a_byte_of_its_own`: rows in a `const`:
  none `00`, false `01`, true `02`, uint 0o755 `03 00000000000001ed`,
  digest 01…01 `05` then 32 × `01`.
- `map_keys_order_by_their_bytes_not_their_encodings`: `map { "ab": none,
  "b": none }` is `07 0000000000000002 04 0000000000000002 6162 00 04
  0000000000000001 62 00`.
- `a_record_without_its_optional_field_holds_an_empty_map`: `map { "adm":
  map { "gid": uint 4 }, "staff": map {} }` is `07 0000000000000002 04
  0000000000000003 61646d 07 0000000000000001 04 0000000000000003 676964
  03 0000000000000004 04 0000000000000005 7374616666 07
  0000000000000000`.
- `a_duplicate_key_in_a_map_is_a_bug`: `map(2)`, `key(b"a")`, `none`,
  `key(b"a")`; `a_key_below_the_previous_is_a_bug`: the same with `b`
  first: "a map's keys strictly increase".
- `a_value_where_a_key_belongs_is_a_bug`: `map(1)`, `none`: "a map's entry
  starts with its key". `a_key_where_a_value_belongs_is_a_bug`: `map(1)`,
  `key(b"a")`, `key(b"b")`: "a map's key is followed by its value".
  `a_key_outside_a_map_is_a_bug`: `list(1)`, `key(b"a")`: "a key belongs
  to a map".
- `a_value_after_the_whole_value_is_a_bug`: `list(1)`, `none`, `none`:
  "an encoding holds one value".
- `an_encoding_finished_with_a_list_open_is_a_bug`: `list(2)`, `none`,
  `measured`: "an encoding is finished with its one value whole".
- `a_value_nested_past_the_depth_bound_is_a_bug`: `list(1)`, `DEPTH + 1`
  times: "a value nests at most DEPTH containers deep".
- `a_value_sixty_four_containers_deep_encodes`: `list(1)` 64 times, then
  `none`, measures 577 bytes (64 × 9 + 1): an export's value as deep as
  render.md 2.3's JSON allows, inside its part, is no bug (Choices).

Proof: unit, the only one that reaches the crate: `heavy cargo
nextest run -p hearth-lib -E
'test(a_list_of_texts_encodes_as_store_md_writes_it)'`, once: store.md
3.2's first vector, byte for byte, measured then written. Nothing above
calls the encoder until hearth 09, whose `hearth-protocol-tree` decoder
yields `Item`s and whose tests encode them again. Bound: 0.02 s.
Don't: decode (hearth 09's); build a tree of values or recurse; sort
(a set's order is its part's); depend on anything but skein-lib; add a
`limits` module, a text method, or a signed or float kind.
Size: at most 260 lines.
Removes: nothing.

### 01.2 `H`, the tag `hearth/1`, and store.md's vectors

Scope: `H` (store.md, 3.2) over skein-lib's streaming SHA-256; the tag
`hearth/1`, version 1; 3.2's five `H` vectors (01.1 holds its two
`encode` vectors), the two of answer H1 that pin absent fields and
enumerations among them.
Needs: 01.1 (its `encode.rs`, `lib.rs`, `tests/fixtures.rs`).
Files: `Cargo.lock` (only if the pin check fails: the repin commit,
first; shared); `crates/hearth-lib/src/encode.rs` (the hash sink),
`src/lib.rs`, new `src/hash.rs`; `src/tests.rs`, `src/tests/fixtures.rs`,
new `src/tests/hash.rs`.
Shape:
- `Sink::Hash { sha: Sha256 }`: each item's bytes go to `update` as they
  are written, with no copy of the encoding (lib.md, 6.2: "`update(&[u8])`
  with each piece as it arrives"; "No heap, no copy"); `hashing()` and
  `hashed(self) -> [u8; 32]` are `pub(crate)`, for `H` alone.
- `hearth_lib::hash` (sketch): `H::new(tag, values)` is `hashing()`,
  then `list(values + 2)` (checked: "H takes fewer than 2^64 - 2
  values"), `bytes(VERSION)`, `bytes(tag)`; `values()` lends the encoder
  for v1 … vn; `finish` is `Digest(hashed())`, whose assertion catches a
  missing value. `src/lib.rs` gains `pub mod hash;`.

Test shape: `src/tests/hash.rs` (`mod hash;`), on 01.1's fixtures plus
`digest(text: &str) -> Digest` (64 hex digits) in `fixtures.rs`; each a
unit test under 20 ms, its digest pasted whole from store.md 3.2. The
reference test:

```rust
use super::fixtures::digest;
use crate::hash::H;
#[test]
fn h_of_empty_is_the_digest_store_md_gives() {
    let h = H::new(b"empty", 0);
    let expected = digest("8f43cea6a4bf9d3aec62c882bdafeab30cc6d71acc9b0b984b7f2fd8f70975d3");
    assert_eq!(h.finish(), expected, "store.md, 3.2: H(\"empty\")");
}
```

- `h_of_packages_over_two_digests_matches_store_md` (`Digest([0; 32])`
  and `Digest([1; 32])`, store.md's digest 00…00 and 01…01): `22d5156f…`.
- `h_of_the_empty_layers_tree_matches_store_md`: under `tree`, `list(1)`,
  `list(7)`, bytes "", uints 1, 0o755, 0, 0, `map(0)`, none: `6022d1fb…`.
- `h_with_staffs_gid_absent_matches_store_md`: `8291ea66…`.
- `staffs_gid_as_none_gives_the_digest_store_md_names`: with `"staff":
  map { "gid": none }`, its first four bytes are `ab 37 93 df`, the
  prefix the design gives.
- `a_variant_by_its_name_matches_store_md`: `map { "kind": bytes
  "file", "mode": uint 0o644, "owner": bytes "root" }`: `72653b99…`.
- `h_finished_before_its_values_is_a_bug`: `H::new(b"test", 1)`, then
  `finish`: "an encoding is finished with its one value whole";
  `h_with_a_count_past_the_limit_is_a_bug`: `H::new(b"test", u64::MAX)`:
  "H takes fewer than 2^64 - 2 values".

The `3f4edef7…` prefix is not a test: no correct encoder makes it.

Proof: unit, the only one: `heavy cargo nextest run -p
hearth-lib -E 'test(h_with_staffs_gid_absent_matches_store_md)'`, once:
`8291ea66…`, a digest that rests on 3.1's keys and absent fields,
through skein's SHA-256. Nothing above calls `H` until hearth 11:
render's spec digest, `H("spec", …)` (render.md, 4.5), which `hearth
render` prints in the thin story. Bound: 0.02 s.
Don't: hash a copy of the encoding; make a hashing encoder public; write
`b"hearth/1"` in step code anywhere but `hash::VERSION` (tests may);
define the tags (each is its user's, from 3.2's table); wrap a content
digest (3.2: "plain SHA-256 of its bytes, without `H`",
`skein_lib::sha256::digest`).
Size: at most 80 lines.
Removes: nothing.

## Interface sketches

`hearth_lib::encode` is the value model hearth 09 decodes into and every
encoding crate writes with. An encoder is ended only by `measured`,
`written` or `H::finish`, each taking it by value; `H::values` lends it.

```rust
// crates/hearth-lib/src/encode.rs (01.1; Sink::Hash, hashing and hashed in 01.2)
/// The first byte of each kind (store.md, 3.1's table), one constant a row.
pub const NONE: u8 = 0x00; // and FALSE 0x01, TRUE 0x02, UINT 0x03, BYTES 0x04, DIGEST 0x05, LIST 0x06, MAP 0x07
/// The most containers open at once: render.md 2.3's JSON depth, 32, plus a part's own levels, with room.
pub const DEPTH: u32 = 64;

/// A SHA-256 digest: a key, a part's digest, a file's content (store.md, 3.2).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Digest(pub [u8; 32]);

/// One item of a value in pre-order: what the encoder takes and hearth 09's decoder yields.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Item {
    /// `none`: a field always present that holds nothing.
    None,
    /// The boolean false.
    False,
    /// The boolean true.
    True,
    /// An unsigned integer, 8 bytes big-endian.
    Uint(u64),
    /// A byte string: text, a name, a variant's name; a key where a map's key belongs.
    Bytes(Box<[u8]>),
    /// A digest's 32 bytes, never `bytes`.
    Digest(Digest),
    /// A list's head: its count of values follow.
    List(u64),
    /// A map's head: its count of entries follow, each a key then its value.
    Map(u64),
}

/// Encodes one value, item by item, into its sink, asserting 3.1's shape as it goes.
#[derive(Debug)]
pub struct Encoder {
    sink: Sink, // Measure { len: u64 } | Write { writer: skein_lib::Writer } | Hash { sha: Sha256 } (01.2)
    open: skein_lib::Stack<Open>,
    placed: bool,
}

/// A container still open: a list's values to come; a map's keys to come, its previous key, and
/// whether that key awaits its value.
#[derive(Debug)]
enum Open {
    List { left: u64 },
    Map { left: u64, last: Option<Box<[u8]>>, awaiting: bool },
}

impl Encoder {
    /// An encoder that counts the bytes of the value it is given.
    #[must_use] pub fn measuring() -> Encoder;
    /// An encoder that writes the value into exactly `len` bytes, as measured.
    #[must_use] pub fn writing(len: u64) -> Encoder;
    pub fn none(&mut self);
    pub fn boolean(&mut self, value: bool);
    pub fn uint(&mut self, value: u64);
    pub fn bytes(&mut self, bytes: &[u8]);
    pub fn digest(&mut self, digest: &Digest);
    /// A list's head; `count` values follow.
    pub fn list(&mut self, count: u64);
    /// A map's head; `count` entries follow, each `key` then a value.
    pub fn map(&mut self, count: u64);
    /// A map's key, above its previous one bytewise.
    pub fn key(&mut self, key: &[u8]);
    /// The item's own method; `Item::Bytes` where a key belongs is `key`.
    pub fn item(&mut self, item: &Item);
    #[must_use] pub fn measured(self) -> u64;
    #[must_use] pub fn written(self) -> Box<[u8]>;
    pub(crate) fn hashing() -> Encoder;
    pub(crate) fn hashed(self) -> [u8; 32];
}

/// After a value ends, closes each container it completed; a head took its slot when placed.
fn settle(encoder: &mut Encoder) {
    for _ in 0..DEPTH {
        match encoder.open.top() {
            Some(Open::List { left: 0 } | Open::Map { left: 0, awaiting: false, .. }) => drop(encoder.open.pop()),
            _ => return,
        }
    }
}
```

```rust
// crates/hearth-lib/src/hash.rs (01.2)
/// The encoding's version and hearth's constants', H's first value (store.md, 3.2). Owner: hearth 01.2.
pub const VERSION: &[u8] = b"hearth/1";

/// One digest being made: its list's head, VERSION and tag hashed, its values to come.
#[derive(Debug)]
pub struct H {
    encoder: Encoder,
}

impl H {
    /// H(tag, v1, …, vn), n = `values`.
    #[must_use] pub fn new(tag: &[u8], values: u64) -> H;
    /// Where v1 … vn go, each one whole value.
    #[must_use] pub fn values(&mut self) -> &mut Encoder;
    /// The digest, once each value is whole.
    #[must_use] pub fn finish(self) -> Digest;
}
```

## Questions to raise when reached

store.md 3.1 and 3.2 leave nothing open here. Raise only:

- a vector of 3.2 the encoder disagrees with while a hand encoding from
  3.1's table agrees with it, both in hex: the vectors are the design's
  (recomputed at planning, 2026-10-16, and agreed: look in the code first).
- `skein_lib::sha256` without lib.md 6.2's streaming form (`update`,
  `finish(self) -> [u8; 32]`), before 01.2's Shape; the same form under
  another path is followed as landed.

## Done when

- This prints 2, from hearth's root:
  `git log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-01\.[12]'`
- In a worktree of hearth at `origin/main`, once, `heavy cargo nextest
  run -p hearth-lib` prints a `Summary` line with 22 tests passed and
  none failed.
- `git -C ~/src/rust/hearth grep -n -F 'b"hearth/1"' origin/main --
  crates ':!*/tests/*' ':!*/tests.rs'` prints one line, in
  `crates/hearth-lib/src/hash.rs`.
- `git -C ~/src/rust/hearth grep -n -E 'mod limits|pub fn (hashing|hashed|text)|i64|f64' origin/main -- crates/hearth-lib/src ':!*/tests/*'`
  prints nothing.
- In that worktree, `cargo tree -p hearth-lib -e normal` prints two
  lines: `hearth-lib`, then `skein-lib` from
  `https://github.com/forgetic/skein.git`.
- `git -C ~/src/rust/hearth show origin/main:Cargo.lock | grep -o
  'skein.git#[0-9a-f]*' | sort -u` prints one commit, which `git -C
  ~/src/rust/skein merge-base --is-ancestor 0f093bb <it>` accepts.
