use hearth_builder_world::fixtures::{self, Repository};
use skein_io::kernel::Exit;
use std::fs;

const UNKNOWN: &[&[&str]] = &[&["render", "--json", "--bogus"], &["render", "--bogus", "--json"]];
#[test]
fn an_unknown_option_is_refused_on_standard_error_in_the_call_s_form() {
    let repository = Repository::empty("binary-usage");
    for args in UNKNOWN {
        let arguments: Vec<_> = args.iter().map(AsRef::as_ref).collect();
        let ended = fixtures::binary(env!("CARGO_BIN_EXE_hearth").as_ref(), &arguments, &[], repository.work_tree());
        assert_eq!(ended.exit, Some(Exit::Code(64)), "{args:?}");
        assert_eq!(ended.stdout, b"", "no answer on usage");
        assert_eq!(ended.stderr, b"{\"record\":\"version\",\"client\":1}\n{\"record\":\"startup\",\"kind\":\"usage\",\"outcome\":\"usage\",\"usage\":{\"misuse\":\"option\",\"word\":\"--bogus\"}}\n", "version and startup records");
    }
}
#[test]
fn a_store_that_cannot_be_made_is_refused_with_75() {
    let repository = Repository::make("binary-store", fixtures::THIN);
    let file = repository.scratch().path().join("file");
    fs::write(&file, b"a file").expect("blocking file");
    let store = file.join("store");
    let ended = fixtures::binary(
        env!("CARGO_BIN_EXE_hearth").as_ref(),
        &["render".as_ref(), "--store".as_ref(), store.as_os_str()],
        &[],
        repository.work_tree(),
    );
    assert_eq!(ended.exit, Some(Exit::Code(75)), "store refusal");
    assert_eq!(ended.stdout, b"", "no answer on startup failure");
    assert_eq!(
        String::from_utf8_lossy(&ended.stderr).replace(store.to_str().expect("scratch path"), "<store>"),
        "hearth: cannot open the store <store>: Not a directory (os error 20)\n",
        "text startup record"
    );
}
