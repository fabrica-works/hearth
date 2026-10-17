use super::{Encoder, Lines, Schema, field, json_names, names, signed, validation};
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Format {
    Path,
    Mode,
    Duration,
    Secret,
    File,
    Key,
    Timestamp,
    Debian,
}
impl Format {
    pub(super) fn name(self) -> &'static [u8] {
        match self {
            Self::Path => b"path",
            Self::Mode => b"mode",
            Self::Duration => b"duration",
            Self::Secret => b"secret",
            Self::File => b"file",
            Self::Key => b"key",
            Self::Timestamp => b"timestamp",
            Self::Debian => b"debian",
        }
    }
}
impl Schema {
    pub(super) fn text(&self, lines: &mut Lines, task: Option<&[u8]>) {
        match self {
            Self::Malformed { format } => {
                lines.put(b"malformed ");
                lines.put(format.name());
            }
            Self::Target { name, environment } => {
                lines.put(b"no target ");
                lines.put(name);
                lines.put(b" in ");
                lines.put(environment);
            }
            Self::File { path } => {
                lines.put(b"no file ");
                lines.put(path);
                lines.put(b" at this revision");
            }
            Self::Large { path, max } => {
                lines.put(path);
                lines.put(b" past ");
                lines.decimal(*max);
                lines.put(b" bytes");
            }
            Self::Main => lines.put(b"debian.components must keep main on"),
            Self::Security => lines.put(b"debian.snapshot.security is required with its pocket"),

            Self::Undeclared { setting, declared } => {
                validation::undeclared(lines, task.unwrap_or(b"task"), setting, declared);
            }
            Self::Missing { setting } => {
                lines.put(b"missing required setting ");
                lines.put(setting);
            }
            Self::Absent { task } => {
                lines.put(b"reads ");
                lines.put(task);
                lines.put(b", which this target lacks");
            }
            Self::Wrong { expected } => {
                lines.put(b"not ");
                lines.put(expected.article());
                lines.put(expected.name());
            }
            Self::Enum { value, listed } => {
                lines.quote(value);
                lines.put(b" is not one of ");
                names(lines, listed);
            }
            Self::Below { min } => {
                lines.put(b"below its min ");
                signed(lines, *min);
            }
            Self::Above { max } => {
                lines.put(b"past its max ");
                signed(lines, *max);
            }
            Self::Length { max_length } => {
                lines.put(b"past its max_length ");
                signed(lines, *max_length);
            }
            Self::Null => lines.put(b"null in a default"),
            Self::UndeclaredField { field, declared } => {
                lines.put(b"undeclared field ");
                lines.quote(field);
                lines.put(b"; the object declares ");
                names(lines, declared);
            }
            Self::MissingField { field } => {
                lines.put(b"missing required field ");
                lines.put(field);
            }

            Self::Task { task } => {
                lines.put(b"reads ");
                lines.put(task);
                lines.put(b", which tasks/ lacks");
            }
            Self::Unread { task, setting, declared } => {
                lines.put(b"reads ");
                lines.put(task);
                lines.put(b".");
                lines.put(setting);
                lines.put(b", which ");
                lines.put(task);
                lines.put(b" does not declare; ");
                lines.put(task);
                if declared.is_empty() {
                    lines.put(b" declares no settings");
                } else {
                    lines.put(b" declares ");
                    for (index, name) in declared.iter().enumerate() {
                        if index != 0 {
                            lines.put(b", ");
                        }
                        lines.put(name);
                    }
                }
            }
        }
    }
    pub(super) fn json(&self, json: &mut Encoder) {
        json.object_start();
        match self {
            Self::Malformed { format } => {
                field(json, b"format", format.name());
                field(json, b"kind", b"malformed");
            }
            Self::Target { name, environment } => {
                field(json, b"environment", environment);
                field(json, b"kind", b"target");
                field(json, b"name", name);
            }
            Self::File { path } => {
                field(json, b"kind", b"file");
                field(json, b"path", path);
            }
            Self::Large { path, max } => {
                field(json, b"kind", b"large");
                json.key(b"max");
                json.unsigned(*max);
                field(json, b"path", path);
            }
            Self::Main => field(json, b"kind", b"main"),
            Self::Security => field(json, b"kind", b"security"),

            Self::Undeclared { setting, declared } => {
                json.key(b"declared");
                json_names(json, declared);
                field(json, b"kind", b"undeclared");
                field(json, b"setting", setting);
            }
            Self::Missing { setting } => {
                field(json, b"kind", b"missing");
                field(json, b"setting", setting);
            }
            Self::Absent { task } => {
                field(json, b"kind", b"absent");
                field(json, b"task", task);
            }
            Self::Wrong { expected } => {
                field(json, b"expected", expected.name());
                field(json, b"kind", b"wrong");
            }
            Self::Enum { value, listed } => {
                field(json, b"kind", b"enum");
                json.key(b"listed");
                json_names(json, listed);
                field(json, b"value", value);
            }
            Self::Below { min } => {
                field(json, b"kind", b"below");
                json.key(b"min");
                json.signed(*min);
            }
            Self::Above { max } => {
                field(json, b"kind", b"above");
                json.key(b"max");
                json.signed(*max);
            }
            Self::Length { max_length } => {
                field(json, b"kind", b"length");
                json.key(b"max_length");
                json.signed(*max_length);
            }
            Self::Null => field(json, b"kind", b"null"),
            Self::UndeclaredField { field: name, declared } => {
                json.key(b"declared");
                json_names(json, declared);
                field(json, b"field", name);
                field(json, b"kind", b"undeclared_field");
            }
            Self::MissingField { field: name } => {
                field(json, b"field", name);
                field(json, b"kind", b"missing_field");
            }

            Self::Task { task } => {
                field(json, b"kind", b"task");
                field(json, b"task", task);
            }
            Self::Unread { task, setting, declared } => {
                json.key(b"declared");
                json.array_start();
                for name in declared {
                    json.string(name);
                }
                json.array_end();
                field(json, b"kind", b"unread");
                field(json, b"setting", setting);
                field(json, b"task", task);
            }
        }
        json.object_end();
    }
}
