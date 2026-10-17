//! Software records translated exhaustively (render.md, 4.2).
use crate::{records::parts::software as source, render::parts::software as target};
#[must_use]
pub(super) fn back(software: source::Software) -> target::Software {
    let source::Software { downloads, scripts, fetch } = software;
    let mut rows = skein_lib::List::with_capacity(u32::try_from(downloads.len()).expect("entry bound"));
    for source::Download { path, sha256, owner, group, how } in downloads {
        let how = match how {
            source::How::File { mode } => target::How::File { mode },
            source::How::Tar { strip } => target::How::Tar { strip },
        };
        rows.push(target::Download { path, sha256: hearth_lib::encode::Digest(sha256), owner, group, how })
            .expect("entry bound");
    }
    let mut hints = skein_lib::List::with_capacity(u32::try_from(fetch.len()).expect("entry bound"));
    for source::Fetch { sha256, url } in fetch {
        hints.push(target::Fetch { sha256: hearth_lib::encode::Digest(sha256), url }).expect("entry bound");
    }
    target::Software {
        downloads: rows.into_boxed(),
        scripts: super::directives::scripts_back(scripts),
        fetch: hints.into_boxed(),
    }
}

#[test]
fn each_software_field_translates_to_the_records_and_back() {
    for (row, input) in super::super::fixtures::software_records().into_iter().enumerate() {
        let translated = crate::translate::parts::software::software(input.clone());
        assert_eq!(back(translated), input, "row {row}: every software field");
    }
}
