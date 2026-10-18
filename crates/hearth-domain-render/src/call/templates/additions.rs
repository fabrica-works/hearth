//! Stream render's generated files after the task templates.
use super::{Digest, Keeping, Request, Side, Target, Token, TreeEntry, parts};
use alloc::boxed::Box;
use skein_lib::bytes::copy_of;
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
