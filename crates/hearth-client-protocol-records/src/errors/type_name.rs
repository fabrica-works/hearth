#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum TypeName {
    Bool,
    Int,
    String,
    Enum,
    List,
    Map,
    Object,
    Path,
    Mode,
    Duration,
    Secret,
    Target,
    File,
}

impl TypeName {
    pub(super) fn article(&self) -> &'static [u8] {
        match self {
            Self::Int | Self::Enum | Self::Object => b"an ",
            Self::Bool
            | Self::String
            | Self::List
            | Self::Map
            | Self::Path
            | Self::Mode
            | Self::Duration
            | Self::Secret
            | Self::Target
            | Self::File => b"a ",
        }
    }
    pub(super) fn name(&self) -> &'static [u8] {
        match self {
            Self::Bool => b"bool",
            Self::Int => b"int",
            Self::String => b"string",
            Self::Enum => b"enum",
            Self::List => b"list",
            Self::Map => b"map",
            Self::Object => b"object",
            Self::Path => b"path",
            Self::Mode => b"mode",
            Self::Duration => b"duration",
            Self::Secret => b"secret",
            Self::Target => b"target",
            Self::File => b"file",
        }
    }
}
