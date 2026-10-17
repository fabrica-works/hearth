use crate::{records::errors as records, render::errors as render};
pub(super) fn format_back(format: records::Format) -> render::Format {
    match format {
        records::Format::Link => render::Format::Link,
        records::Format::Path => render::Format::Path,
        records::Format::Mode => render::Format::Mode,
        records::Format::Duration => render::Format::Duration,
        records::Format::Secret => render::Format::Secret,
        records::Format::File => render::Format::File,
        records::Format::Key => render::Format::Key,
        records::Format::Timestamp => render::Format::Timestamp,
        records::Format::Debian => render::Format::Debian,
    }
}
const FORMATS: &[render::Format] = &[
    render::Format::Link,
    render::Format::Path,
    render::Format::Mode,
    render::Format::Duration,
    render::Format::Secret,
    render::Format::File,
    render::Format::Key,
    render::Format::Timestamp,
    render::Format::Debian,
];
#[test]
fn each_schema_format_translates_and_back() {
    for (row, &input) in FORMATS.iter().enumerate() {
        let back = format_back(crate::translate::errors::format(input));
        assert_eq!(back, input, "row {row}: schema format");
    }
}
