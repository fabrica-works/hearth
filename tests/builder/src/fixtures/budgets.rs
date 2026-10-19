//! Session 47's fleet edits, request counts and program counts.
use super::{File, Scenario};
use crate::domain;
use hearth_domain_builder::vocabulary::{Call, Request};
use hearth_domain_render::{revision::Oid, vocabulary::Request as RenderRequest};
use hearth_render_world::golden;
use std::collections::BTreeSet;

/// Session 47 compares three generated repositories through eight pipe schedules.
pub const CASES: core::ops::Range<u64> = 0..3;
/// A revision is listed once (domain/render.md, 3.10).
pub const RENDER_LISTS: usize = 1;
/// A diff lists its before and after revisions.
pub const DIFF_LISTS: usize = RENDER_LISTS + RENDER_LISTS;
/// Both git discovery processes precede the batched blob reads (domain/render.md, 2.2).
pub const DISCOVERY_SPAWNS: usize = 2;
pub const FLEET_EDIT: File = File {
    path: "hosts/x1/x1-prod.json",
    body: br#"{"placement":{"kind":"metal","firmware":"efi","hardware":"8th-gen"},"debian":{"components":{"non-free-firmware":true}},"firmware":{}}"#,
    mode: 0o644,
};
pub const FLEET_DUPLICATE: File = File { path: "tasks/core/tree/etc/golden/shared", body: b"enabled\n", mode: 0o644 };
#[must_use]
pub fn fleet() -> Vec<File> {
    super::files_of(&golden::load("fleet").tree)
}
#[must_use]
pub fn fleet_run(diff: bool) -> domain::Run {
    let mut files = fleet();
    files.push(FLEET_DUPLICATE);
    let call = if diff {
        Call::Diff {
            resolve: false,
            repository: super::GIT_DIR.into(),
            from: b"before".as_slice().into(),
            to: b"HEAD".as_slice().into(),
            targets: Box::default(),
        }
    } else {
        super::thin().call
    };
    domain::Run::new(&Scenario { files: &files, call, ..super::thin() }, super::SIMULATION_SEED).finish()
}
#[derive(Debug, Default)]
pub struct Reads {
    pub lists: usize,
    pub batches: usize,
    pub blobs: Vec<Oid>,
    pub second: Vec<Oid>,
}
#[must_use]
pub fn reads(run: &domain::Run) -> Reads {
    let mut reads = Reads::default();
    for request in run.requests() {
        if let Request::Render(RenderRequest::List { .. }) = request {
            reads.lists += 1;
        }
        if let Request::Render(RenderRequest::Read { blobs, .. }) = request {
            reads.batches += 1;
            reads.blobs.extend(blobs.iter().map(|blob| blob.oid.clone()));
            if reads.lists == DIFF_LISTS {
                reads.second.extend(blobs.iter().map(|blob| blob.oid.clone()));
            }
        }
    }
    reads
}
#[must_use]
pub fn distinct(blobs: &[Oid]) -> usize {
    blobs.iter().map(|oid| oid.0.as_ref()).collect::<BTreeSet<_>>().len()
}
/// Hosted children are spawned by the simulator's kernel, whose ledger is
/// its trace; the pure fake sees each hosted child's subsequent file calls.
#[must_use]
pub fn programs(trace: &[skein_sim::Entry]) -> (usize, usize) {
    (super::tool_spans(trace, b"/usr/bin/git").len(), super::tool_spans(trace, b"/usr/bin/mmdebstrap").len())
}
/// The fleet's 35 original objects plus one duplicate listing entry.
pub const FLEET_BLOBS: usize = 35;
pub const FLEET_ENTRIES: usize = FLEET_BLOBS + 1;
/// The after revision contributes one changed production document.
pub const DIFF_BLOBS: usize = FLEET_BLOBS + 1;
#[derive(Clone, Copy, Debug)]
pub enum Repository {
    Thin,
    Fleet,
    Resolution,
}
pub struct Programs {
    pub repository: Repository,
    pub git: usize,
    pub mmdebstrap: usize,
}
pub const PROGRAMS: &[Programs] = &[
    Programs { repository: Repository::Thin, git: 4, mmdebstrap: 0 },
    Programs { repository: Repository::Fleet, git: 4, mmdebstrap: 0 },
    Programs { repository: Repository::Resolution, git: 3, mmdebstrap: 0 },
];
#[must_use]
pub fn files(repository: Repository) -> Vec<File> {
    match repository {
        Repository::Thin => super::THIN.to_vec(),
        Repository::Fleet => fleet(),
        Repository::Resolution => super::files_of(&super::resolution_golden().tree),
    }
}
#[must_use]
pub fn scenario<'a>(repository: Repository, files: &'a [File], laid: &'a [super::Laid]) -> Scenario<'a> {
    match repository {
        Repository::Thin | Repository::Fleet => Scenario { files, ..super::thin() },
        Repository::Resolution => Scenario {
            files,
            laid,
            call: Call::Render {
                resolve: true,
                repository: super::GIT_DIR.into(),
                revision: b"HEAD".as_slice().into(),
                targets: Box::new([b"test-min".as_slice().into()]),
            },
            ..super::thin()
        },
    }
}

/// Git blob hash of `FLEET_EDIT`, independently encoded once with Python hashlib.
pub const FLEET_EDIT_OID: &[u8] = b"2223ad179798b47326209edecf2cbc0d712d5846";
