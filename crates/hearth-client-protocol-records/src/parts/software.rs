//! Software records and their fetch hints (render.md, 4.2).
pub use super::config::Script;
use crate::form::text::Lines;
use alloc::boxed::Box;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Software {
    pub downloads: Box<[Download]>,
    pub scripts: Box<[Script]>,
    pub fetch: Box<[Fetch]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Download {
    pub path: Box<[u8]>,
    pub sha256: [u8; 32],
    pub owner: Box<[u8]>,
    pub group: Box<[u8]>,
    pub how: How,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum How {
    File { mode: u16 },
    Tar { strip: u32 },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Fetch {
    pub sha256: [u8; 32],
    pub url: Box<[u8]>,
}
impl Software {
    pub(crate) fn text(&self, lines: &mut Lines) {
        for download in &self.downloads {
            lines.put(b"  download   ");
            lines.put(&download.path);
            match download.how {
                How::File { mode } => {
                    lines.put(b" file ");
                    lines.put(&super::config::octal(mode));
                }
                How::Tar { strip } => {
                    lines.put(b" tar ");
                    lines.decimal(u64::from(strip));
                }
            }
            lines.put(b" ");
            lines.put(&download.owner);
            lines.put(b":");
            lines.put(&download.group);
            lines.put(b" ");
            lines.hex12(&download.sha256);
            lines.put(b"\n");
        }
        for script in &self.scripts {
            lines.put(b"  scripts    ");
            script.text(lines);
            lines.put(b"\n");
        }
        for fetch in &self.fetch {
            lines.put(b"  fetch      ");
            lines.hex12(&fetch.sha256);
            lines.put(b" ");
            lines.put(&fetch.url);
            lines.put(b"\n");
        }
    }
    pub(crate) fn json(&self, json: &mut skein_json::writer::Encoder) {
        json.object_start();
        json.key(b"downloads");
        json.object_start();
        for download in &self.downloads {
            json.key(&download.path);
            json.object_start();
            json.key(b"group");
            json.string(&download.group);
            if let How::File { mode } = download.how {
                json.key(b"mode");
                json.string(&super::config::octal(mode));
            }
            json.key(b"owner");
            json.string(&download.owner);
            json.key(b"sha256");
            crate::form::json::digest(json, &download.sha256);
            if let How::Tar { strip } = download.how {
                json.key(b"strip");
                json.unsigned(u64::from(strip));
            }
            json.key(b"unpack");
            json.string(match download.how {
                How::File { .. } => b"file",
                How::Tar { .. } => b"tar",
            });
            json.object_end();
        }
        json.object_end();
        json.key(b"fetch");
        json.object_start();
        for fetch in &self.fetch {
            json.key(&crate::form::json::digest_hex(&fetch.sha256));
            json.string(&fetch.url);
        }
        json.object_end();
        json.key(b"scripts");
        super::config::scripts_json(&self.scripts, json);
        json.object_end();
    }
}
