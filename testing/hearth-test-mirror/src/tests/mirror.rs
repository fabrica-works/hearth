use super::fixtures;
use crate::{CACHE, Mirror, Missing, Pin};
use skein_scratch::Scratch;
use std::{fs, os::unix::fs::symlink, path::Path};

const INDEX: &str = "checked/main/binary-amd64/Packages";
#[test]
fn the_mirror_is_found_under_the_cache_by_its_digest() {
    let scratch = Scratch::new("test-mirror");
    let laid = fixtures::laid(scratch.path());
    let mirror = Mirror::find_in(scratch.path(), laid.pin()).unwrap();
    let root = scratch.path().join(CACHE).join(&*laid.pin().digest);
    assert_eq!(mirror.pin(), laid.pin());
    assert_eq!(mirror.index("main"), root.join(INDEX));
}
const FETCH_SCRIPT: &str = "testing/mirror/fetch";
#[test]
fn a_mirror_not_fetched_names_the_fetch_script() {
    let scratch = Scratch::new("missing-mirror");
    let pin = Pin::read(fixtures::PIN).unwrap();
    let error = Mirror::find_in(scratch.path(), &pin).unwrap_err();
    assert_eq!(error, Missing::Unfetched { path: scratch.path().join(CACHE).join(&*pin.digest) });
    assert!(error.to_string().contains(FETCH_SCRIPT));
}
const XZ: &[u8] = b"xz\n";
#[test]
fn a_snapshot_path_names_its_file_beneath_the_mirror() {
    let scratch = Scratch::new("test-mirror");
    let mirror = fixtures::laid(scratch.path());
    let path = Path::new(fixtures::SNAPSHOT).join("main/binary-amd64/by-hash/SHA256").join(fixtures::XZ_SHA256);
    let file = mirror.file(&path).expect("the by-hash link is served");
    assert_eq!(fs::read(file).unwrap(), XZ);
}
const NO_FILES: &[&str] = &[
    "archive/./debian/20261016T091523Z/dists/forky/InRelease",
    "archive/debian/../debian/20261016T091523Z/dists/forky/InRelease",
    "archive//debian/20261016T091523Z/dists/forky/InRelease",
    "archive/debian/20261016T091523Z/dists/forky/InRelease/",
    "archive/debian/20261016T091523Z/dists/forky/link",
    "archive/linked/InRelease",
    "checked/main/binary-amd64/Packages",
    "/archive/debian/20261016T091523Z/dists/forky/InRelease",
    "archive",
    "archive/missing",
];
#[test]
fn a_dot_segment_a_link_or_the_checked_index_is_no_file() {
    let scratch = Scratch::new("test-mirror");
    let mirror = fixtures::laid(scratch.path());
    let root = scratch.path().join(CACHE).join(&*mirror.pin().digest);
    let suite = root.join(fixtures::SNAPSHOT);
    symlink(suite.join("InRelease"), suite.join("link")).unwrap();
    symlink(&suite, root.join("archive/linked")).unwrap();
    for path in NO_FILES {
        assert_eq!(mirror.file(Path::new(path)), None, "{path}");
    }
}
