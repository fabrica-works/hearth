use hearth::{Call, Client, Form, Invocation};
use skein_io::kernel::{Exit, Fd};
use skein_scratch::Scratch;
use skein_shell::{Clock, Config, Host, Kernel};
use std::{
    ffi::OsStr,
    fs,
    os::{
        fd::IntoRawFd,
        unix::{ffi::OsStrExt, fs::PermissionsExt},
    },
    path::{Path, PathBuf},
    process::Command,
};
#[derive(Clone, Copy, Debug)]
pub struct File {
    pub path: &'static str,
    pub body: &'static [u8],
    pub mode: u32,
}
pub const THIN: &[File] = &[
    File { path: "fleet.json", body: b"{}", mode: 0o644 },
    File { path: "envs/prod.json", body: b"{}", mode: 0o644 },
    File { path: "hosts/x1/x1.json", body: b"{\"motd\":{}}", mode: 0o644 },
    File {
        path: "hosts/x1/x1-prod.json",
        body: b"{\"placement\":{\"kind\":\"metal\",\"firmware\":\"efi\",\"hardware\":\"7th-gen\"}}",
        mode: 0o644,
    },
    File { path: "tasks/motd/task.json", body: b"{}", mode: 0o644 },
    File { path: "tasks/motd/tree/etc/motd", body: b"Welcome to x1.\n", mode: 0o644 },
    File { path: "tasks/motd/tree/usr/local/bin/motd", body: b"#!/bin/sh\nexec cat /etc/motd\n", mode: 0o755 },
];
#[derive(Debug)]
pub struct Repository {
    scratch: Scratch,
    work_tree: PathBuf,
}
impl Repository {
    #[must_use]
    pub fn make(label: &str, files: &[File]) -> Self {
        let path = std::env::var_os("PATH").expect("git is required on PATH");
        Self::make_on(&path, label, files)
    }
    #[must_use]
    pub fn make_on(path: &OsStr, label: &str, files: &[File]) -> Self {
        let repository = Self::empty(label);
        git(&repository.work_tree, path, &["init", "--quiet", "--initial-branch=main"]);
        for file in files {
            let target = repository.work_tree.join(file.path);
            fs::create_dir_all(target.parent().expect("a parent")).expect("fixture directories");
            fs::write(&target, file.body).expect("fixture file");
            fs::set_permissions(target, fs::Permissions::from_mode(file.mode)).expect("fixture mode");
        }
        git(&repository.work_tree, path, &["add", "-A"]);
        git(&repository.work_tree, path, &["commit", "--allow-empty", "-q", "-m", "thin"]);
        repository
    }
    #[must_use]
    pub fn empty(label: &str) -> Self {
        let scratch = Scratch::new(label);
        let work_tree = scratch.path().join("repo");
        fs::create_dir(&work_tree).expect("empty repository directory");
        Self { scratch, work_tree }
    }
    #[must_use]
    pub fn work_tree(&self) -> &Path {
        &self.work_tree
    }
    #[must_use]
    pub fn scratch(&self) -> &Scratch {
        &self.scratch
    }
}
fn git(directory: &Path, path: &OsStr, args: &[&str]) {
    let result = Command::new("git")
        .current_dir(directory)
        .args(args)
        .env_clear()
        .env("PATH", path)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("HOME", "/dev/null")
        .env("GIT_AUTHOR_NAME", "hearth")
        .env("GIT_AUTHOR_EMAIL", "hearth@hearth.invalid")
        .env("GIT_AUTHOR_DATE", "1700000000 +0000")
        .env("GIT_COMMITTER_NAME", "hearth")
        .env("GIT_COMMITTER_EMAIL", "hearth@hearth.invalid")
        .env("GIT_COMMITTER_DATE", "1700000000 +0000")
        .output()
        .expect("git is required on the supplied PATH");
    assert!(result.status.success(), "git {args:?} failed: {}", String::from_utf8_lossy(&result.stderr));
}
#[derive(Debug)]
pub struct Rendered {
    pub exit: Exit,
    pub stdout: Vec<u8>,
}
#[must_use]
pub fn render(repository: &Repository, targets: &[&[u8]], form: Form) -> Rendered {
    render_at(repository, "HEAD", targets, form)
}
#[must_use]
pub fn render_at(repository: &Repository, at: &str, targets: &[&[u8]], form: Form) -> Rendered {
    let call = Call::Render {
        repository: repository.work_tree.join(".git").as_os_str().as_bytes().into(),
        revision: at.as_bytes().into(),
        targets: targets.iter().map(|target| Box::<[u8]>::from(*target)).collect(),
    };
    let output = repository.scratch.path().join("stdout");
    let fd = Fd::new(fs::File::create(&output).expect("output file").into_raw_fd());
    let limits = hearth::limits();
    let mut client =
        Client::open(Invocation { call, form, store: repository.scratch.path().join("store") }, &limits, fd, 7)
            .expect("client startup");
    let mut kernel = Kernel::open(Config { operations: client.operations() }).expect("io_uring is required");
    let exit = skein_shell::drive(&mut kernel, &Clock::new(), &mut client);
    Rendered { exit, stdout: fs::read(output).expect("printed output") }
}

pub const THIN_TEXT: &str = "target       x1-prod   base x1   environment prod\nplacement    metal efi 7th-gen\nspec         9970fb4d8f97\nconfig       part e2475ba7ef36\n  /etc/motd                file 0644 root:root 775324fcea7f\n  /usr/local/bin/motd      file 0755 root:root f487088ec8b1\n";

mod binary;
pub use binary::{Ended, binary};
/// Overrides THIN by path and appends the new files.
#[must_use]
pub fn thin_with(files: &[File]) -> Vec<File> {
    let mut combined = THIN.to_vec();
    for file in files {
        if let Some(previous) = combined.iter_mut().find(|previous| previous.path == file.path) {
            *previous = *file;
        } else {
            combined.push(*file);
        }
    }
    combined
}
pub const LAYERS: &[File] = &[
    File { path: "fleet.json", body: br#"{"motd":{}}"#, mode: 0o644 },
    File { path: "envs/prod.json", body: br#"{"issue":{}}"#, mode: 0o644 },
    File { path: "hosts/x1/x1.json", body: br#"{"motd":{},"issue":{}}"#, mode: 0o644 },
    File {
        path: "hosts/x1/x1-prod.json",
        body: br#"{"issue":null,"placement":{"kind":"metal","firmware":"efi","hardware":"7th-gen"}}"#,
        mode: 0o644,
    },
    File { path: "hosts/x2/x2.json", body: b"{}", mode: 0o644 },
    File {
        path: "hosts/x2/x2-prod.json",
        body: br#"{"placement":{"kind":"vm","firmware":"bios","hardware":"ionos"}}"#,
        mode: 0o644,
    },
    File { path: "tasks/issue/task.json", body: b"{}", mode: 0o644 },
    File { path: "tasks/issue/tree/etc/issue", body: b"An issue\n", mode: 0o644 },
];
#[must_use]
pub fn lines_starting(stdout: &[u8], prefixes: &[&str]) -> String {
    let mut lines = String::new();
    for line in String::from_utf8_lossy(stdout).lines() {
        if prefixes.iter().any(|prefix| line.starts_with(prefix)) {
            lines.push_str(line);
            lines.push('\n');
        }
    }
    lines
}
