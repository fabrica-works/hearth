//! Software records translated exhaustively (render.md, 4.2).
use crate::{records::parts::software as target, render::parts::software as source};
#[must_use]
pub fn software(software: source::Software) -> target::Software {
    let source::Software { downloads, scripts, fetch } = software;
    let mut rows = skein_lib::List::with_capacity(u32::try_from(downloads.len()).expect("entry bound"));
    for source::Download { path, sha256, owner, group, how } in downloads {
        let how = match how {
            source::How::File { mode } => target::How::File { mode },
            source::How::Tar { strip } => target::How::Tar { strip },
        };
        rows.push(target::Download { path, sha256: sha256.0, owner, group, how }).expect("entry bound");
    }
    let mut hints = skein_lib::List::with_capacity(u32::try_from(fetch.len()).expect("entry bound"));
    for source::Fetch { sha256, url } in fetch {
        hints.push(target::Fetch { sha256: sha256.0, url }).expect("entry bound");
    }
    target::Software {
        downloads: rows.into_boxed(),
        scripts: super::config::scripts(scripts),
        fetch: hints.into_boxed(),
    }
}
