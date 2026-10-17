# Session 08: the test mirror and its one fetch

testing.md 6.2's local mirror at M1, indexes only. 08.1 starts a kit,
`testing/hearth-test-mirror` (`hearth-test-mirror`), that reads the pin and
finds the fetched mirror by its digest; 08.2 writes `testing/mirror/`'s
fetch script, runs it once against snapshot.debian.org (the plan's one live
service), commits the pin it makes, and lays the test repository
`testing/repos/test/`, whose targets are `test-min` and `test-boot`; 08.3
serves the mirror on loopback as an HTTP proxy. hearth 41 runs real
mmdebstrap through that proxy and records it; hearth 38.2 replays the
recordings; hearth 46 runs the same cases on its fake. Nothing in the gate
reads the cache or reaches past loopback: the kit's gate tests lay a
mirror of their own.

## Read first

- hearth's testing.md: 2 (2.8, 2.9), 4.5, 6.2, 9.2, 11 ("The local
  mirror"); format.md: 2.1, 3.1, 3.3, 3.4, 5.3 (`debian`), 5.12;
  client/README.md, 3.5.
- This plan's README, "Live services"; decisions.md, 4, 12, 14, 16;
  `findings/skein-testing.md`, 6, 7.
- skein's `testing/skein-scratch/src/lib.rs:23-55` (`Scratch::new`, `path`),
  `tests/clippy.toml` (the kit copies it), `tests/json/Cargo.toml:17-33` (an
  ordinary-Rust lints table); `programming-model.md`: 10.2;
  `testing-strategy.md`: 4.1, 5, 9 (by number only).

## What the design asks

- **The pin** (testing.md, 6.2): "a snapshot timestamp (the canonical one
  snapshot.debian.org gives), the suite, components and architecture, and
  the requests of the test repository's targets: `test-min` (the essential
  variant and apt), and `test-boot` for the VM tier (a kernel,
  tiny-initramfs, systemd, systemd-container, openssh-server, incus-agent,
  and the tools hearth runs: mmdebstrap, uidmap, efibootmgr, git)"; "A bump
  is a commit".
- **At M1:** "a snapshot of forky, `main`, amd64, without the updates and
  security pockets; the suite's `InRelease` and its `Packages` index, whole
  and signed, so apt verifies them"; `test-min` and `test-boot` are targets
  of `testing/repos`.
- **The fetch:** "Fetched once, online, by a script ... into a cache
  directory named by the mirror digest, outside the repository, each file
  checked as any fetch is (`InRelease`, then SHA-256). A run whose digest is
  present fetches nothing".
- **The proxy** (6.2, H10): "Served on loopback as an HTTP proxy, by the
  harness, at a port it chooses: it answers each request for
  `http://snapshot.debian.org/archive/…` with the file the mirror holds at
  that path (`by-hash` paths included), and any other request with 404";
  it is "the builder's configuration, made by the test (client/README.md,
  3.5), and never in a key"; sources are format.md, 5.3's fixed URLs.

## Choices this plan makes

- **The mirror is a kit, `testing/hearth-test-mirror`,** not a module of the
  conformance world: its proxy answers on a thread of its own while the
  test's thread runs mmdebstrap, which `tests/clippy.toml` (every world's)
  bans and a kit's own `clippy.toml` may let in, since conformance does not
  replay (testing.md, 2.8); and hearth 46's fake reads the same pin and
  index (decisions.md, 14), as M2's fetch scenario and real tier serve the
  same proxy (6.2). Ordinary Rust, std only: a lints table as skein's
  `tests/json/Cargo.toml:17-33` plus `missing_errors_doc = "allow"` (a kit
  returns `Result`s), and `clippy.toml` skein's `tests/clippy.toml`
  at `f981746` less its `std::thread::spawn` line only (`Builder::spawn` and
  `scope` stay), its comment saying why.
