/// Filters by name and argument count (protocol/format.md, 7.5).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Filter {
    Default,
    Join,
    Split,
    First,
    Last,
    Length,
    Replace,
    Items,
    Keys,
    Values,
    Sort,
    Tojson,
    ShellQuote,
    SystemdEscape,
    SystemdEscapePath,
    Basename,
    Dirname,
}
impl Filter {
    /// The filter with this grammatical name, if one exists.
    #[must_use]
    pub fn named(name: &[u8]) -> Option<Filter> {
        match name {
            b"default" => Some(Filter::Default),
            b"join" => Some(Filter::Join),
            b"split" => Some(Filter::Split),
            b"first" => Some(Filter::First),
            b"last" => Some(Filter::Last),
            b"length" => Some(Filter::Length),
            b"replace" => Some(Filter::Replace),
            b"items" => Some(Filter::Items),
            b"keys" => Some(Filter::Keys),
            b"values" => Some(Filter::Values),
            b"sort" => Some(Filter::Sort),
            b"tojson" => Some(Filter::Tojson),
            b"shell_quote" => Some(Filter::ShellQuote),
            b"systemd_escape" => Some(Filter::SystemdEscape),
            b"systemd_escape_path" => Some(Filter::SystemdEscapePath),
            b"basename" => Some(Filter::Basename),
            b"dirname" => Some(Filter::Dirname),
            _ => None,
        }
    }
    /// The explicit arguments, excluding the input value.
    #[must_use]
    pub const fn arity(self) -> u32 {
        match self {
            Filter::Default | Filter::Join | Filter::Split => 1,
            Filter::Replace => 2,
            Filter::First
            | Filter::Last
            | Filter::Length
            | Filter::Items
            | Filter::Keys
            | Filter::Values
            | Filter::Sort
            | Filter::Tojson
            | Filter::ShellQuote
            | Filter::SystemdEscape
            | Filter::SystemdEscapePath
            | Filter::Basename
            | Filter::Dirname => 0,
        }
    }
}
