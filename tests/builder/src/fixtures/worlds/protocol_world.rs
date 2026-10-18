//! Scenarios and input builders shared by protocol-world checks.
use super::{Fault, File, GIT_DIR, Scenario, Shared, thin};
use hearth_domain_builder::vocabulary::Call;
use hearth_fake_machine::hearth::mirror::{Mirror, Release, Suite, Timestamp};
use hearth_lib::encode::Digest;

/// The plan's sixty entries exceed a single 64-byte pipe by many writes.
pub const MANY_BLOBS: usize = 60;
/// Seven bytes cut rev-parse inside its forty-byte object name.
pub const KILLED_BYTES: u32 = 7;
pub const MOTD_KIND_SHA256: Digest = Digest([
    0xa6, 0x03, 0xa5, 0x5b, 0x16, 0x48, 0xd6, 0x7c, 0xb6, 0x67, 0x25, 0xe7, 0x9b, 0xb1, 0xd1, 0xc5, 0xe3, 0x4d, 0x1b,
    0xfa, 0x5b, 0x59, 0x72, 0x77, 0x3c, 0xb8, 0xe3, 0xa8, 0xb0, 0xa3, 0xe8, 0xb2,
]);
/// The plan's sixty-four-seed sweep covers every fault repeatedly.
pub const FUZZY_PROTOCOL_SEEDS: core::ops::Range<u64> = 0..64;
pub const PROTOCOL_LOCK_RELEASE: skein_lib::Time = skein_lib::Time::from_nanos(2_500_000_000);

#[must_use]
pub fn diff_motd(form: hearth::Form) -> Scenario<'static> {
    Scenario {
        files: super::super::MOTD,
        call: Call::Diff {
            resolve: false,
            repository: GIT_DIR.into(),
            from: b"before".as_slice().into(),
            to: b"HEAD".as_slice().into(),
            targets: Box::new([b"x1-lab".as_slice().into(), b"x1-prod".as_slice().into()]),
        },
        form,
        ..thin()
    }
}

#[must_use]
pub fn many_blobs() -> Vec<File> {
    let mut files = super::super::THIN.to_vec();
    for number in 0..MANY_BLOBS {
        files.push(File { path: format!("tasks/motd/tree/etc/blob-{number}").leak(), body: b"blob\n", mode: 0o644 });
    }
    files
}

pub fn serve(shared: &Shared) {
    let universe = shared.borrow().universe.clone();
    let mut mirror = Mirror::new();
    mirror.add(
        Timestamp::new(b"20261001T030405Z").expect("fixture timestamp"),
        Suite::new(Release::debian(b"forky", b"forky"), b"amd64", Box::new([(b"main".as_slice().into(), universe)])),
    );
    shared.borrow_mut().behind.machine_mut().serve(mirror, None);
}

#[must_use]
pub fn faulty(fault: Fault) -> Scenario<'static> {
    Scenario { fault: Some(fault), ..thin() }
}

pub fn each_protocol(scenario: &Scenario<'_>, mut test: impl FnMut(crate::protocol::Run, u64)) {
    for seed in super::WORLD_SEEDS {
        test(crate::protocol::run(scenario, seed), seed);
    }
}

#[must_use]
pub fn protocol_lines(bytes: &[u8]) -> Vec<&str> {
    core::str::from_utf8(bytes).expect("client UTF-8").lines().collect()
}

#[derive(Clone, Copy, Debug)]
pub struct ProtocolFault {
    pub name: &'static str,
    pub fault: Option<Fault>,
    pub fell: bool,
    pub code: Option<u8>,
}
pub const PROTOCOL_FAULTS: &[ProtocolFault] = &[
    ProtocolFault { name: "normal", fault: None, fell: false, code: Some(0) },
    ProtocolFault {
        name: "killed",
        fault: Some(Fault::KilledAt { stdout_bytes: KILLED_BYTES }),
        fell: true,
        code: Some(75),
    },
    ProtocolFault {
        name: "cache gone",
        fault: Some(Fault::CacheFileGone { sha256: MOTD_KIND_SHA256 }),
        fell: true,
        code: Some(75),
    },
    ProtocolFault { name: "absent", fault: Some(Fault::NotInstalled { absent: true }), fell: true, code: Some(75) },
    ProtocolFault {
        name: "not executable",
        fault: Some(Fault::NotInstalled { absent: false }),
        fell: true,
        code: Some(75),
    },
];
#[must_use]
pub fn swept_fault(seed: u64) -> (&'static ProtocolFault, Scenario<'static>) {
    let position = usize::try_from(seed).expect("seed fits") % PROTOCOL_FAULTS.len();
    let case = PROTOCOL_FAULTS.get(position).expect("fault drawn within the table");
    let scenario = match case.fault {
        Some(Fault::CacheFileGone { .. }) => diff_motd(hearth::Form::JsonLines),
        _ => thin(),
    };
    (case, Scenario { fault: case.fault, ..scenario })
}

#[must_use]
pub fn protocol_records(run: &crate::protocol::Run) -> Vec<hearth_protocol_tree::resolution::Resolution> {
    run.shared
        .borrow()
        .renames
        .iter()
        .filter_map(|rename| {
            if !rename.to.ends_with(b".record") {
                return None;
            }
            let path = [b"/srv/store/resolutions/".as_slice(), rename.to.as_ref()].concat();
            let bytes = run.shared.contents(&path)?;
            Some(
                hearth_protocol_tree::resolution::read(&bytes, &hearth_protocol_tree::limits::SHIPPED)
                    .expect("closure record"),
            )
        })
        .collect()
}

#[must_use]
pub fn diff_record_kinds(bytes: &[u8]) -> Vec<&str> {
    protocol_lines(bytes)
        .into_iter()
        .map(|line| line.strip_prefix(r#"{"record":""#).expect("JSON record").split('"').next().expect("record name"))
        .collect()
}

#[must_use]
pub fn protocol_index_moves(run: &crate::protocol::Run) -> Vec<Vec<u8>> {
    run.shared
        .borrow()
        .renames
        .iter()
        .filter(|rename| rename.from.ends_with(b"_InRelease") || rename.from.ends_with(b"_Packages"))
        .map(|rename| rename.from.to_vec())
        .collect()
}
