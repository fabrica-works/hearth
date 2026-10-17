use crate::render::parts::software::{Download, Fetch, How, Script, Software};
#[must_use]
pub fn software_records() -> [Software; 2] {
    [How::File { mode: 0o755 }, How::Tar { strip: 2 }].map(|how| Software {
        downloads: Box::new([Download {
            path: b"/opt/tool".as_slice().into(),
            sha256: hearth_lib::encode::Digest([3; 32]),
            owner: b"tool".as_slice().into(),
            group: b"staff".as_slice().into(),
            how,
        }]),
        scripts: Box::new([Script {
            task: b"motd".as_slice().into(),
            order: 400,
            content_digest: hearth_lib::encode::Digest([4; 32]),
        }]),
        fetch: Box::new([Fetch {
            sha256: hearth_lib::encode::Digest([3; 32]),
            url: b"https://example.org/tool".as_slice().into(),
        }]),
    })
}
