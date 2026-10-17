use hearth::Form;
use hearth_builder_world::fixtures::{self, Repository};
use skein_io::kernel::Exit;
#[test]
fn a_recorded_resolution_prints_known_and_the_layer_keys() {
    let repository = fixtures::recorded_test_min("known");
    let out = fixtures::render(&repository, &[b"test-min"], Form::Text);
    let printed = fixtures::lines_starting(&out.stdout, &["packages ", "keys "]);
    assert_eq!(
        printed,
        "packages     part 78ec4b1fb53a   resolution 21e9a02feb65 known\nkeys         packages d018a0eea488   software f60610453970   config 8c9bf06f6262\n"
    );
}
#[test]
fn a_recorded_resolution_exits_successfully() {
    let repository = fixtures::recorded_test_min("known-exit");
    assert_eq!(fixtures::render(&repository, &[b"test-min"], Form::Text).exit, Exit::Code(0));
}
#[test]
fn an_unrecorded_resolution_prints_needed_and_no_keys_line() {
    let repository = Repository::make("needed", fixtures::TEST_REPOSITORY);
    let out = fixtures::render(&repository, &[b"test-min"], Form::Text);
    assert_eq!(
        fixtures::lines_starting(&out.stdout, &["packages ", "keys "]),
        "packages     part 78ec4b1fb53a   resolution 21e9a02feb65 needed\n"
    );
}
#[test]
fn an_unrecorded_resolution_exits_successfully() {
    let repository = Repository::make("needed-exit", fixtures::TEST_REPOSITORY);
    assert_eq!(fixtures::render(&repository, &[b"test-min"], Form::Text).exit, Exit::Code(0));
}
#[test]
fn a_needed_and_a_known_render_give_the_same_spec_and_packages_digests() {
    let repository = Repository::make("same-digests", fixtures::TEST_REPOSITORY);
    let input = fixtures::render(&repository, &[b"test-min"], Form::JsonLines);
    fixtures::record_test_min(&repository);
    let known = fixtures::render(&repository, &[b"test-min"], Form::JsonLines);
    assert_eq!(fixtures::spec_packages_digests(&known.stdout), fixtures::spec_packages_digests(&input.stdout));
}
#[test]
fn the_json_spec_gives_the_resolution_and_its_keys_whole() {
    let repository = fixtures::recorded_test_min("json-known");
    let out = fixtures::render(&repository, &[b"test-min"], Form::JsonLines);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("\"resolution\":{\"key\":\"21e9a02feb654883ef051d247cb14db878e04c03c2f7675c784a679ea0956d5d\",\"keys\":{\"config\":\"8c9bf06f62629795e2ee99885a9faf9ad94bd7a066f5a40f39739824ecb3bde6\",\"packages\":\"d018a0eea48807bb1966946d32e001cc2a2a6378bb257e8786e4961427de36a1\",\"software\":\"f6061045397016408cb4dcc2c3145f790f1ff970564964c7f9cbfd5c4c206b5c\"},\"kind\":\"known\"}"), "{text}");
}
#[test]
fn the_test_repositorys_targets_render_the_pins_resolution_inputs() {
    let repository = Repository::make("pinned-inputs", fixtures::TEST_REPOSITORY);
    let out = fixtures::render(&repository, &[b"test-min"], Form::JsonLines);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("\"sources\":{\"architecture\":\"amd64\",\"archive\":\"20261017T143929Z\",\"components\":[\"main\"],\"suite\":\"forky\",\"updates\":false}"), "{text}");
}

#[test]
fn resolve_with_its_key_recorded_runs_no_mmdebstrap() {
    let repository = fixtures::recorded_test_min("resolve-recorded");
    let rendered = fixtures::resolve(&repository, &[b"test-min"], Form::Text, &fixtures::dead_proxy());
    assert!(fixtures::lines_starting(&rendered.stdout, &["packages "]).ends_with("resolution 21e9a02feb65 known\n"));
    assert_eq!(rendered.exit, Exit::Code(0));
    assert!(fixtures::names(&repository.scratch().path().join("store/staging")).is_empty());
}
