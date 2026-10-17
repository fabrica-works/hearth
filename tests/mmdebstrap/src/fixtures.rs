use hearth_protocol_mmdebstrap::limits::{self, Limits};
pub const LIMITS: Limits = limits::SHIPPED;
#[must_use]
pub fn at_the_limits(limits: &Limits) -> Vec<u8> {
    let name = "a".repeat(usize::try_from(limits.name).expect("bounded fixture value"));
    let mut bytes = format!(
        "Inst {name} ({} Debian [{name}])\n",
        "v".repeat(usize::try_from(limits.name).expect("bounded fixture value"))
    )
    .into_bytes();
    bytes.extend(std::iter::repeat_n(b'x', usize::try_from(limits.line).expect("bounded fixture value") + 1));
    bytes.push(b'\n');
    for _ in 0..limits.tail_lines {
        bytes.extend_from_slice(
            format!("E: {}\n", "x".repeat(usize::try_from(limits.tail_line).expect("tail size fits"))).as_bytes(),
        );
    }
    bytes.extend_from_slice(format!("E: Unable to locate package {name}\n").as_bytes());
    bytes
}

/// All selected payloads reach their limits; an ignored field fills the stanza.
#[must_use]
pub fn index_at_the_limits(limits: &Limits) -> Vec<u8> {
    let name = "a".repeat(usize::try_from(limits.name).expect("name fits"));
    let filename = "f".repeat(usize::try_from(limits.filename).expect("filename fits"));
    let mut text = format!(
        "Package: {name}\nVersion: {name}\nArchitecture: {name}\nFilename: {filename}\nSize: 18446744073709551615\nSHA256: {}\nProvides: ",
        "f".repeat(64)
    );
    let ignored = usize::try_from(limits.stanza)
        .expect("stanza fits")
        .checked_sub(text.len() + 2)
        .expect("room for ignored field");
    text.push_str(&"x".repeat(ignored));
    text.push_str("\n\n");
    text.into_bytes()
}

/// Case inputs are pinned independently of the captured argument vectors.
#[must_use]
pub fn simulate(case: &str) -> hearth_protocol_mmdebstrap::vector::Simulate {
    use hearth_protocol_mmdebstrap::vector::{Apt, Component, Name, Simulate, Sources, Timestamp, Variant};
    use hearth_test_mirror::Pin;
    let pin = Pin::committed();
    let (target, request, archive): (&str, Option<&[&str]>, Option<&str>) = match case {
        "test-min" | "unreachable-mirror" => ("test-min", None, None),
        "test-boot" => ("test-boot", None, None),
        "unknown-package" => ("test-min", Some(&["hearth-absent"]), None),
        "no-candidate" => ("test-min", Some(&["mail-transport-agent"]), None),
        "unsatisfiable" => ("test-min", Some(&["exim4-daemon-light", "postfix"]), None),
        "absent-snapshot" => ("test-min", None, Some("20000101T000000Z")),
        _ => panic!("unknown recorded case {case}"),
    };
    let name = |text: &str| Name::new(text.as_bytes(), &LIMITS).expect("pinned name");
    let target = pin.target(target).expect("pinned target");
    let request = request.map_or_else(
        || target.request.iter().map(|text| name(text)).collect(),
        |request| request.iter().map(|text| name(text)).collect(),
    );
    assert_eq!(&*target.variant, "apt", "M1's pinned variant");
    Simulate {
        sources: Sources {
            suite: name(&pin.suite),
            architecture: name(&pin.architecture),
            components: Box::new([Component::Main]),
            archive: Timestamp::new(archive.unwrap_or(&pin.archive).as_bytes()).expect("pinned timestamp"),
            updates: false,
            security: None,
        },
        request,
        variant: Variant::Apt,
        apt: Apt { install_recommends: false },
    }
}
/// Recreate the vector using the scratch and proxy literals in the capture.
#[must_use]
pub fn invocation(case: &crate::recordings::Case) -> hearth_protocol_mmdebstrap::vector::Invocation {
    use hearth_protocol_mmdebstrap::vector::{self, Proxy};
    let recording = &case.recording;
    let scratch = recording.env.iter().find_map(|line| line.strip_prefix(b"TMPDIR=")).expect("captured scratch");
    let proxy = recording
        .argv
        .iter()
        .find_map(|arg| arg.strip_prefix(b"--aptopt=Acquire::http::Proxy \""))
        .and_then(|value| value.strip_suffix(b"\""))
        .expect("captured proxy");
    let proxy = Proxy::new(proxy, &LIMITS).expect("masked proxy URI");
    vector::simulate(&simulate(&case.case), scratch, Some(&proxy), &LIMITS).expect("pinned vector")
}
/// The lists manifest records regular files and directories, by name.
#[must_use]
pub fn listed_files(recording: &hearth_recordings::Recording) -> Vec<(&str, &[u8])> {
    recording
        .lists
        .as_ref()
        .expect("captured lists")
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| {
            let (kind, rest) = line.split_at(line.iter().position(|byte| *byte == b' ').expect("manifest kind"));
            let rest = &rest[1..];
            let name = if kind == b"file" {
                &rest[rest.iter().position(|byte| *byte == b' ').expect("manifest size") + 1..]
            } else {
                rest
            };
            (std::str::from_utf8(kind).expect("ASCII kind"), name)
        })
        .collect()
}
