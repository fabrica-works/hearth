use crate::{
    expect::{Expected, Failure},
    world::{self, Settings},
};
use hearth_protocol_git::{batch, limits::Limits, rev_parse};
use hearth_recordings::Recording;
/// Replay with no assertion on malformed output; callers compare any successful result.
#[must_use]
pub fn replay(recording: &Recording, expected: &Expected, limits: &Limits, settings: &Settings) -> Option<Expected> {
    let status = exit(&recording.exit);
    if !succeeded(recording, status) {
        return Some(Expected::Failed(Failure::Exit(status)));
    }
    match expected {
        Expected::Failed(_) => match world::diff(&recording.stdout, limits, settings) {
            world::DiffOutcome::Failed(error) => {
                Some(Expected::Failed(Failure::Decoder(format!("{error:?}").to_ascii_lowercase().into())))
            }
            world::DiffOutcome::Done(_) | world::DiffOutcome::Closed => None,
        },
        Expected::Hunks(_) => match world::diff(&recording.stdout, limits, settings) {
            world::DiffOutcome::Done(hunks) => Some(Expected::Hunks(hunks)),
            world::DiffOutcome::Failed(_) | world::DiffOutcome::Closed => None,
        },
        Expected::Revision(_) => {
            rev_parse::decode(&recording.stdout, exit(&recording.exit)).ok().map(Expected::Revision)
        }
        Expected::Listing(_) => match world::list(&recording.stdout, limits, settings) {
            world::Outcome::Done(entries) => Some(Expected::Listing(entries)),
            world::Outcome::Failed(_) | world::Outcome::Closed => None,
        },
        Expected::Blobs(blobs) => {
            let requests: Vec<_> = blobs
                .iter()
                .map(|b| batch::Request::Read {
                    oid: b.oid,
                    size: b.bytes.as_ref().map_or(0, |b| u64::try_from(b.len()).expect("blob length")),
                    max: 1_048_576,
                })
                .collect();
            match world::read(&recording.stdout, &requests, limits, settings) {
                world::BatchOutcome::Done(blobs) => Some(Expected::Blobs(blobs)),
                world::BatchOutcome::Failed(_) | world::BatchOutcome::Closed => None,
            }
        }
    }
}
#[must_use]
pub fn exit(text: &str) -> hearth_protocol_git::invocation::Exit {
    use hearth_protocol_git::invocation::Exit;
    if let Some(signal) = text.strip_prefix("signal ") {
        Exit::Signal(signal.parse().expect("signal"))
    } else {
        Exit::Code(text.parse().expect("exit code"))
    }
}

fn succeeded(recording: &Recording, status: hearth_protocol_git::invocation::Exit) -> bool {
    use hearth_protocol_git::invocation::{self, Command};
    let revision = invocation::revision(b"main").expect("valid placeholder revision");
    let zero =
        hearth_protocol_git::oid::Oid::parse(b"0000000000000000000000000000000000000000").expect("placeholder id");
    let command = match recording.argv[2].as_slice() {
        b"rev-parse" => Command::RevParse { repository: b"", revision: &revision },
        b"ls-tree" => Command::LsTree { repository: b"", commit: zero },
        b"cat-file" => Command::CatFile { repository: b"" },
        b"diff" => Command::Diff { store: b".", before: &[0; 32], after: &[0; 32] },
        other => panic!("recorded command {other:?}"),
    };
    invocation::succeeded(command, status)
}