- **The pin, `testing/mirror/pin`:** `<field> <value>…` lines and `#`
  comments: `archive`, `suite`, `architecture`, `components` (`main`
  first), `digest`, and `target <name> <variant> <package>…` per target, its
  packages in strictly increasing byte order. `fetch --pin` writes `archive`
  and `digest`; the implementer writes the rest.
- **The mirror digest at M1 is `InRelease`'s SHA-256** (testing.md 6.2), kept
  in the pin, so a run names its directory before it fetches.
- **The cache:** `$XDG_CACHE_HOME/hearth-test-mirror/<digest>/`
  (`$HOME/.cache/…` when it is unset or not absolute). Served, below it:
  `archive/debian/<archive>/dists/<suite>/InRelease`,
  `…/main/binary-amd64/Packages.xz`, and
  `…/main/binary-amd64/by-hash/SHA256/<its SHA-256>`, a hard link. Never
  served: `checked/main/binary-amd64/Packages`, the index decompressed and
  checked against `InRelease`'s entry, which hearth 41.1 holds apt's list
  to and hearth 46 loads as its universe.
- **The compressed index is `Packages.xz`:** the first of
  `Acquire::CompressionTypes` in x1's `apt-config dump`, and 9,993,660 bytes
  in x1's own forky lists at `20260928T022309Z` (read locally, not fetched):
  the README's "about 10 MB". A 404 apt's run fails on is a fetch the next
  budgeted run adds (Rules).
