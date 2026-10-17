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
        repository.write(files);
        git(&repository.work_tree, path, &["add", "-A"]);
        git(&repository.work_tree, path, &["commit", "--allow-empty", "-q", "-m", "thin"]);
        repository
    }
    fn write(&self, files: &[File]) {
        for file in files {
            let target = self.work_tree.join(file.path);
            fs::create_dir_all(target.parent().expect("a parent")).expect("fixture directories");
            fs::write(&target, file.body).expect("fixture file");
            fs::set_permissions(target, fs::Permissions::from_mode(file.mode)).expect("fixture mode");
        }
    }
    #[must_use]
    pub fn head(&self) -> String {
        let path = std::env::var_os("PATH").expect("git is required on PATH");
        String::from_utf8(git_output(&self.work_tree, &path, &["rev-parse", "HEAD"]))
            .expect("commit id")
            .trim()
            .to_owned()
    }
    #[must_use]
    pub fn next(&self, files: &[File], removed: &[&str]) -> String {
        self.write(files);
        for path in removed {
            fs::remove_file(self.work_tree.join(path)).expect("removed file");
        }
        let path = std::env::var_os("PATH").expect("git is required on PATH");
        git(&self.work_tree, &path, &["add", "-A"]);
        git(&self.work_tree, &path, &["commit", "--allow-empty", "-q", "-m", "next"]);
        self.head()
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
    drop(git_output(directory, path, args));
}
fn git_output(directory: &Path, path: &OsStr, args: &[&str]) -> Vec<u8> {
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
    result.stdout
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
    run(repository, call, form, None)
}
fn run(repository: &Repository, call: Call, form: Form, store: Option<&Path>) -> Rendered {
    run_holding(repository, call, form, store, &hearth::limits(), Release::After(skein_lib::Duration::ZERO)).rendered
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
#[derive(Debug)]
pub struct Asked<'a> {
    pub from: &'a str,
    pub to: &'a str,
    pub targets: &'a [&'a [u8]],
    pub form: Form,
    pub store: Option<&'a Path>,
}
impl<'a> Asked<'a> {
    #[must_use]
    pub const fn new(from: &'a str, to: &'a str) -> Self {
        Self { from, to, targets: &[], form: Form::Text, store: None }
    }
}
#[must_use]
pub fn diff(repository: &Repository, asked: &Asked<'_>) -> Rendered {
    run(
        repository,
        Call::Diff {
            repository: repository.work_tree.join(".git").as_os_str().as_bytes().into(),
            from: asked.from.as_bytes().into(),
            to: asked.to.as_bytes().into(),
            targets: asked.targets.iter().map(|target| Box::<[u8]>::from(*target)).collect(),
        },
        asked.form,
        asked.store,
    )
}
pub const MOTD: &[File] = &[
    File { path: "fleet.json", body: b"{}", mode: 0o644 },
    File { path: "envs/prod.json", body: b"{}", mode: 0o644 },
    File { path: "envs/lab.json", body: b"{}", mode: 0o644 },
    File { path: "hosts/ion/ion.json", body: b"{}", mode: 0o644 },
    File { path: "hosts/x1/x1.json", body: b"{}", mode: 0o644 },
    File {
        path: "hosts/ion/ion-prod.json",
        body: b"{\"placement\":{\"kind\":\"metal\",\"firmware\":\"efi\",\"hardware\":\"7th-gen\"},\"motd\":{}}",
        mode: 0o644,
    },
    File {
        path: "hosts/x1/x1-prod.json",
        body: b"{\"placement\":{\"kind\":\"metal\",\"firmware\":\"efi\",\"hardware\":\"7th-gen\"},\"motd\":{}}",
        mode: 0o644,
    },
    File {
        path: "hosts/ion/ion-lab.json",
        body: b"{\"placement\":{\"kind\":\"metal\",\"firmware\":\"efi\",\"hardware\":\"7th-gen\"}}",
        mode: 0o644,
    },
    File {
        path: "hosts/x1/x1-lab.json",
        body: b"{\"placement\":{\"kind\":\"metal\",\"firmware\":\"efi\",\"hardware\":\"7th-gen\"}}",
        mode: 0o644,
    },
    File { path: "tasks/motd/task.json", body: b"{}", mode: 0o644 },
    File { path: "tasks/motd/tree/etc/motd", body: b"Welcome.\n", mode: 0o644 },
];
pub const MOTD_KIND: File = File { path: "tasks/motd/tree/etc/motd", body: b"Welcome, and be kind.\n", mode: 0o644 };
pub const MOTD_DIFF_TEXT: &str = "ion-prod                                            rebuild from config\n  config   ~ /etc/motd                              0644 root:root\nx1-prod                                             rebuild from config\n  config   ~ /etc/motd                              0644 root:root\n\n2 of 4 targets change; 2 are unchanged.\n";
pub const BAD_ION_LAB: File = File { path: "hosts/ion/ion-lab.json", body: b"not JSON", mode: 0o644 };
pub const BAD_X1_LAB: File = File { path: "hosts/x1/x1-lab.json", body: b"not JSON", mode: 0o644 };
pub const FIXED_X1_LAB: File = File {
    path: "hosts/x1/x1-lab.json",
    body: b"{\"placement\":{\"kind\":\"metal\",\"firmware\":\"efi\",\"hardware\":\"7th-gen\"}}",
    mode: 0o644,
};

