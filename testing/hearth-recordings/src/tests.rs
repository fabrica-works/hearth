use super::*;
const MASK_CASES: &[(&[u8], &[u8])] = &[
    (b"/tmp/example/store", b"/scratch/store"),
    (b"tmp/example/store", b"scratch/store"),
    (b"0123456789abcdef", b"0123456789abcdef"),
];
#[test]
fn the_scratch_path_is_masked_with_and_without_its_slash() {
    let mask = Mask::new(Path::new("/tmp/example"));
    for (input, expected) in MASK_CASES {
        assert_eq!(mask.bytes(input), *expected, "{input:?}");
    }
}
const RECORDING_FILES: &[(&str, &[u8])] = &[
    ("argv", b"tool\narg\n"),
    ("env", b"A=b\n"),
    ("dir", b"/scratch/store"),
    ("stdin", &[0, 255]),
    ("stdout", &[1, 0, 255]),
    ("stderr", b"error\n"),
    ("exit", b"signal 9\n"),
    ("expect", b"expected\n"),
];
fn fixture() -> Recording {
    Recording {
        version: "2.test".into(),
        argv: lines(RECORDING_FILES[0].1),
        env: lines(RECORDING_FILES[1].1),
        dir: RECORDING_FILES[2].1.to_vec(),
        stdin: Some(RECORDING_FILES[3].1.to_vec()),
        stdout: RECORDING_FILES[4].1.to_vec(),
        stderr: RECORDING_FILES[5].1.to_vec(),
        exit: "signal 9".into(),
        expect: Some(RECORDING_FILES[7].1.to_vec()),
    }
}
#[test]
fn a_recording_written_reads_back_file_for_file() {
    let scratch = skein_scratch::Scratch::new("recordings");
    let dir = scratch.path().join("case");
    let original = fixture();
    write_at(&dir, &original);
    assert_eq!(read_at(&dir, "2.test"), original);
    for (name, expected) in RECORDING_FILES {
        assert_eq!(fs::read(dir.join(name)).expect("recorded file"), *expected, "{name}");
    }
}
const ABSENT_FILES: &[&str] = &["stdin", "expect"];
#[test]
fn replacing_a_recording_removes_its_old_optional_files() {
    let scratch = skein_scratch::Scratch::new("recording-replaced");
    let dir = scratch.path().join("case");
    let original = fixture();
    write_at(&dir, &original);
    let replaced = Recording { stdin: None, expect: None, ..original };
    write_at(&dir, &replaced);
    for name in ABSENT_FILES {
        assert!(!dir.join(name).exists(), "old {name} removed");
    }
    assert_eq!(read_at(&dir, "2.test"), replaced);
}
