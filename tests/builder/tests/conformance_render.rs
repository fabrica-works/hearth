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
    let pin = hearth_test_mirror::Pin::committed();
    let mirror = hearth_test_mirror::Mirror::find(&pin).expect("fetched pinned mirror");
    let proxy = hearth_test_mirror::Proxy::serve(&mirror).expect("loopback proxy");
    let rendered = fixtures::resolve(&repository, &[b"test-min"], Form::Text, proxy.url().as_bytes());
    let text = String::from_utf8_lossy(&rendered.stdout);
    assert!(text.contains("test-min: apt cannot locate package \"hearth-absent\""), "5.5: {text}");
    assert_eq!(rendered.exit, Exit::Code(2));
}