#[must_use]
pub fn with_store(store: hearth_domain_store::limits::Limits) -> hearth::Limits {
    hearth::Limits { domain: hearth::service::domain::Limits { store, ..hearth::limits().domain }, ..hearth::limits() }
}
#[must_use]
pub fn render_with(repository: &Repository, limits: &hearth::Limits) -> Rendered {
    render_call(repository, "HEAD", &[], Form::Text, limits, Release::After(skein_lib::Duration::ZERO)).rendered
}
#[derive(Clone, Copy, Debug)]
pub enum Release {
    After(skein_lib::Duration),
}
#[derive(Debug)]
pub struct Held {
    pub rendered: Rendered,
    pub printed_while_held: Vec<u8>,
}
#[must_use]
pub fn render_holding(repository: &Repository, limits: &hearth::Limits, release: Release) -> Held {
    render_call(repository, "HEAD", &[], Form::Text, limits, release)
}

fn render_call(
    repository: &Repository,
    at: &str,
    targets: &[&[u8]],
    form: Form,
    limits: &hearth::Limits,
    release: Release,
) -> Held {
    let call = Call::Render {
        repository: repository.work_tree.join(".git").as_os_str().as_bytes().into(),
        revision: at.as_bytes().into(),
        targets: targets.iter().map(|target| Box::<[u8]>::from(*target)).collect(),
    };
    run_holding(repository, call, form, None, limits, release)
}
fn run_holding(
    repository: &Repository,
    call: Call,
    form: Form,
    store: Option<&Path>,
    limits: &hearth::Limits,
    release: Release,
) -> Held {
    let output = repository.scratch.path().join("stdout");
    let fd = Fd::new(fs::File::create(&output).expect("output file").into_raw_fd());
    let store = store.map_or_else(|| repository.scratch.path().join("store"), Path::to_path_buf);
    fs::create_dir_all(&store).expect("store directory");
    fs::set_permissions(&store, fs::Permissions::from_mode(0o700)).expect("private store");
    let Release::After(span) = release;
    let lock = if span == skein_lib::Duration::ZERO {
        None
    } else {
        let lock = fs::File::open(&store).expect("lock root");
        lock.lock().expect("lock store");
        Some(lock)
    };
    let client = Client::open(Invocation { call, form, store }, limits, fd, 7).expect("client startup");
    let mut held = Holding { client, lock, first: None, span, output: output.clone(), printed: Vec::new() };
    let mut kernel = Kernel::open(Config { operations: held.operations() }).expect("io_uring is required");
    let exit = skein_shell::drive(&mut kernel, &Clock::new(), &mut held);
    Held {
        rendered: Rendered { exit, stdout: fs::read(output).expect("printed output") },
        printed_while_held: held.printed,
    }
}

#[derive(Debug)]
struct Holding {
    client: Client,
    lock: Option<fs::File>,
    first: Option<skein_lib::Time>,
    span: skein_lib::Duration,
    output: PathBuf,
    printed: Vec<u8>,
}
impl Host for Holding {
    fn iterate(&mut self, now: skein_lib::Time, wall: skein_lib::Wall) {
        let first = *self.first.get_or_insert(now);
        if self.lock.is_some() && now >= first.saturating_add(self.span) {
            self.printed = fs::read(&self.output).expect("output while lock held");
            self.lock = None;
        }
        self.client.iterate(now, wall);
    }
    fn completions(&mut self) -> &mut skein_lib::Queue<skein_io::kernel::Complete> {
        self.client.completions()
    }
    fn submissions(&mut self) -> &mut skein_lib::Queue<skein_io::kernel::Submit> {
        self.client.submissions()
    }
    fn work_pending(&self, now: skein_lib::Time) -> bool {
        self.client.work_pending(now)
    }
    fn next_deadline(&self) -> Option<skein_lib::Time> {
        self.client.next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<skein_lib::Time> {
        self.client.next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.client.is_empty()
    }
    fn exit(&self) -> Option<Exit> {
        self.client.exit()
    }
    fn operations(&self) -> u32 {
        self.client.operations()
    }
    fn worst_case(&self) -> u64 {
        self.client.worst_case()
    }
    fn drain(&mut self) {
        self.client.drain();
    }
}

pub const BROKEN_LAB: &[File] = &[
    File { path: "envs/lab.json", body: br#"{"motd": {}, "motd": {}}"#, mode: 0o644 },
    File {
        path: "hosts/x1/x1-lab.json",
        body: br#"{"placement":{"kind":"vm","firmware":"efi","hardware":"incus"}}"#,
        mode: 0o644,
    },
];
/// THIN with a duplicate key in its task declaration.
#[must_use]
pub fn broken_task() -> Vec<File> {
    let mut files = THIN.to_vec();
    files.iter_mut().find(|file| file.path == "tasks/motd/task.json").expect("THIN task").body = br#"{"a": 1, "a": 2}"#;
    files
}
pub const BROKEN_LAB_TEXT: &str = concat!(
    "target       x1-prod   base x1   environment prod\nplacement    metal efi 7th-gen\nspec         9970fb4d8f97\nconfig       part e2475ba7ef36\n  /etc/motd                file 0644 root:root 775324fcea7f\n  /usr/local/bin/motd      file 0755 root:root f487088ec8b1\n",
    "\nenvs/lab.json:1:14: duplicate key \"motd\" (envs/lab.json:1:2)\n"
);
