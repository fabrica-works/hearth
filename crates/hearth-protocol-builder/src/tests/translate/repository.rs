use crate::records::errors as record_errors;
use crate::render::errors as render_errors;
use crate::render::revision as target;

fn repository_back(why: record_errors::Repository) -> render_errors::Repository {
    match why {
        record_errors::Repository::NoScript => render_errors::Repository::NoScript,
        record_errors::Repository::NoTemplate => render_errors::Repository::NoTemplate,
        record_errors::Repository::Unused => render_errors::Repository::Unused,
        record_errors::Repository::NoTree => render_errors::Repository::NoTree,
        record_errors::Repository::LinkTarget => render_errors::Repository::LinkTarget,
        record_errors::Repository::Stray => render_errors::Repository::Stray,
        record_errors::Repository::Name => render_errors::Repository::Name,
        record_errors::Repository::Declaration => render_errors::Repository::Declaration,
        record_errors::Repository::Static => render_errors::Repository::Static,
        record_errors::Repository::Destination => render_errors::Repository::Destination,
        record_errors::Repository::Submodule => render_errors::Repository::Submodule,
        record_errors::Repository::Link => render_errors::Repository::Link,
        record_errors::Repository::Environment { env } => render_errors::Repository::Environment { env },
    }
}
const REPOSITORY: &[render_errors::Repository] = &[
    render_errors::Repository::NoScript,
    render_errors::Repository::NoTemplate,
    render_errors::Repository::Unused,
    render_errors::Repository::NoTree,
    render_errors::Repository::LinkTarget,
    render_errors::Repository::Stray,
    render_errors::Repository::Name,
    render_errors::Repository::Declaration,
    render_errors::Repository::Static,
    render_errors::Repository::Destination,
    render_errors::Repository::Submodule,
    render_errors::Repository::Link,
];
#[test]
fn each_repository_kind_translates_and_back() {
    for (row, input) in REPOSITORY.iter().enumerate() {
        let back = repository_back(crate::translate::errors::repository(input.clone()));
        assert_eq!(back, *input, "row {row}: repository kind through records and back");
    }
}
fn ended_back(ended: record_errors::Ended) -> render_errors::Ended {
    match ended {
        record_errors::Ended::Timeout => render_errors::Ended::Timeout,
        record_errors::Ended::Start => render_errors::Ended::Start,
        record_errors::Ended::Exit(code) => render_errors::Ended::Exit(code),
        record_errors::Ended::Signal(signal) => render_errors::Ended::Signal(signal),
        record_errors::Ended::Output => render_errors::Ended::Output,
        record_errors::Ended::Stream => render_errors::Ended::Stream,
        record_errors::Ended::Missing { oid } => render_errors::Ended::Missing { oid: target::Oid(oid) },
    }
}
fn command_back(command: record_errors::Command) -> render_errors::Command {
    match command {
        record_errors::Command::Mmdebstrap => render_errors::Command::Mmdebstrap,
        record_errors::Command::RevParse => render_errors::Command::RevParse,
        record_errors::Command::LsTree => render_errors::Command::LsTree,
        record_errors::Command::CatFile => render_errors::Command::CatFile,
        record_errors::Command::Diff => render_errors::Command::Diff,
    }
}
const TOOLS: &[(render_errors::Command, render_errors::Ended)] = &[
    (render_errors::Command::Mmdebstrap, render_errors::Ended::Timeout),
    (render_errors::Command::Diff, render_errors::Ended::Exit(1)),
    (render_errors::Command::RevParse, render_errors::Ended::Start),
    (render_errors::Command::LsTree, render_errors::Ended::Exit(128)),
    (render_errors::Command::CatFile, render_errors::Ended::Signal(9)),
    (render_errors::Command::CatFile, render_errors::Ended::Output),
    (render_errors::Command::CatFile, render_errors::Ended::Stream),
];
#[test]
fn each_tool_command_and_end_translates_and_back() {
    for (row, (command, ended)) in TOOLS.iter().enumerate() {
        let input = render_errors::Tool { command: *command, ended: ended.clone(), tail: Box::new([]) };
        let record = crate::translate::errors::tool(input.clone());
        let back = render_errors::Tool {
            command: command_back(record.command),
            ended: ended_back(record.ended),
            tail: record.tail,
        };
        assert_eq!(back, input, "row {row}: tool through records and back");
    }
}