- **The fetch, `testing/mirror/fetch`:** POSIX sh with `curl`, `sqv` against
  `/usr/share/keyrings/debian-archive-keyring.pgp` (x1 has `sqv`, not
  `gpgv`), `xz` and `sha256sum`; it refuses to start naming a missing one.
  `--pin` asks for `…/archive/debian/<now, from date -u>/dists/<suite>/InRelease`
  and takes the canonical timestamp from the URL it is redirected to
  (format.md, 5.3: snapshot.debian.org "redirects any other to the nearest
  earlier one").
- **The test repositories' layout** (code-organization.md, 3: hearth
  08's): `testing/repos/<repository>/` holds one repository's files at one
  revision, format.md 2.1's layout and nothing else; a later revision is
  `<repository>@<k>/`, whole; a test commits one into a scratch repository
  (hearth 11's fixture) or lays it into fake git (hearth 44). M1's is
  `testing/repos/test/`: base `test` in environments `min` and `boot`, the
  one way format.md 3.1 makes `test-min` and `test-boot` from one base.

## Rules that bind it

- **Live services** (README): "snapshot.debian.org alone, by hearth 08.2's
  fetch script ... Budget: one run of the script, two files (about 10 MB);
  a run that fails may be repeated twice, after a change aimed at the cause
  its output names; a fourth run is a question." A run under `unshare
  --map-root-user --net` reaches nothing and is not one of them. Each commit
  whose work ran the script carries `Fetch-Runs: <n>` and `Fetch-Bytes:
  <n>` trailers.

## Increments

### 08.1 The pin, and the fetched mirror found by its digest
Scope: the kit; the pin's form (6.2); the cache's layout and its lookup.
Needs: nothing.
Files: new `testing/hearth-test-mirror/`: `Cargo.toml` (std only;
dev-dependency `skein-scratch`), `clippy.toml`, `src/{lib.rs, pin.rs,
mirror.rs, tests.rs}`, `src/tests/{fixtures.rs, pin.rs, mirror.rs}`; shared:
root `Cargo.toml` (the `testing/hearth-*` glob, if absent;
`[workspace.dependencies]`' `hearth-test-mirror`, and `skein-scratch` if
absent), `Cargo.lock`.
Shape: the sketch's `pin` and `mirror`. `Pin::read` takes each field once
but `target`, and refuses an unknown field, an `archive` not
`YYYYMMDDTHHMMSSZ`, a `digest` not 64 lowercase hex digits, components
without `main` first, a request not in strictly increasing order, a field
missing (line 0). `Mirror::file` answers a regular file below
`<root>/archive/`, reached through no symbolic link and no `.`, `..` or
empty segment; anything else is none.
Test shape: `src/tests/pin.rs` and `src/tests/mirror.rs` on
`src/tests/fixtures.rs` (new: `PIN`, the text below; `laid(dir) -> Mirror`,
the cache's layout in a `Scratch` with `InRelease` `signed\n`,
`Packages.xz` `xz\n` and its by-hash link, `checked/…/Packages`), each under
20 ms: `a_pin_reads_its_snapshot_suite_and_targets` (reference),
`a_pin_out_of_form_is_refused_naming_its_line` (a `const` table of texts and
`PinError`s), `the_mirror_is_found_under_the_cache_by_its_digest`,
`a_mirror_not_fetched_names_the_fetch_script`,
`a_snapshot_path_names_its_file_beneath_the_mirror` (reference),
`a_dot_segment_a_link_or_the_checked_index_is_no_file`.

```rust
// testing/hearth-test-mirror/src/tests/fixtures.rs: the digest is sha256sum of `signed\n`, the hash of `xz\n`.
pub const PIN: &[u8] = b"# a pin\narchive 20261016T091523Z\nsuite forky\narchitecture amd64\ncomponents main\n\
digest ad093de033f89b590459503f94d25ea4a36fd044f7ce6d674f43e0d8412161f9\ntarget test-min apt\ntarget test-boot apt efibootmgr git\n";
pub const XZ_SHA256: &str = "b44f7d6b5283a44ee5f2bd98f84087a04810092122d75e8fbf8ad85f8f2981f1";

// testing/hearth-test-mirror/src/tests/pin.rs
#[test]
fn a_pin_reads_its_snapshot_suite_and_targets() {
    let pin = Pin::read(fixtures::PIN).expect("the fixture is in form");
    assert_eq!((&*pin.archive, &*pin.suite, &*pin.architecture), ("20261016T091523Z", "forky", "amd64"));
    let min = pin.target("test-min").expect("test-min is pinned");
    assert_eq!((&*min.variant, min.request.len()), ("apt", 0), "the essential variant and apt");
    let boot = pin.target("test-boot").expect("test-boot is pinned");
    let request: Vec<&str> = boot.request.iter().map(|name| &**name).collect();
    assert_eq!(request, ["efibootmgr", "git"]);
}

// testing/hearth-test-mirror/src/tests/mirror.rs
#[test]
fn a_snapshot_path_names_its_file_beneath_the_mirror() {
    let scratch = Scratch::new("test-mirror");
    let mirror = fixtures::laid(scratch.path());
    let path = Path::new("archive/debian/20261016T091523Z/dists/forky/main/binary-amd64/by-hash/SHA256").join(fixtures::XZ_SHA256);
    let file = mirror.file(&path).expect("the by-hash link is served");
    assert_eq!(fs::read(file).unwrap(), b"xz\n");
}
```
Proof: unit, `the_mirror_is_found_under_the_cache_by_its_digest`: a pin
leads to its directory under a given cache; bound 1 s for the crate.
Nothing calls the kit yet: 08.2's tests read the committed pin, hearth
41.1 the fetched mirror.
Don't: fetch, hash or decompress (the script's); read the environment but
`XDG_CACHE_HOME` and `HOME`, by `var_os`; name a hearth crate.
Size: at most 170 lines. Removes: nothing.

### 08.2 The pin made, the fetch run once, and the test repository
Scope: `testing/mirror/fetch`; its one run, making the pin's timestamp and
digest (6.2); `testing/repos/test/` (6.2; format.md, 2.1, 3, 5.3).
Needs: 08.1 (the pin's form).
Files: new `testing/mirror/fetch` (mode 0755), `testing/mirror/pin`;
`testing/repos/test/` (shared): `fleet.json`, `envs/{min,boot}.json`,
`hosts/test/{test,test-min,test-boot}.json`, `tasks/hearth/task.json`,
`tasks/boot/task.json`; the kit's `src/tests.rs`, `src/tests/committed.rs`
(new), `src/tests/fixtures.rs` (`repository(name)`: `../repos/<name>` beside
the crate).
Shape:
- The script, `set -eu`: tools checked; the cache (Choices); without
  `--pin`, `<cache>/<digest>/` present prints `present: <dir>` and exits 0.
  Else into `<cache>/.fetch-<pid>/`: `InRelease` (with `--pin` from
  `<now>`, through its redirect); `sqv --cleartext --output=`; from the
  signed text, the `SHA256` entries of `main/binary-<arch>/Packages.xz` and
  `…/Packages`; `Packages.xz` by its plain path, checked by size and SHA-256;
  `xz -dc` into `checked/`, checked against `…/Packages`'s entry; the
  by-hash link; without `--pin`, `InRelease`'s SHA-256 must be the pin's
  `digest`. Then renamed to `<cache>/<digest>/`; `--pin` rewrites `archive`
  and `digest`. Each curl: `--fail --location --max-redirs 2 --write-out
  '%{url_effective} %{size_download}\n'`; it prints each line, then the
  total bytes and the directory. Components loop over the pin's. Each
  curl also takes `--connect-timeout 20 --max-time 120`.
- The repository: `fleet.json` `{"debian": {"suite": "forky", "architecture":
  "amd64", "snapshot": {"archive": "<the pin's>"}, "pockets": {"updates":
  false, "security": false}}}`; each environment `{"placement": {"kind":
  "vm", "firmware": "efi", "hardware": "incus"}}`; `test.json` and
  `test-min.json` `{}`; `test-boot.json` `{"boot": {}}`; `tasks/boot` a `doc`
  and test-boot's ten packages, each `{}`; `tasks/hearth` a `doc` alone
  (testing.md 6.2: test-min is the `apt` variant, no package). The pin's
  targets: `test-min apt`, and `test-boot apt` with
  `efibootmgr git incus-agent linux-image-amd64 mmdebstrap openssh-server
  systemd systemd-container tiny-initramfs uidmap`.
Test shape: `src/tests/committed.rs`, each under 20 ms:
`the_committed_pin_names_test_min_and_test_boot`,
`the_test_repository_names_the_pin_s_snapshot_timestamp` (reference),
`the_boot_task_declares_test_boot_s_request` (each name a `"<name>": {}` in
`tasks/boot/task.json`),
`the_test_repository_holds_the_eight_files_above_and_nothing_else` (the
file list of `testing/repos/test/` equals them). Text checks until hearth
24 renders the targets.

```rust
#[test]
fn the_test_repository_names_the_pin_s_snapshot_timestamp() {
    let pin = Pin::committed();
    let fleet = fs::read_to_string(fixtures::repository("test").join("fleet.json")).expect("test's fleet.json");
    let archive = format!("\"archive\": \"{}\"", pin.archive);
    assert!(fleet.contains(&archive), "debian.snapshot.archive is the pin's {}", pin.archive);
}
```
Proof: live: `testing/mirror/fetch --pin` once, on the run's machine: it
prints `…/<archive>/dists/forky/InRelease` (about 150 KB) and
`…/main/binary-amd64/Packages.xz` (about 10 MB), their total and the
directory; then `unshare --map-root-user --net testing/mirror/fetch` once
prints `present: <that directory>` and exits 0, with no network to reach;
then `heavy cargo nextest run -p hearth-test-mirror` passes. Bound: 5 min;
one run reaching snapshot.debian.org, two files. The commit's trailers give
the runs and the bytes the script printed.
Don't: fetch beyond the two files; a `.gitignore` or any fetched file in
the repository; a run beyond the budget; the hearth task's packages or
download in the test repository (testing.md 6.2).
Size: no counted lines (`testing/mirror/*` and `testing/repos/*` are left
out, code-organization.md, 7); the script at most 130 lines, a guide.
Removes: nothing.

### 08.3 The mirror served on loopback as an HTTP proxy
Scope: 6.2's proxy, over the mirror 08.1 finds.
Needs: 08.1.
Files: `testing/hearth-test-mirror/src/{lib.rs, proxy.rs}`,
`src/tests/{fixtures.rs (get: a request sent on a TcpStream, read to its
end), proxy.rs (new)}`.
Shape: the sketch's `proxy`. `serve` binds `127.0.0.1:0` and spawns one
thread: accept, read the head to its blank line (at most `HEAD`, 10 s),
answer, log, close. `target` takes `GET` or `HEAD` of `ORIGIN` followed by
`archive/…`, in absolute form, with no `?`, `%` or `#`; `Mirror::file`
then names the file. A file is `HTTP/1.1 200 OK`, `Content-Length`,
`Connection: close`, the bytes (none for `HEAD`); anything else `HTTP/1.1
404 Not Found`, `Content-Length: 0`, `Connection: close`. `Range` and
conditions are ignored. Each answer is logged before its connection closes.
Test shape: `src/tests/proxy.rs`, each under 20 ms:
`a_snapshot_request_names_its_path_below_the_origin`,
`another_host_origin_form_or_another_method_is_not_found` (a `const`
table), `the_proxy_serves_a_laid_mirror_over_loopback` (reference),
`a_by_hash_request_is_answered_the_file_it_names`,
`a_head_request_is_answered_the_head_alone`,
`each_answer_is_logged_before_its_connection_closes`,
`a_stalled_head_is_answered_404` (`serve`'s inner function takes the
head's timeout: 50 ms here, 10 s from `serve`).

```rust
#[test]
fn the_proxy_serves_a_laid_mirror_over_loopback() {
    let scratch = Scratch::new("test-mirror-proxy");
    let proxy = Proxy::serve(&fixtures::laid(scratch.path())).expect("a loopback port");
    let answer = fixtures::get(&proxy, "http://snapshot.debian.org/archive/debian/20261016T091523Z/dists/forky/InRelease");
    assert_eq!(answer, b"HTTP/1.1 200 OK\r\nContent-Length: 7\r\nConnection: close\r\n\r\nsigned\n");
}
```
Proof: process, the kit's tests on loopback in the gate,
`the_proxy_serves_a_laid_mirror_over_loopback`: a TCP client is answered
the laid `InRelease`'s bytes; bound 0.1 s. Nothing above calls the proxy
until hearth 41.1, where real apt reads the fetched mirror through it.
Don't: bind beyond `127.0.0.1`; serve `checked/`; keep a connection open;
share state with the thread but its log's channel.
Size: at most 110 lines. Removes: nothing.

## Interface sketches

The kit is hearth 08's (code-organization.md, 3): hearth 46 loads
`Mirror::index("main")` as its universe; hearth 41 and 24 serve `Proxy`
(client/README.md, 3.5). An enum's doc line names each variant. Derives
(`Clone`, `PartialEq`, `Eq`, `Debug`; `Copy` where it holds) are elided.

```rust
// testing/hearth-test-mirror/src/pin.rs (08.1)
/// `testing/mirror/pin`, read (testing.md, 6.2); a bump is a commit of it.
pub struct Pin {
    /// snapshot.debian.org's canonical timestamp, `YYYYMMDDTHHMMSSZ` (format.md, 5.3's `snapshot.archive`).
    pub archive: Box<str>,
    pub suite: Box<str>,
    pub architecture: Box<str>,
    /// `main` first; M1's is `main` alone.
    pub components: Box<[Box<str>]>,
    /// The mirror digest, 64 lowercase hex digits: at M1, `InRelease`'s SHA-256 (testing.md 6.2).
    pub digest: Box<str>,
    pub targets: Box<[Target]>,
}
/// A target of `testing/repos/test` and its request, sorted, as 07.1's `Simulate::request` takes it.
pub struct Target { pub name: Box<str>, pub variant: Box<str>, pub request: Box<[Box<str>]> }
/// A pin refused: its line from 1 (0 for a field missing), and what is wrong.
pub struct PinError { pub line: u32, pub wrong: Wrong }
/// An unknown field; a field twice; a field missing; a value out of its form, by field.
pub enum Wrong { Unknown, Twice, Missing(&'static str), Form(&'static str) }
impl Pin {
    pub fn read(text: &[u8]) -> Result<Pin, PinError>;
    /// `../mirror/pin` beside this crate; panics naming the file and the error.
    pub fn committed() -> Pin;
    pub fn target(&self, name: &str) -> Option<&Target>;
}

// testing/hearth-test-mirror/src/mirror.rs (08.1)
/// The directory below the cache: `hearth-test-mirror/<digest>/`.
pub const CACHE: &str = "hearth-test-mirror";
/// One pin's fetched mirror, in 08.2's layout.
pub struct Mirror { root: PathBuf, pin: Pin }
/// No cache directory (`XDG_CACHE_HOME` absolute, or `HOME`); the pin's digest not fetched. `Display` names `testing/mirror/fetch`.
pub enum Missing { Home, Unfetched { path: PathBuf } }
impl Mirror {
    pub fn find(pin: &Pin) -> Result<Mirror, Missing>;
    pub fn find_in(cache: &Path, pin: &Pin) -> Result<Mirror, Missing>;
    pub fn pin(&self) -> &Pin;
    /// `checked/<component>/binary-<architecture>/Packages`: decompressed, checked against `InRelease`, never served.
    pub fn index(&self, component: &str) -> PathBuf;
    /// The regular file served at `path`, relative and below `archive/`; none otherwise.
    pub fn file(&self, path: &Path) -> Option<PathBuf>;
}
impl std::fmt::Display for Missing { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result; }

// testing/hearth-test-mirror/src/proxy.rs (08.3)
/// The mirror as an HTTP proxy on `127.0.0.1`, at a port the system chose; one thread, living as long as the process.
pub struct Proxy { port: u16, log: std::sync::mpsc::Receiver<Served> }
/// One answer, logged before its connection closes: the request's target as sent, 200 or 404, the body's length.
pub struct Served { pub target: Box<[u8]>, pub status: u16, pub length: u64 }
/// `GET`, or `HEAD`: the methods answered with a file.
pub enum Method { Get, Head }
/// The longest request head read; past it the answer is 404.
pub const HEAD: usize = 8_192;
/// The one origin served.
pub const ORIGIN: &[u8] = b"http://snapshot.debian.org/";
impl Proxy {
    pub fn serve(mirror: &Mirror) -> std::io::Result<Proxy>;
    pub fn port(&self) -> u16;
    /// `http://127.0.0.1:<port>/`, as 07.1's `Proxy::new` and `Acquire::http::Proxy` take it.
    pub fn url(&self) -> String;
    /// The answers logged since the last call, in order.
    pub fn served(&self) -> Vec<Served>;
}
/// A request head's method and the path it names below `ORIGIN`, in absolute form; none for any other.
pub fn target(head: &[u8]) -> Option<(Method, PathBuf)>;
```

## Questions to raise when reached

None.

## Done when

1. `git -C ~/src/rust/hearth ls-tree --name-only origin/main
   testing/hearth-test-mirror testing/mirror/pin testing/mirror/fetch
   testing/repos/test | wc -l` prints 4.
2. `git -C ~/src/rust/hearth show origin/main:testing/mirror/pin | grep -c
   -E '^(archive [0-9]{8}T[0-9]{6}Z|digest [0-9a-f]{64})$'` prints 2.
3. `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$'
   --format='%(trailers:key=Fetch-Runs,valueonly,separator=)' | awk '{s+=$1}
   END {print s}'` prints 1, 2 or 3.
4. `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-08\.[1-3]'` prints 3.
