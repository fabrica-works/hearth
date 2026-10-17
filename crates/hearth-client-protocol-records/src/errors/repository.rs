use super::{Box, Encoder, Lines, field};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Repository {
    NoTree,
    NoTemplate,
    Unused,
    LinkTarget,
    Stray,
    Environment { env: Box<[u8]> },
    Name,
    Declaration,
    Static,
    Destination,
    Submodule,
    Link,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Tool {
    pub command: Command,
    pub ended: Ended,
    pub tail: Box<[Box<[u8]>]>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Command {
    Mmdebstrap,
    RevParse,
    LsTree,
    CatFile,
    Diff,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Ended {
    Timeout,
    Start,
    Exit(u8),
    Signal(u32),
    Output,
    Missing { oid: Box<[u8]> },
    Stream,
}

impl Repository {
    pub(super) fn text(&self, lines: &mut Lines) {
        match self {
            Self::NoTemplate => lines.put(b"no template at this path"),
            Self::Unused => lines.put(b"a template no entry names"),
            Self::NoTree => lines.put(b"no tree entry at this path"),
            Self::LinkTarget => lines.put(b"a symbolic link target out of form"),
            Self::Stray => lines.put(b"a path format.md 2.1 does not allow"),
            Self::Name => lines.put(b"a name format.md 2.4 does not allow"),
            Self::Declaration => lines.put(b"a task directory without task.json"),
            Self::Static => lines.put(b"a template without a destination name"),
            Self::Destination => lines.put(b"two sources of one destination"),
            Self::Submodule => lines.put(b"a submodule under a path render reads"),
            Self::Link => lines.put(b"a document or template listed as a symbolic link"),
            Self::Environment { env } => {
                lines.put(b"no environment ");
                lines.put(env);
                lines.put(b" under envs/");
            }
        }
    }
    pub(super) fn json(&self, json: &mut Encoder) {
        match self {
            Self::Environment { env } => {
                json.object_start();
                field(json, b"kind", b"environment");
                field(json, b"env", env);
                json.object_end();
            }
            Self::NoTemplate => json.string(b"no_template"),
            Self::Unused => json.string(b"unused"),
            Self::NoTree => json.string(b"no_tree"),
            Self::LinkTarget => json.string(b"link_target"),
            Self::Stray => json.string(b"stray"),
            Self::Name => json.string(b"name"),
            Self::Declaration => json.string(b"declaration"),
            Self::Static => json.string(b"static"),
            Self::Destination => json.string(b"destination"),
            Self::Submodule => json.string(b"submodule"),
            Self::Link => json.string(b"link"),
        }
    }
}
impl Command {
    fn name(self) -> &'static [u8] {
        match self {
            Self::Mmdebstrap => b"mmdebstrap",
            Self::RevParse => b"rev_parse",
            Self::LsTree => b"ls_tree",
            Self::CatFile => b"cat_file",
            Self::Diff => b"diff",
        }
    }
    pub(super) fn text(self) -> &'static [u8] {
        match self {
            Self::Mmdebstrap => b"mmdebstrap",
            Self::RevParse => b"git rev-parse",
            Self::LsTree => b"git ls-tree",
            Self::CatFile => b"git cat-file",
            Self::Diff => b"git diff",
        }
    }
}
impl Tool {
    pub(super) fn text(&self, lines: &mut Lines) {
        lines.put(self.command.text());
        match &self.ended {
            Ended::Timeout => lines.put(b" timed out"),
            Ended::Start => lines.put(b" could not start"),
            Ended::Exit(code) => {
                lines.put(b" exited ");
                lines.decimal(u64::from(*code));
            }
            Ended::Signal(signal) => {
                lines.put(b" was killed by signal ");
                lines.decimal(u64::from(*signal));
            }
            Ended::Output => lines.put(b" gave invalid output"),
            Ended::Missing { oid } => {
                lines.put(b" is missing object ");
                lines.put(oid);
            }
            Ended::Stream => lines.put(b" failed to read or write a pipe"),
        }
    }
    pub(super) fn json(&self, json: &mut Encoder) {
        json.object_start();
        field(json, b"command", self.command.name());
        json.key(b"ended");
        json.object_start();
        match &self.ended {
            Ended::Timeout => field(json, b"kind", b"timeout"),
            Ended::Start => field(json, b"kind", b"start"),
            Ended::Exit(code) => {
                json.key(b"code");
                json.unsigned(u64::from(*code));
                field(json, b"kind", b"exit");
            }
            Ended::Signal(signal) => {
                json.key(b"signal");
                json.unsigned(u64::from(*signal));
                field(json, b"kind", b"signal");
            }
            Ended::Output => field(json, b"kind", b"output"),
            Ended::Missing { oid } => {
                field(json, b"kind", b"missing");
                field(json, b"oid", oid);
            }
            Ended::Stream => field(json, b"kind", b"stream"),
        }
        json.object_end();
        json.key(b"tail");
        json.array_start();
        for line in &self.tail {
            json.string(line);
        }
        json.array_end();
        json.object_end();
    }
}
