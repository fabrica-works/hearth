//! Software records translated exhaustively (render.md, 4.2).
use crate::{records::parts::software as target, render::parts::software as source};
use alloc::boxed::Box;
#[must_use]
pub fn software(software: source::Software) -> target::Software {
    let source::Software { downloads, scripts, fetch } = software;
    let mut rows = skein_lib::List::with_capacity(u32::try_from(downloads.len()).expect("entry bound"));
    for row in downloads {
        rows.push(download(row)).expect("entry bound");
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

fn download(value: source::Download) -> target::Download {
    let source::Download { path, sha256, owner, group, how } = value;
    target::Download {
        path,
        sha256: sha256.0,
        owner,
        group,
        how: match how {
            source::How::File { mode } => target::How::File { mode },
            source::How::Tar { strip } => target::How::Tar { strip },
        },
    }
}

#[must_use]
pub fn change(value: source::Change) -> target::Change {
    let values = value.downloads;
    let downloads = download_changes(values);
    target::Change { downloads, scripts: scripts(value.scripts) }
}
fn download_changes(values: Box<[source::DownloadKeyed]>) -> Box<[target::DownloadKeyed]> {
    let mut rows = skein_lib::List::with_capacity(u32::try_from(values.len()).expect("entry count"));
    for value in values {
        rows.push(match value {
            source::DownloadKeyed::Added { key, value } => target::DownloadKeyed::Added { key, value: download(value) },
            source::DownloadKeyed::Removed { key, value } => {
                target::DownloadKeyed::Removed { key, value: download(value) }
            }
            source::DownloadKeyed::Changed { key, before, after } => {
                target::DownloadKeyed::Changed { key, before: download(before), after: download(after) }
            }
        })
        .expect("entry count");
    }
    rows.into_boxed()
}

fn scripts(value: Option<crate::render::parts::config::ScriptsChange>) -> Option<crate::records::diff::ScriptsChange> {
    let crate::render::parts::config::ScriptsChange { before, after } = value?;
    Some(crate::records::diff::ScriptsChange {
        before: super::config::scripts(before),
        after: super::config::scripts(after),
    })
}
