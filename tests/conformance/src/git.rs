//! Scenario data shared with fake git (testing.md, 5.2).
#[derive(Clone, Copy, Debug)]
pub struct File {
    pub path: &'static str,
    pub mode: &'static str,
    pub contents: &'static [u8],
    pub length: usize,
}
#[derive(Clone, Copy, Debug)]
pub struct Repository {
    pub name: &'static str,
    pub format: &'static str,
    pub files: &'static [File],
}
pub const REPOSITORIES: &[Repository] = &[
    Repository {
        name: "modes",
        format: "sha1",
        files: &[
            File { path: "a b", mode: "100644", contents: b"hello\n", length: 6 },
            File { path: "big", mode: "100644", contents: b"x", length: 10_000_000 },
            File { path: "empty", mode: "100644", contents: b"", length: 0 },
            File { path: "executable", mode: "100755", contents: b"#!/bin/sh\n", length: 10 },
            File { path: "link", mode: "120000", contents: b"a b", length: 3 },
            File { path: "nested/file", mode: "100644", contents: b"nested\n", length: 7 },
            File { path: "sub", mode: "160000", contents: b"", length: 0 },
            File { path: "é", mode: "100644", contents: b"non-ascii\n", length: 10 },
        ],
    },
    Repository { name: "empty", format: "sha1", files: &[] },
    Repository {
        name: "sha256",
        format: "sha256",
        files: &[File { path: "file", mode: "100644", contents: b"sha256\n", length: 7 }],
    },
];
#[derive(Clone, Copy, Debug)]
pub enum Kind {
    Revision(&'static str),
    Commit,
    Tree,
    Listing,
    Blobs,
    Missing,
}
#[derive(Clone, Copy, Debug)]
pub struct Case {
    pub name: &'static str,
    pub repository: usize,
    pub kind: Kind,
}
impl Case {
    #[must_use]
    pub fn is_listing(&self) -> bool {
        matches!(self.kind, Kind::Listing)
    }
}
pub const CASES: &[Case] = &[
    Case { name: "rev-parse-branch", repository: 0, kind: Kind::Revision("main") },
    Case { name: "rev-parse-commit", repository: 0, kind: Kind::Commit },
    Case { name: "rev-parse-tag", repository: 0, kind: Kind::Revision("v1") },
    Case { name: "rev-parse-missing", repository: 0, kind: Kind::Revision("missing") },
    Case { name: "rev-parse-tree", repository: 0, kind: Kind::Tree },
    Case { name: "rev-parse-sha256", repository: 2, kind: Kind::Revision("main") },
    Case { name: "ls-tree-modes", repository: 0, kind: Kind::Listing },
    Case { name: "ls-tree-empty", repository: 1, kind: Kind::Listing },
    Case { name: "ls-tree-sha256", repository: 2, kind: Kind::Listing },
    Case { name: "cat-file-blobs", repository: 0, kind: Kind::Blobs },
    Case { name: "cat-file-missing", repository: 0, kind: Kind::Missing },
];
