use hearth::{Call, Client, Form, Invocation};
use skein_io::kernel::{Exit, Fd};
use skein_scratch::Scratch;
use skein_shell::{Clock, Config, Host, Kernel};
use std::{
    ffi::OsStr,
    fs,
    os::{
        fd::IntoRawFd,
        unix::{
            ffi::OsStrExt,
            fs::{DirBuilderExt, PermissionsExt},
        },
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
pub const DEBIAN: &[u8] = br#"{"debian":{"suite":"forky","architecture":"amd64","snapshot":{"archive":"20261001T030405Z"},"pockets":{"updates":false,"security":false}}}"#;
pub const THIN: &[File] = &[
    File { path: "fleet.json", body: DEBIAN, mode: 0o644 },
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
            if file.mode == 0o120_000 {
                if fs::symlink_metadata(&target).is_ok() {
                    fs::remove_file(&target).expect("replace link");
                }
                std::os::unix::fs::symlink(OsStr::from_bytes(file.body), &target).expect("fixture link");
            } else {
                fs::write(&target, file.body).expect("fixture file");
                fs::set_permissions(target, fs::Permissions::from_mode(file.mode)).expect("fixture mode");
            }
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
    pub fn git(&self, args: &[&str]) -> String {
        let path = std::env::var_os("PATH").expect("git is required on PATH");
        String::from_utf8(git_output(&self.work_tree, &path, args)).expect("git text").trim().to_owned()
    }
    pub fn remove_object(&self, oid: &str) {
        let (directory, name) = oid.split_at(2);
        fs::remove_file(self.work_tree.join(".git/objects").join(directory).join(name)).expect("loose object");
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
    pub stderr: Vec<u8>,
}
#[must_use]
pub fn render(repository: &Repository, targets: &[&[u8]], form: Form) -> Rendered {
    render_at(repository, "HEAD", targets, form)
}
#[must_use]
pub fn render_at(repository: &Repository, at: &str, targets: &[&[u8]], form: Form) -> Rendered {
    let call = call_at(repository, at, targets, false);
    run(repository, call, form, None)
}
fn run(repository: &Repository, call: Call, form: Form, store: Option<&Path>) -> Rendered {
    run_holding(repository, call, form, store, &hearth::limits(), Release::After(skein_lib::Duration::ZERO), None)
        .rendered
}

pub const THIN_TEXT: &str = "target       x1-prod   base x1   environment prod\nplacement    metal efi 7th-gen\nspec         23656fdf46b7\nboot         part bc6cab417252\n  mutable    false\npackages     part 64b1b99b813a   resolution 4ee22520f418 needed\n  sources    forky amd64 main 20261001T030405Z\n  variant    apt\n  microcode  generic\n  apt        install_recommends false\nsoftware     part 656d135b92c3\nconfig       part 369415fd653c\n  /etc/motd                file 0644 root:root 775324fcea7f\n  /usr/local/bin/motd      file 0755 root:root f487088ec8b1\ndata         part 2d4765200bab\nsecrets      part ac870b5c968e\nlive         part 6f1b7fda0c21\nexports      part 970579fd41be\n";

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
    File { path: "fleet.json", body: br#"{"debian":{"suite":"forky","architecture":"amd64","snapshot":{"archive":"20261001T030405Z"},"pockets":{"updates":false,"security":false}},"motd":{}}"#, mode: 0o644 },
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
            resolve: false,
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
    File { path: "fleet.json", body: DEBIAN, mode: 0o644 },
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
pub const MOTD_DIFF_TEXT: &str = "ion-prod                                            rebuild from config\n  config   ~ /etc/motd                              0644 root:root\n               @@ -1 +1 @@\n               -Welcome.\n               +Welcome, and be kind.\nx1-prod                                             rebuild from config\n  config   ~ /etc/motd                              0644 root:root\n               @@ -1 +1 @@\n               -Welcome.\n               +Welcome, and be kind.\n\n2 of 4 targets change; 2 are unchanged.\n";
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
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Release {
    OnNotice,
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
    let call = call_at(repository, at, targets, false);
    run_holding(repository, call, form, None, limits, release, None)
}
fn run_holding(
    repository: &Repository,
    call: Call,
    form: Form,
    store: Option<&Path>,
    limits: &hearth::Limits,
    release: Release,
    proxy: Option<Box<[u8]>>,
) -> Held {
    let output = repository.scratch.path().join("stdout");
    let fd = Fd::new(fs::File::create(&output).expect("output file").into_raw_fd());
    let store = store.map_or_else(|| repository.scratch.path().join("store"), Path::to_path_buf);
    fs::create_dir_all(&store).expect("store directory");
    fs::set_permissions(&store, fs::Permissions::from_mode(0o700)).expect("private store");
    let lock = if release == Release::After(skein_lib::Duration::ZERO) {
        None
    } else {
        let lock = fs::File::open(&store).expect("lock root");
        lock.lock().expect("lock store");
        Some(lock)
    };
    let mut client = Client::open(Invocation { call, form, store, proxy }, limits, fd, 7).expect("client startup");
    let error = repository.scratch.path().join("stderr");
    client
        .adopt_error(Fd::new(fs::File::create(&error).expect("error output").into_raw_fd()))
        .expect("error output adoption");
    let mut held = Holding {
        client,
        lock,
        first: None,
        release,
        error: error.clone(),
        output: output.clone(),
        printed: Vec::new(),
    };
    let mut kernel = Kernel::open(Config { operations: held.operations() }).expect("io_uring is required");
    let exit = skein_shell::drive(&mut kernel, &Clock::new(), &mut held);
    Held {
        rendered: Rendered {
            exit,
            stdout: fs::read(output).expect("printed output"),
            stderr: fs::read(error).expect("printed error"),
        },
        printed_while_held: held.printed,
    }
}

#[derive(Debug)]
struct Holding {
    client: Client,
    lock: Option<fs::File>,
    first: Option<skein_lib::Time>,
    release: Release,
    error: PathBuf,
    output: PathBuf,
    printed: Vec<u8>,
}
impl Host for Holding {
    fn iterate(&mut self, now: skein_lib::Time, wall: skein_lib::Wall) {
        let first = *self.first.get_or_insert(now);
        let release = match self.release {
            Release::After(span) => now >= first.saturating_add(span),
            Release::OnNotice => !fs::read(&self.error).expect("notice output").is_empty(),
        };
        if self.lock.is_some() && release {
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
        if let Some(bytes) = self.client.take_startup() {
            use std::io::Write;
            fs::OpenOptions::new()
                .append(true)
                .open(&self.error)
                .expect("startup output")
                .write_all(&bytes)
                .expect("startup written");
        }
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
    "target       x1-prod   base x1   environment prod\nplacement    metal efi 7th-gen\nspec         23656fdf46b7\nboot         part bc6cab417252\n  mutable    false\npackages     part 64b1b99b813a   resolution 4ee22520f418 needed\n  sources    forky amd64 main 20261001T030405Z\n  variant    apt\n  microcode  generic\n  apt        install_recommends false\nsoftware     part 656d135b92c3\nconfig       part 369415fd653c\n  /etc/motd                file 0644 root:root 775324fcea7f\n  /usr/local/bin/motd      file 0755 root:root f487088ec8b1\ndata         part 2d4765200bab\nsecrets      part ac870b5c968e\nlive         part 6f1b7fda0c21\nexports      part 970579fd41be\n",
    "\nenvs/lab.json:1:14: duplicate key \"motd\" (envs/lab.json:1:2)\n"
);
pub const D1: hearth::service::domain::render::vocabulary::Digest =
    hearth::service::domain::render::vocabulary::Digest([1; 32]);
pub const D3: hearth::service::domain::render::vocabulary::Digest =
    hearth::service::domain::render::vocabulary::Digest([3; 32]);
#[derive(Clone, Debug)]
pub enum Laid {
    Resolution(hearth_protocol_tree::resolution::Resolution),
    File(&'static str),
    Record(&'static str, hearth::service::domain::render::vocabulary::Digest),
}
pub fn lay(store: &Path, entries: &[Laid]) {
    use hearth_protocol_tree::{
        cache_file::{self, CacheFile, Origin},
        limits,
    };
    fs::DirBuilder::new().recursive(true).mode(0o700).create(store.join("cache")).expect("cache directory");
    fs::DirBuilder::new().recursive(true).mode(0o700).create(store.join("resolutions")).expect("resolutions directory");
    for entry in entries {
        if let Laid::Resolution(record) = entry {
            let name = hex_digest(record.key.0);
            let bytes = hearth_protocol_tree::resolution::write(record, &limits::SHIPPED).expect("resolution record");
            fs::write(store.join("resolutions").join(format!("{name}.record")), bytes).expect("laid resolution");
            continue;
        }
        let (name, bytes) = match entry {
            Laid::Resolution(_) => unreachable!("written above"),
            Laid::File(name) => (*name, b"fixture".to_vec()),
            Laid::Record(name, digest) => (
                *name,
                cache_file::write(
                    &CacheFile {
                        at: skein_lib::Wall::EPOCH,
                        origin: Origin::Rendered { path: b"fixture".as_slice().into() },
                        sha256: *digest,
                        size: 7,
                        took: skein_lib::Duration::ZERO,
                    },
                    &limits::SHIPPED,
                )
                .expect("cache record")
                .into_vec(),
            ),
        };
        fs::write(store.join(name), bytes).expect("laid file");
    }
}
#[must_use]
pub fn names(directory: &Path) -> Vec<String> {
    let mut names: Vec<_> = fs::read_dir(directory)
        .expect("directory")
        .map(|entry| entry.expect("entry").file_name().into_string().expect("name"))
        .collect();
    names.sort();
    names
}

/// A containing work tree whose configuration must not affect diff's cache files.
pub fn work_tree(at: &Path, config: &[(&str, &str)]) {
    fs::create_dir(at).expect("containing work tree");
    let path = std::env::var_os("PATH").expect("git is required on PATH");
    git(at, &path, &["init", "--quiet", "--initial-branch=main"]);
    for (name, value) in config {
        git(at, &path, &["config", name, value]);
    }
}
pub const TWENTY_LINES: File = File { body: b"line 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\nline 8\nline 9\nline 10\nline 11\nline 12\nline 13\nline 14\nline 15\nline 16\nline 17\nline 18\nline 19\nline 20\n", ..MOTD_KIND };
pub const TWENTY_LINES_TWO_CHANGED: File = File { body: b"line 1\nline two\nline 3\nline 4\nline 5\nline 6\nline 7\nline 8\nline 9\nline 10\nline 11\nline twelve\nline 13\nline 14\nline 15\nline 16\nline 17\nline 18\nline 19\nline 20\n", ..MOTD_KIND };
#[must_use]
pub fn diff_beneath_work_tree(label: &str, attributes: bool) -> Rendered {
    let repository = Repository::make(label, MOTD);
    let from = repository.next(&[TWENTY_LINES], &[]);
    let to = repository.next(&[TWENTY_LINES_TWO_CHANGED], &[]);
    let containing = repository.scratch().path().join("containing");
    work_tree(&containing, &[("diff.interHunkContext", "10")]);
    if attributes {
        fs::write(containing.join(".gitattributes"), b"* -diff\n").expect("attributes");
        let path = std::env::var_os("PATH").expect("git is required on PATH");
        git(&containing, &path, &["add", ".gitattributes"]);
        git(&containing, &path, &["commit", "-q", "-m", "attributes"]);
    }
    let store = containing.join("store");
    diff(&repository, &Asked { store: Some(&store), targets: &[b"x1-prod"], ..Asked::new(&from, &to) })
}

/// THIN and independently broken environments, including environments with no target.
#[must_use]
pub fn broken_envs(count: u32) -> Repository {
    let repository = Repository::make("broken-envs", THIN);
    for index in 0..count {
        fs::write(repository.work_tree.join(format!("envs/e{index:03}.json")), b"{\"motd\": {}, \"motd\": {}}")
            .expect("broken environment");
    }
    drop(repository.next(&[], &[]));
    repository
}

pub const MISKEYED: File = File { path: "tasks/motd/task.json", body: b"{\"setting\": {}}", mode: 0o644 };

pub const MISREAD: &[File] = &[
    File { path: "tasks/motd/task.json", body: br#"{"reads": {"core": {"settings": ["domian"]}}}"#, mode: 0o644 },
    File {
        path: "tasks/core/task.json",
        body: br#"{"settings":{"domain":{"type":"string","doc":"Domain."}}}"#,
        mode: 0o644,
    },
];

pub const MISTYPED: File = File {
    path: "tasks/motd/task.json",
    body: br#"{"settings": {"greeting": {"type": "int", "default": "hi", "doc": "The greeting."}}}"#,
    mode: 0o644,
};

/// One package and source line accepted by the resolution codec.
#[must_use]
pub fn resolution(
    key: hearth::service::domain::render::vocabulary::Digest,
    variant: &[u8],
    digest: hearth::service::domain::render::vocabulary::Digest,
) -> hearth_protocol_tree::resolution::Resolution {
    use hearth_lib::encode::Item;
    use hearth_protocol_tree::resolution::{Costs, Inputs, Line, Made, Package, Resolution};
    Resolution {
        key,
        digest,
        inputs: Inputs {
            sources: Box::new([Item::Map(0)]),
            request: Box::new([Item::List(1), Item::Bytes(b"hello".as_slice().into())]),
            variant: Box::new([Item::Bytes(variant.into())]),
            apt: Box::new([Item::Map(0)]),
        },
        closure: Box::new([Package {
            name: b"apt".as_slice().into(),
            architecture: b"amd64".as_slice().into(),
            version: b"3.0.3".as_slice().into(),
            source: 0,
            filename: b"pool/apt.deb".as_slice().into(),
            size: 7,
            sha256: D1,
        }]),
        sources: Box::new([Line {
            uri: b"https://deb.example/debian".as_slice().into(),
            suite: b"stable".as_slice().into(),
            components: Box::new([b"main".as_slice().into()]),
            indices: Box::new([]),
        }]),
        made: Made {
            apt: b"3.0.3".as_slice().into(),
            at: skein_lib::Wall::EPOCH,
            costs: Costs { fetched: 0, took: skein_lib::Duration::ZERO },
            mmdebstrap: b"1.5".as_slice().into(),
        },
    }
}
pub const RESOLUTION_KEY: hearth::service::domain::render::vocabulary::Digest =
    hearth::service::domain::render::vocabulary::Digest([
        1, 234, 129, 26, 252, 153, 48, 138, 159, 204, 51, 108, 148, 73, 102, 199, 249, 221, 152, 133, 123, 39, 89, 47,
        224, 10, 31, 184, 75, 212, 157, 0,
    ]);
pub const MINBASE_KEY: hearth::service::domain::render::vocabulary::Digest =
    hearth::service::domain::render::vocabulary::Digest([
        163, 63, 187, 161, 226, 131, 202, 148, 42, 188, 123, 35, 118, 177, 142, 72, 113, 210, 142, 118, 110, 225, 160,
        241, 233, 4, 112, 215, 184, 46, 33, 187,
    ]);
pub const CLOSURE: hearth::service::domain::render::vocabulary::Digest =
    hearth::service::domain::render::vocabulary::Digest([
        242, 251, 234, 142, 207, 175, 92, 30, 225, 209, 84, 99, 193, 172, 188, 52, 212, 154, 210, 160, 229, 114, 68,
        15, 124, 191, 52, 95, 192, 174, 126, 227,
    ]);
pub const STRAY: File = File { path: "tasks/motd/tre/etc/motd", body: b"stray", mode: 0o644 };

#[must_use]
pub fn notice_limits() -> hearth::Limits {
    with_store(hearth_domain_store::limits::Limits {
        said: skein_lib::Duration::from_millis(100),
        retry: skein_lib::Duration::from_millis(20),
        ..hearth_domain_store::limits::SHIPPED
    })
}

pub const BROKEN_TEMPLATE: File =
    File { path: "tasks/motd/tree/etc/issue.j2", body: b"Host:\n{{ target.name\n", mode: 0o644 };

pub const CLUSTER: &[File] = &[
    File { path: "fleet.json", body: DEBIAN, mode: 0o644 },
    File {
        path: "envs/lab.json",
        body: br#"{"placement":{"kind":"vm","firmware":"efi","hardware":"incus"}}"#,
        mode: 0o644,
    },
    File { path: "hosts/stoa/stoa.json", body: br#"{"motd":{},"members":{"1":{},"2":{},"3":{}}}"#, mode: 0o644 },
    File {
        path: "hosts/stoa/stoa-lab.json",
        body: br#"{"members":{"2":{"placement":{"kind":"vm","firmware":"bios","hardware":"pool"}},"3":null}}"#,
        mode: 0o644,
    },
];
#[must_use]
pub fn cluster() -> Vec<File> {
    let mut files = CLUSTER.to_vec();
    for file in THIN {
        if file.path.starts_with("tasks/") {
            files.push(*file);
        }
    }
    files
}
pub const USERS: &[File] = &[
    File { path: "tasks/users/task.json", mode: 0o644, body: br#"{"settings":{"accounts":{"type":{"map":{"object":{"uid":{"type":"int","min":1000,"max":59999,"doc":"The account's id."},"shell":{"type":"path","default":"/bin/bash","doc":"The login shell."},"groups":{"type":{"list":"string"},"default":[],"doc":"Supplementary groups."},"password":{"type":"secret","optional":true,"doc":"The password."}}}},"default":{},"doc":"Accounts, by user name."}}}"# },
    File { path: "hosts/x1/x1.json", mode: 0o644, body: b"{\"motd\": {}, \"users\": {\"accounts\": {\n\"ann\": {\"uid\": \"1000\"},\n\"bob\": {\"uid\": 1001, \"shel\": \"/bin/sh\"},\n\"cy\": {}}}}" },
];

pub const CONTAINERS: &[File] = &[
    File { path: "fleet.json", body: DEBIAN, mode: 0o644 },
    File { path: "envs/prod.json", body: b"{}", mode: 0o644 },
    File { path: "envs/lab.json", body: b"{}", mode: 0o644 },
    File { path: "hosts/rhi/rhi.json", body: b"{}", mode: 0o644 },
    File {
        path: "hosts/rhi/rhi-prod.json",
        body: br#"{"placement":{"kind":"vm","firmware":"bios","hardware":"serverhino"}}"#,
        mode: 0o644,
    },
    File {
        path: "hosts/rhi/rhi-lab.json",
        body: br#"{"placement":{"kind":"vm","firmware":"efi","hardware":"incus"}}"#,
        mode: 0o644,
    },
    File { path: "hosts/chat/chat.json", body: b"{}", mode: 0o644 },
    File {
        path: "hosts/chat/chat-prod.json",
        body: br#"{"placement": {"kind": "container", "on": "rhi-lab"}}"#,
        mode: 0o644,
    },
];

pub const TEMPLATED: &[File] = &[
    File { path: "tasks/motd/tree/etc/hostname.j2", body: b"{{ target.name }}\n", mode: 0o644 },
    File { path: "tasks/motd/tree/etc/issue.j2", body: b"{% set kind = target.placement.kind %}{{ target.base }} runs on {{ kind }}{% if target.member is defined %} as {{ target.member }}{% endif %}.\n{% for word in ['one', 'two'] %}{{ loop.index }}: {{ word }}\n{% endfor %}", mode: 0o644 },
];
pub const MISSING_TEMPLATE_KEY: File =
    File { path: "tasks/motd/tree/etc/issue.j2", body: b"{{ target.nam }}\n", mode: 0o644 };
pub const GREET: &[File] = &[
    File { path: "hosts/x1/x1.json", body: br#"{"motd":{},"greet":{}}"#, mode: 0o644 },
    File {
        path: "tasks/greet/task.json",
        body: br#"{"settings":{"greeting":{"type":"string","default":"hello","doc":"The greeting."}}}"#,
        mode: 0o644,
    },
    File { path: "tasks/greet/tree/etc/greeting.j2", body: b"{{ settings.greeting }}\n", mode: 0o644 },
];

pub const REFERENCES: &[File] = &[
    File { path: "tasks/net/task.json", mode: 0o644, body: br#"{"settings":{"ca":{"type":"file","doc":"The certificate."},"peer":{"type":"target","doc":"The peer."}}}"# },
    File { path: "hosts/x1/x1.json", mode: 0o644, body: br#"{"motd": {}, "net": {"ca": "lab-ca/lab.crt", "peer": "x9-prod"}}"# },
];

pub const RENDERED_FILES: &[(&str, &[u8])] = &[
    ("21c50adeb849aa6df1c779b5464f84f614f2b3775ef6d2ffb2ad0f0a1d650011", b"x1-prod\n"),
    ("e75f92b4b8419984111db313564627b1ee90ecb312dbffd66a1a42f64e6e2334", b"x1 runs on metal.\n1: one\n2: two\n"),
];

#[must_use]
pub fn target_output_limit() -> hearth::Limits {
    let mut limits = hearth::limits();
    limits.domain.render.rendered_target = TARGET_OUTPUT_LIMIT;
    limits
}

pub const TARGET_OUTPUT_LIMIT: u64 = 8;

pub const GUARDED: File = File { path: "tasks/motd/task.json", mode: 0o644, body: b"{\"settings\": {\"greeting\": {\"type\": \"string\", \"default\": \"hello\", \"doc\": \"The greeting.\"}},\n\"requires\": {\n    \"greeting\": {\"if\": \"settings.greeting != 'hello'\", \"message\": \"set a greeting of your own\"}}}" };
pub const GREETING: File = File { path: "hosts/x1/x1.json", mode: 0o644, body: br#"{"motd":{"greeting":"hi"}}"# };
pub const WRONG_GREETING: File = File { path: "hosts/x1/x1.json", mode: 0o644, body: br#"{"motd":{"greeting":1}}"# };

pub const TWICE: File = File { path: "tasks/motd/task.json", mode: 0o644, body: b"{\"settings\": {\"teams\": {\"type\": {\"list\": \"string\"}, \"default\": [\"ops\", \"dev\", \"ops\"], \"doc\": \"Teams.\"}},\n\"groups\": {\n    \"{{ value }}\": {\"for_each\": \"settings.teams\"}}}" };
pub const DISTINCT_TEAMS: File =
    File { path: "hosts/x1/x1.json", mode: 0o644, body: br#"{"motd":{"teams":["ops","dev"]}}"# };

pub const UNSET: File = File { path: "tasks/motd/task.json", mode: 0o644, body: b"{\"settings\": {\"origin\": {\"type\": \"string\", \"optional\": true, \"doc\": \"Where from.\"}},\n\"groups\": {\n    \"ops\": {\"from\": \"{{ settings.origin }}\"},\n    \"dev\": {\"from\": \"x{{ settings.origin }}\"}}}" };
pub const BOOLEAN_FIELD: File = File { path: "tasks/motd/task.json", mode: 0o644, body: b"{\"settings\": {\"enabled\": {\"type\": \"bool\", \"default\": true, \"doc\": \"Enabled.\"}},\n\"groups\": {\n    \"ops\": {\"from\": \"{{ settings.enabled }}\"}}}" };

pub const TYPO: File = File { path: "tasks/motd/task.json", mode: 0o644, body: b"{\"settings\": {\"greeting\": {\"type\": \"string\", \"default\": \"hello\", \"doc\": \"The greeting.\"}},\n\"requires\": {\n    \"known\": {\"if\": \"settings.greting != ''\", \"message\": \"set one\"}}}" };
pub const TYPO_BRANCH: &[File] = &[
 File { path: "tasks/motd/task.json", mode: 0o644, body: b"{\"settings\": {\"greeting\": {\"type\": \"string\", \"default\": \"hello\", \"doc\": \"The greeting.\"}}}" },
 File { path: "tasks/motd/tree/etc/issue.j2", mode: 0o644, body: b"{% if false %}{{ settings.greting }}{% endif %}\n" },
];

pub const CONTAINED: &[File] = &[
    File { path: "envs/prod.json", body: b"{}", mode: 0o644 },
    File { path: "envs/lab.json", body: b"{}", mode: 0o644 },
    File { path: "hosts/rhi/rhi.json", body: br#"{"inventory":{}}"#, mode: 0o644 },
    File { path: "hosts/rhi/rhi-prod.json", body: br#"{"placement":{"kind":"vm","firmware":"bios","hardware":"serverhino"}}"#, mode: 0o644 },
    File { path: "hosts/rhi/rhi-lab.json", body: br#"{"placement":{"kind":"vm","firmware":"efi","hardware":"incus"}}"#, mode: 0o644 },
    File { path: "hosts/chat/chat.json", body: br#"{"inventory":{}}"#, mode: 0o644 },
    File { path: "hosts/chat/chat-prod.json", body: br#"{"placement":{"kind":"container","on":"rhi-prod"}}"#, mode: 0o644 },
    File { path: "hosts/mail/mail-prod.json", body: br#"{"placement":{"kind":"container","on":"rhi-prod"}}"#, mode: 0o644 },
    File { path: "hosts/mail/mail-lab.json", body: br#"{"placement":{"kind":"container","on":"rhi-lab"}}"#, mode: 0o644 },
    File { path: "tasks/inventory/task.json", body: b"{}", mode: 0o644 },
    File { path: "tasks/inventory/tree/etc/containers.j2", body: b"{% for name, placement in target.containers | items %}{{ name }}={{ placement.kind }}@{{ placement.on }};{% endfor +%}\n", mode: 0o644 },
];

#[must_use]
pub fn contained() -> Rendered {
    let repository = Repository::make("contained", &thin_with(CONTAINED));
    render(&repository, &[b"chat-prod", b"rhi-lab", b"rhi-prod"], Form::Text)
}

pub const DEBIAN_LAYERS: &[File] = &[
    File { path: "fleet.json", body: br#"{"debian":{"suite":"trixie","architecture":"amd64","snapshot":{"archive":"20261001T030405Z"},"pockets":{"updates":false,"security":false}}}"#, mode: 0o644 },
    File { path: "hosts/x1/x1.json", body: br#"{"motd":{},"debian":{"suite":"forky","components":{"contrib":true}}}"#, mode: 0o644 },
    File { path: "hosts/x1/x1-prod.json", body: br#"{"placement":{"kind":"metal","firmware":"efi","hardware":"7th-gen"},"debian":{"components":{"non-free-firmware":true}}}"#, mode: 0o644 },
    File { path: "tasks/motd/tree/etc/debian.j2", body: b"{{ debian.suite }} {{ debian.architecture }} {{ debian.snapshot.archive }} {{ debian.components | keys | join(',') }}\n", mode: 0o644 },
];

pub const REFUSED_DEBIAN: File =
    File { path: "hosts/x1/x1.json", body: br#"{"motd":{},"debian":{"components":{"main":false}}}"#, mode: 0o644 };

/// Secrets, identity and a password travel through the real declaration decoder.
pub const LIVE: File = File { path: "tasks/motd/task.json", body: br#"{"live":{"postfix":{"unit":"postfix.service","action":"reload","paths":["/etc/postfix/","/etc/mailname"]},"ripgrep":{"action":"none","paths":["/opt/ripgrep/"]}}}"#, mode: 0o644 };

pub const SECRETS: File = ids::SECRETS_FIXED;
pub const SECRET_NO_PATH: File =
    File { path: "tasks/motd/task.json", body: br#"{"secrets": {"mail/tls/mx1.key": {}}}"#, mode: 0o644 };

impl File {
    #[must_use]
    pub const fn link(path: &'static str, target: &'static [u8]) -> Self {
        Self { path, body: target, mode: 0o120_000 }
    }
}
mod worlds;
pub use worlds::*;
mod exports;
pub mod files;
mod packages;
pub use exports::*;
pub use packages::*;

/// Version, spec, meta, boot, packages, software, config, data, secrets, live, exports and outcome.
pub const THIN_RECORDS: usize = 12;
mod resolution;
pub use resolution::*;
mod vpn;
pub use vpn::*;
pub mod accounts;

pub const DISTINCT_TEAMS_TEXT: &str = "target       x1-prod   base x1   environment prod\nplacement    metal efi 7th-gen\nspec         44c412f39f88\nboot         part bc6cab417252\n  mutable    false\npackages     part 64b1b99b813a   resolution 4ee22520f418 needed\n  sources    forky amd64 main 20261001T030405Z\n  variant    apt\n  microcode  generic\n  apt        install_recommends false\nsoftware     part 656d135b92c3\nconfig       part d8c246718666\n  groups     dev -\n  groups     ops -\n  /etc/motd                file 0644 root:root 775324fcea7f\n  /usr/local/bin/motd      file 0755 root:root f487088ec8b1\ndata         part 2d4765200bab\nsecrets      part ac870b5c968e\nlive         part 6f1b7fda0c21\nexports      part 970579fd41be\n";

pub mod additions;
pub mod data;
pub mod scripts;
pub mod software;

mod diff_resolution;
pub use diff_resolution::*;
pub mod ids;

pub mod collisions;
pub mod part_changes;
