use super::{Box, Encoder, Lines, field};
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Composition {
    Duplicate,
    Long,
    Placement,
    Members,
    Unplaced,
    On { on: Box<[u8]>, host: Host },
    Null,
    Task,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Host {
    Unknown,
    Elsewhere { environment: Box<[u8]> },
    Container,
}
impl Composition {
    pub(super) fn text(&self, lines: &mut Lines, target: Option<&[u8]>, task: Option<&[u8]>) {
        match self {
            Self::Duplicate => {
                lines.put(b"two targets are named ");
                if let Some(target) = target {
                    lines.put(target);
                }
            }
            Self::Long => lines.put(b"a target name past 63 bytes"),
            Self::Placement => lines.put(b"placement in fleet.json, a base or its member"),
            Self::Members => lines.put(b"members in fleet.json or an environment"),
            Self::Unplaced => lines.put(b"no placement once composed"),
            Self::Null => lines.put(b"null for a task in a default"),
            Self::Task => {
                lines.put(b"no task ");
                if let Some(task) = task {
                    lines.put(task);
                }
                lines.put(b" under tasks/");
            }
            Self::On { on, host } => {
                lines.put(b"placement's on names ");
                lines.put(on);
                match host {
                    Host::Unknown => lines.put(b", no discovered target"),
                    Host::Container => lines.put(b", a container"),
                    Host::Elsewhere { environment } => {
                        lines.put(b", a target of ");
                        lines.put(environment);
                    }
                }
            }
        }
    }
    pub(super) fn json(&self, json: &mut Encoder) {
        if let Self::On { on, host } = self {
            json.object_start();
            field(json, b"kind", b"on");
            field(json, b"on", on);
            json.key(b"host");
            match host {
                Host::Unknown => json.string(b"unknown"),
                Host::Container => json.string(b"container"),
                Host::Elsewhere { environment } => {
                    json.object_start();
                    field(json, b"kind", b"elsewhere");
                    field(json, b"environment", environment);
                    json.object_end();
                }
            }
            json.object_end();
        } else {
            json.string(self.name());
        }
    }
    pub(super) fn name(&self) -> &'static [u8] {
        match self {
            Self::Duplicate => b"duplicate",
            Self::Long => b"long",
            Self::Placement => b"placement",
            Self::Members => b"members",
            Self::Unplaced => b"unplaced",
            Self::On { .. } => b"on",
            Self::Null => b"null",
            Self::Task => b"task",
        }
    }
}
