use hearth::Form;
use hearth_builder_world::fixtures::{self, Repository};
use skein_io::kernel::Exit;
#[test]
fn an_unreachable_mirror_fails_the_resolution_as_mmdebstraps_and_exits_75() {
    let repository = Repository::make("unreachable", fixtures::TEST_REPOSITORY);
    let rendered = fixtures::resolve(&repository, &[b"test-min"], Form::Text, &fixtures::dead_proxy());
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
