//! Refusals before an answer can be made (client/README.md, 3.2).
use crate::{
    form::{json::field, text::Lines},
    outcome::Outcome,
};
use alloc::boxed::Box;
use skein_json::writer::Encoder;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Startup {
    Usage { misuse: Misuse, word: Box<[u8]> },
    Unopened { what: Opened, path: Option<Box<[u8]>>, error: Box<[u8]> },
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Misuse {
    Command,
    Option,
    Value,
    Revision,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Opened {
    Store,
    Repository,
}
impl Startup {
    #[must_use]
    pub fn outcome(&self) -> Outcome {
        match self {
            Self::Usage { .. } => Outcome::Usage,
            Self::Unopened { .. } => Outcome::TryAgain,
        }
    }
    pub(crate) fn text(&self, lines: &mut Lines) {
        lines.put(b"hearth: ");
        match self {
            Self::Usage { misuse, word } => {
                lines.put(match misuse {
                    Misuse::Command => b"unknown command: ",
                    Misuse::Option => b"unknown option: ",
                    Misuse::Value => b"option needs a value: ",
                    Misuse::Revision => b"malformed revision: ",
                });
                lines.put(word);
            }
            Self::Unopened { what, path, error } => {
                lines.put(b"cannot open the ");
                lines.put(what.name());
                if let Some(path) = path {
                    lines.put(b" ");
                    lines.put(path);
                }
                lines.put(b": ");
                lines.put(error);
            }
        }
        lines.put(b"\n");
    }
    pub(crate) fn json(&self, json: &mut Encoder) {
        field(json, b"record", b"startup");
        match self {
            Self::Usage { misuse, word } => {
                field(json, b"kind", b"usage");
                field(json, b"outcome", b"usage");
                json.key(b"usage");
                json.object_start();
                field(
                    json,
                    b"misuse",
                    match misuse {
                        Misuse::Command => b"command",
                        Misuse::Option => b"option",
                        Misuse::Value => b"value",
                        Misuse::Revision => b"revision",
                    },
                );
                field(json, b"word", word);
                json.object_end();
            }
            Self::Unopened { what, path, error } => {
                field(json, b"kind", b"unopened");
                field(json, b"outcome", b"try_again");
                json.key(b"unopened");
                json.object_start();
                field(json, b"error", error);
                if let Some(path) = path {
                    field(json, b"path", path);
                }
                field(json, b"what", what.name());
                json.object_end();
            }
        }
    }
}
impl Opened {
    fn name(self) -> &'static [u8] {
        match self {
            Self::Store => b"store",
            Self::Repository => b"repository",
        }
    }
}
