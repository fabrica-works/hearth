use crate::errors::{Class, Command, Ended, Error, Kind, Resolution};
#[derive(Clone, Copy, Debug)]
pub enum ResolutionInput {
    Unknown,
    Unsatisfiable,
    Absent,
    Missing,
    Unmatched,
    Disagree,
    TooMany,
    TooLarge,
}
pub fn resolution_error(input: ResolutionInput) -> Error {
    let resolution = match input {
        ResolutionInput::Unknown => Resolution::Unknown { package: b"apt".as_slice().into() },
        ResolutionInput::Unsatisfiable => Resolution::Unsatisfiable { package: Some(b"apt".as_slice().into()) },
        ResolutionInput::Absent => {
            Resolution::Absent { uri: b"http://one/".as_slice().into(), suite: b"forky".as_slice().into() }
        }
        ResolutionInput::Missing => Resolution::Missing { package: b"apt".as_slice().into() },
        ResolutionInput::Unmatched => Resolution::Unmatched {
            name: b"apt".as_slice().into(),
            version: b"3.3.3".as_slice().into(),
            architecture: b"amd64".as_slice().into(),
        },
        ResolutionInput::Disagree => Resolution::Disagree {
            name: b"apt".as_slice().into(),
            version: b"3.3.3".as_slice().into(),
            architecture: b"amd64".as_slice().into(),
        },
        ResolutionInput::TooLarge => Resolution::TooLarge { max: RECORD_BOUND },
        ResolutionInput::TooMany => Resolution::TooMany { max: CLOSURE_LIMIT },
    };
    Error {
        side: None,
        kind: Kind::Resolution(resolution),
        class: match input {
            ResolutionInput::Unmatched | ResolutionInput::Disagree => Class::Operational,
            ResolutionInput::Unknown
            | ResolutionInput::Unsatisfiable
            | ResolutionInput::Absent
            | ResolutionInput::Missing
            | ResolutionInput::TooLarge
            | ResolutionInput::TooMany => Class::Configuration,
        },
        at: None,
        also: None,
        target: None,
        task: None,
    }
}
pub fn mmdebstrap_error(ended: Ended) -> Error {
    let mut error = super::git_failure(Command::Mmdebstrap, 1);
    let Kind::Tool(tool) = &mut error.kind else { panic!("tool fixture") };
    tool.ended = ended;
    tool.tail = Box::new([]);
    error
}
pub const CLOSURE_LIMIT: u32 = 8_192;

pub const RECORD_BOUND: u64 = 4 * 1024 * 1024;
