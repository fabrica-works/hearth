use hearth_conformance_world::fake_git::{self, ALLOWED};
#[test]
fn the_fake_prints_each_git_recording_byte_for_byte() {
    let recordings = hearth_recordings::all("git");
    assert!(!recordings.is_empty(), "recordings are committed");
    for (case, recording) in recordings.iter().filter(|(case, _)| !ALLOWED.iter().any(|a| a.case == &**case)) {
        let run = fake_git::run(case, recording);
        fake_git::assert_same(case, &run.captured, recording);
    }
}
