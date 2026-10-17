use hearth::Form;
use hearth_builder_world::fixtures;
use skein_io::kernel::Exit;
#[test]
fn an_unreachable_mirror_fails_the_resolution_as_mmdebstraps_and_exits_75() {
    let (repository, rendered) = fixtures::failed_test_min("unreachable");
    let text = String::from_utf8_lossy(&rendered.stdout);
    assert!(text.contains("test-min: mmdebstrap exited ") && text.contains(" needed\n"), "7.2: {text}");
    assert_eq!(rendered.exit, Exit::Code(75));
    assert!(fixtures::names(&repository.scratch().path().join("store/staging")).is_empty(), "5.2");
}

#[test]
fn an_unknown_package_fails_its_resolution_naming_it_and_exits_2() {
    let repository = fixtures::unknown_package_repository();
    let mirror = fixtures::mirror();
    let proxy = hearth_test_mirror::Proxy::serve(&mirror).expect("loopback proxy");
    let rendered = fixtures::resolve(&repository, &[b"test-min"], Form::Text, proxy.url().as_bytes());
    let text = String::from_utf8_lossy(&rendered.stdout);
    assert!(text.contains("test-min: apt cannot locate package \"hearth-absent\""), "5.5: {text}");
    assert_eq!(rendered.exit, Exit::Code(2));
}

#[test]
fn test_mins_closure_is_recorded_as_mmdebstraps_recording_resolved_it() {
    let (repository, rendered) = fixtures::resolved_test_min("record-closure");
    let record = fixtures::read_test_min(&repository);
    assert_eq!(record.closure, fixtures::recorded_closure());
    assert_eq!(
        record
            .sources
            .iter()
            .map(|line| (line.uri.as_ref(), line.suite.as_ref(), line.components.as_ref()))
            .collect::<Vec<_>>(),
        [(
            b"http://snapshot.debian.org/archive/debian/20261017T143929Z/".as_slice(),
            b"forky".as_slice(),
            [b"main".as_slice().into()].as_slice()
        )]
    );
    let text = String::from_utf8_lossy(&rendered.stdout);
    assert!(text.lines().any(|line| line.starts_with("packages ") && line.ends_with(" known")), "{text}");
    assert_eq!(rendered.exit, Exit::Code(0));
}

#[test]
fn test_mins_indices_enter_the_cache_by_their_digests_with_their_records() {
    use std::os::unix::fs::PermissionsExt;
    let (repository, rendered) = fixtures::resolved_test_min("kept-indices");
    let record = fixtures::read_test_min(&repository);
    let digests = fixtures::mirror_indices();
    assert_eq!(
        record
            .sources
            .iter()
            .flat_map(|line| line.indices.iter())
            .map(|index| (index.path.as_ref(), index.sha256.0))
            .collect::<Vec<_>>(),
        digests
    );
    for (_, digest) in digests {
        let (path, record) = fixtures::cached_index(&repository, digest);
        assert_eq!(skein_lib::sha256::digest(&std::fs::read(&path).expect("kept index")), digest);
        assert_eq!(std::fs::metadata(&path).expect("index mode").permissions().mode() & 0o777, 0o644);
        assert_eq!(record.sha256.0, digest);
        assert_eq!(
            record.origin,
            hearth_protocol_tree::cache_file::Origin::Index { resolution: fixtures::TEST_MIN_RESOLUTION }
        );
    }
    assert_eq!(rendered.exit, Exit::Code(0));
}

#[test]
fn a_resolution_says_on_standard_error_that_it_starts_and_ends() {
    let (_, rendered) = fixtures::failed_test_min("resolution-progress");
    assert_eq!(
        String::from_utf8_lossy(&rendered.stderr).lines().collect::<Vec<_>>(),
        ["resolving 21e9a02feb65", "resolution 21e9a02feb65 failed"]
    );
    assert!(
        !String::from_utf8_lossy(&rendered.stdout)
            .lines()
            .any(|line| line.starts_with("resolving ") || line.starts_with("resolution "))
    );
}
