//! Boundary obligations and the store's settled invariants (domain/testing.md, 8.4).
use crate::files::Files;
use skein_io::kernel::{Kind, OpenHow};
use skein_world::domain::Ledger;

#[derive(Debug)]
pub struct Referee {
    requests: Ledger<u64, ()>,
    pub asked: u64,
    pub ended: u64,
}
impl Default for Referee {
    fn default() -> Self {
        Self { requests: Ledger::new("builder world request"), asked: 0, ended: 0 }
    }
}
impl Referee {
    pub fn ask(&mut self, key: u64) {
        self.requests.open(key, ());
        self.asked += 1;
    }
    pub fn end(&mut self, key: u64) {
        self.requests.end(key);
        self.ended += 1;
    }
    pub fn settled(&self, files: &Files) {
        self.requests.assert_settled();
        assert_eq!(self.asked, self.ended, "each request has one terminal");
        assert_eq!(strays(files), 0, "domain/store.md, 11: no temporary and no staging entry");
        let names = files.names(files.roots.cache);
        for entry in &names {
            if let Some(file) = entry.name.strip_suffix(b".record") {
                assert!(
                    names.iter().any(|entry| entry.name.as_ref() == file && entry.kind == Kind::File),
                    "a file for each cache record"
                );
            }
        }
    }
}
#[must_use]
pub fn strays(files: &Files) -> usize {
    let mut count = files.names(files.roots.staging).len();
    for root in [files.roots.store, files.roots.cache, files.roots.resolutions] {
        count += files
            .names(root)
            .iter()
            .filter(|entry| entry.name.ends_with(b".tmp") || entry.name.windows(7).any(|part| part == b".skein-"))
            .count();
    }
    count.saturating_add(layer_strays(files))
}
/// The temporaries and staging layers in `layers/`, which the store clears when a layer fails (4.4).
fn layer_strays(files: &Files) -> usize {
    let Ok(directory) = files.open(files.store_root(), b"layers", OpenHow::Directory) else {
        return 0;
    };
    let entries = files.list(directory, 4096).expect("layer entries");
    files.close(directory);
    entries.iter().filter(|entry| entry.name.ends_with(b".tmp") || entry.name.ends_with(b".staging")).count()
}
