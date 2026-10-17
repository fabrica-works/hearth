use hearth_builder_world::fixtures::{self, Repository};
use skein_io::kernel::Exit;
use std::{fs, os::unix::fs::PermissionsExt};

const MADE: &[&str] = &["hearth", "hearth/store", "hearth/store/cache"];
#[test]
fn end_to_end_render() {
    let repository = Repository::make("end-to-end", fixtures::THIN);
    fs::write(repository.work_tree().join("tasks/motd/tree/etc/motd"), "not committed\n").expect("an edit");
    let xdg = repository.scratch().path().join("xdg");
    let environment = [("XDG_CACHE_HOME", xdg.as_os_str()), ("HOME", repository.scratch().path().as_os_str())];
    let ended = fixtures::binary(
        env!("CARGO_BIN_EXE_hearth").as_ref(),
        &["render".as_ref()],
        &environment,
        repository.work_tree(),
    );
    assert_eq!(String::from_utf8_lossy(&ended.stdout), fixtures::THIN_TEXT, "3.1: only committed files render");
    assert_eq!(ended.exit, Some(Exit::Code(0)), "stderr: {}", String::from_utf8_lossy(&ended.stderr));
    for made in MADE {
        assert_eq!(fs::metadata(xdg.join(made)).expect("made").permissions().mode() & 0o7777, 0o700, "3.5: {made}");
    }
}

#[test]
fn end_to_end_diff() {
    let repository = Repository::make("end-to-end-diff", fixtures::MOTD);
    let from = repository.head();
    let to = repository.next(&[fixtures::MOTD_KIND], &[]);
    let xdg = repository.scratch().path().join("xdg");
    let environment = [("XDG_CACHE_HOME", xdg.as_os_str()), ("HOME", repository.scratch().path().as_os_str())];
    let ended = fixtures::binary(
        env!("CARGO_BIN_EXE_hearth").as_ref(),
        &["diff".as_ref(), std::ffi::OsStr::new(&from), std::ffi::OsStr::new(&to)],
        &environment,
        repository.work_tree(),
    );
    assert_eq!(String::from_utf8_lossy(&ended.stdout), fixtures::MOTD_DIFF_TEXT);
    assert_eq!(ended.exit, Some(Exit::Code(1)), "stderr: {}", String::from_utf8_lossy(&ended.stderr));
}

#[test]
fn end_to_end_render_resolve() {
    let repository = fixtures::recorded_test_min("end-to-end-render-resolve");
    let store = repository.scratch().path().join("store");
    let ended = fixtures::binary(
        env!("CARGO_BIN_EXE_hearth").as_ref(),
        &["render".as_ref(), "--resolve".as_ref(), "test-min".as_ref(), "--store".as_ref(), store.as_os_str()],
        &[],
        repository.work_tree(),
    );
    assert!(fixtures::lines_starting(&ended.stdout, &["packages "]).ends_with("resolution 21e9a02feb65 known\n"));
    assert_eq!(ended.exit, Some(Exit::Code(0)), "stderr: {}", String::from_utf8_lossy(&ended.stderr));
}
