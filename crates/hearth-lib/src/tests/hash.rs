use super::fixtures::{digest, put, text};
use crate::encode::{Digest, Item};
use crate::hash::H;

#[test]
fn h_of_empty_is_the_digest_store_md_gives() {
    let h = H::new(b"empty", 0);
    let expected = digest("8f43cea6a4bf9d3aec62c882bdafeab30cc6d71acc9b0b984b7f2fd8f70975d3");
    assert_eq!(h.finish(), expected, "store.md, 3.2: H(empty)");
}

#[test]
fn h_of_packages_over_two_digests_matches_store_md() {
    let mut h = H::new(b"packages", 2);
    let values = [Item::Digest(Digest([0; 32])), Item::Digest(Digest([1; 32]))];
    put(h.values(), &values);
    let expected = digest("22d5156ffefadb1f0df4eb2c4d405b2c8937215e044486b06b1725e3506b238d");
    assert_eq!(h.finish(), expected, "store.md, 3.2: H(packages, two digests)");
}

#[test]
fn h_of_the_empty_layers_tree_matches_store_md() {
    let mut h = H::new(b"tree", 1);
    let values = [
        Item::List(1),
        Item::List(7),
        text(b""),
        Item::Uint(1),
        Item::Uint(0o755),
        Item::Uint(0),
        Item::Uint(0),
        Item::Map(0),
        Item::None,
    ];
    put(h.values(), &values);
    let expected = digest("6022d1fbc6ad38d08d2ae02cefc21e7e6af5a35e6be9d115f0ba267fdb0d005a");
    assert_eq!(h.finish(), expected, "store.md, 3.2: H(tree, the empty layer)");
}

#[test]
fn h_with_staffs_gid_absent_matches_store_md() {
    let mut h = H::new(b"test", 1);
    let values = [Item::Map(2), text(b"adm"), Item::Map(1), text(b"gid"), Item::Uint(4), text(b"staff"), Item::Map(0)];
    put(h.values(), &values);
    let expected = digest("8291ea66351528fd3979b84aec0397019d25dae6f8d8aa9a0318331d2fd164f5");
    assert_eq!(h.finish(), expected, "store.md, 3.2: staff's gid is absent");
}

#[test]
fn staffs_gid_as_none_gives_the_digest_store_md_names() {
    let mut h = H::new(b"test", 1);
    let values = [
        Item::Map(2),
        text(b"adm"),
        Item::Map(1),
        text(b"gid"),
        Item::Uint(4),
        text(b"staff"),
        Item::Map(1),
        text(b"gid"),
        Item::None,
    ];
    put(h.values(), &values);
    let digest = h.finish();
    assert_eq!(
        digest.0.get(..4),
        Some([0xab, 0x37, 0x93, 0xdf].as_slice()),
        "store.md, 3.2: gid none has a distinct digest"
    );
}

#[test]
fn a_variant_by_its_name_matches_store_md() {
    let mut h = H::new(b"test", 1);
    let values =
        [Item::Map(3), text(b"kind"), text(b"file"), text(b"mode"), Item::Uint(0o644), text(b"owner"), text(b"root")];
    put(h.values(), &values);
    let expected = digest("72653b9931df15f292b4f5f17e091263cd9fb738b0f2c06cce1cd74242b5b211");
    assert_eq!(h.finish(), expected, "store.md, 3.2: a variant is its name");
}

#[test]
#[should_panic(expected = "an encoding is finished with its one value whole")]
fn h_finished_before_its_values_is_a_bug() {
    let h = H::new(b"test", 1);
    let _digest = h.finish();
}

#[test]
#[should_panic(expected = "H takes fewer than 2^64 - 2 values")]
fn h_with_a_count_past_the_limit_is_a_bug() {
    let _h = H::new(b"test", u64::MAX);
}
