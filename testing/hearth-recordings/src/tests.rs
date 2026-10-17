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
        argv: lines(b"tool\narg\n"),
        env: lines(b"A=b\n"),
        dir: b"/scratch/store".to_vec(),
        stdin: Some(vec![0, 255]),
        stdout: vec![1, 0, 255],
        stderr: b"error\n".to_vec(),
        exit: "signal 9".into(),
        expect: Some(b"expected\n".to_vec()),
        lists: None,
        stanzas: None,
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

const RESOLVE_FILES: &[(&str, &[u8])] = &[
    ("lists", b"directory partial\nfile 4 index_Packages\n"),
    ("stanzas", b"Package: apt\nVersion: 3.3.3\nArchitecture: amd64\n\n"),
];
#[test]
fn a_recording_s_lists_and_stanzas_read_back_file_for_file() {
    let scratch = skein_scratch::Scratch::new("resolve-recording");
    let dir = scratch.path().join("case");
    let original = Recording {
        lists: Some(b"directory partial\nfile 4 index_Packages\n".to_vec()),
        stanzas: Some(b"Package: apt\nVersion: 3.3.3\nArchitecture: amd64\n\n".to_vec()),
        ..fixture()
    };
    write_at(&dir, &original);
    assert_eq!(read_at(&dir, "2.test"), original);
    for (name, expected) in RESOLVE_FILES {
        assert_eq!(fs::read(dir.join(name)).expect("resolve recording file"), *expected, "{name}");
    }
}
#[test]
fn a_further_mask_replaces_its_literal_after_the_scratch() {
    let mask =
        Mask::new(Path::new("/tmp/example")).also(b"scratch", b"masked").also(b"127.0.0.1:1234", b"127.0.0.1:PORT");
    assert_eq!(mask.bytes(b"/tmp/example 127.0.0.1:1234"), b"/masked 127.0.0.1:PORT");
}
