//! Stream render's generated files after the task templates.
use super::{Digest, Keeping, Request, Side, Target, Token, TreeEntry, parts};
use alloc::boxed::Box;
use skein_lib::{List, Map, bytes::copy_of};
impl Target {
    pub(super) fn addition(&mut self, side: &mut Side) -> bool {
        for stage in
            [Addition::Subuid, Addition::Subgid, Addition::Initramfs].into_iter().skip(usize::from(self.addition))
        {
            self.addition = self.addition.saturating_add(1);
            let (path, bytes) = match stage {
                Addition::Subuid => (b"/etc/subuid".as_slice(), self.subordinate.clone()),
                Addition::Subgid => (b"/etc/subgid".as_slice(), self.subordinate.take()),
                Addition::Initramfs => {
                    (b"/etc/tiny-initramfs/tiny-initramfs.conf".as_slice(), initramfs(&self.packages))
                }
            };
            let Some(bytes) = bytes else { continue };
            let digest = Digest(skein_lib::sha256::digest(&bytes));
            side.sizes.insert(digest, u64::try_from(bytes.len()).expect("file bound")).expect("addition bound");
            self.keeping = Some(if stage == Addition::Initramfs {
                Keeping::Initramfs(digest)
            } else {
                Keeping::File(TreeEntry {
                    path: copy_of(path),
                    node: parts::config::Node::File {
                        content_digest: digest,
                        mode: 0o644,
                        owner: copy_of(b"root"),
                        group: copy_of(b"root"),
                    },
                })
            });
            self.request = Some(Request::Keep { request: Token::new(4), path: copy_of(path), sha256: digest, bytes });
            side.ready = false;
            return true;
        }
        false
    }
}
pub(super) fn tmpfiles(
    expansion: &crate::entries::Expansion,
    tmpfiles: Box<[parts::config::Tmpfile]>,
) -> Box<[parts::config::Tmpfile]> {
    let located = expansion.of(crate::revision::declaration::Map::Data);
    let count = u32::try_from(located.len()).expect("entry bound");
    let mut entries = List::with_capacity(count);
    for entry in &located {
        entries.push(crate::kinds::data::entry(entry).expect("data checked before rendering")).expect("entry bound");
    }
    let mut rows = Map::with_capacity(count.saturating_add(u32::try_from(tmpfiles.len()).expect("entry bound")));
    for row in tmpfiles {
        rows.insert(row.path.clone(), row).expect("entry bound");
    }
    for line in crate::additions::data_lines(entries.as_slice()) {
        if !rows.contains_key(&line.tmpfile.path) {
            rows.insert(line.tmpfile.path.clone(), line.tmpfile).expect("entry bound");
        }
    } // transitional: hearth 22.4 compares task lines and generated data lines.
    let mut lines = List::with_capacity(rows.len());
    for (_, row) in &rows {
        lines.push(row.clone()).expect("entry bound");
    }
    lines.into_boxed()
}

#[derive(PartialEq, Eq)]
enum Addition {
    Subuid,
    Subgid,
    Initramfs,
}

fn initramfs(packages: &parts::packages::Packages) -> Option<Box<[u8]>> {
    let initramfs = packages.initramfs.as_ref()?;
    Some(crate::additions::initramfs(initramfs))
}
