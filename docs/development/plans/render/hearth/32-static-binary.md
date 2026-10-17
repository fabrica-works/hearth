# Session 32: static files into the cache, and the binary

render.md 4.2, 3.1 and client/README.md 3.1, 3.5. 32.1: the `config` part
of static files, each blob streamed into the cache by SHA-256 (store.md,
6.8). 32.2: the binary, `hearth render`'s arguments, startup and exit code.
Needs hearth 11 (11.3: the real loop); 32.2 also hearth 40.2
(`records::Startup`). It grows the thin story that hearth 11 meets: its
Choices, Rules and "Where later sessions add" bind here as there.

## Read first

- `render.md`: 2.2, 2.3, 3.1, 3.2, 4.1, 4.2 (`config`), 4.6, 9.2;
  `format.md`: 3.3, 5.1 (first sentence), 5.5 (first bullet);
  `client/README.md`: 3.1 to 3.3, 3.5; `store.md`: 2.1, 2.2, 6.8;
  `testing.md`: 2.7.
- Drafts: `hearth/11-thin-story.md` whole (Choices, "Where later sessions
  add", 11.2, 11.3, sketches); `40-client-diff-errors.md` (`records::Startup`, 40.2).
- skein: `programming-model.md`: 2.1; `skein-io`'s `FileIo`
  (`findings/skein-io.md`); smith's `crates/smith-agent-service/src/
  service.rs:968` (completions tagged for io or files); temper's
  `crates/temper/tests/support/mod.rs` (`serve` :85, `StdoutReader` :119,
  `until_link_then` :338).

## What the design asks

- **A static file "is never decoded and never held"** (render.md, 3.1):
  the protocol layer streams its blob into the cache by SHA-256; "written
  to `cache/<random>.tmp` (2.1), hashed as it is written, synced, and
  renamed to `cache/<sha256>`"; a name already present "is not written
  again" (store.md, 6.8); the tree entry's "mode is 0755 if git records
  it executable, and 0644 otherwise ... Owner and group are root"
  (format.md, 5.5); "Cache files are 0644 in a 0700 directory" (2.2).
- **The binary** (client/README.md, 3.1, 3.5): `--at` "defaults to
  `HEAD`"; `--store` "by default `$XDG_CACHE_HOME/hearth/store`
  (`~/.cache/hearth/store` when it is unset), made 0700 on first use";
  "Its `main` is shell code ... returns 3.3's code"; what keeps it from
  answering "goes to standard error, as a record with `--json`" (3.2).

## Choices this plan makes

- **`task.json` is `{}`** (the planner's ruling): 32.1 reads each present
  task's `task.json` as `As::Declaration` through 03.1's `json::Decoder`
  alone, so a malformed one fails on this path, and render gets
  `Decoded::Declaration`, no type; hearth 13 and 16 add the declarations.
  No hearth 04 increment is needed.
- **The shell owns the process** (client/README.md, 3.5): the umask
  (32.1: one `libc` call), the generator's seed (`skein_shell::seed`) and
  what keeps the client from answering (32.2) are `crates/hearth`'s
  (programming-model.md, 2.1).
- **`PartRecord`** is 11.1's, with no variant; 32.1 adds `Config`.
- **Two-word fields are snake case** (`content_digest`; render.md 4.6); a
  repository or store the client cannot open is 75 (client/README.md 3.3).

## Increments

### 32.1 A `config` part of static files, streamed into the cache
Scope: presence (format.md, 3.3: the base's and the base-env's task
keys, `null` removing), the base document joining the first `Read`; the
classes `Declaration` and `Tree`; a second `Read` of each present
task's `task.json` (`As::Declaration`) and its `tree/` blobs
(`As::Cached`), templates (`.j2`) and links left out until hearth 15 and
18, a submodule failing its target (render.md, 2.2); each
cached blob streamed by store.md 6.8 (`cache/<64 random hex>.tmp`, written
as its pieces pass `skein_lib::sha256::Sha256`, synced; if
`cache/<sha256>` opens, the temporary removed, else renamed and `cache/`
synced); `Cached`; the `config` part (4.2's `tree`, files only); the
service's `FileIo`; `translate/parts.rs`, `parts/config.rs`.
Needs: nothing beyond hearth 11 (11.3); `skein_lib::sha256` is in hearth's pin.
Files: root `Cargo.toml`, `Cargo.lock` (shared; `libc = "0.2.186"`, skein's
version, in `[workspace.dependencies]`); render `src/{vocabulary, revision, compose, call, parts}.rs`
(shared), new `src/parts/config.rs`, `src/tests/{revision, parts}.rs`,
`tests/render/{src/fixtures.rs, tests/thin.rs}`; protocol `src/{lib,
read}.rs`, new `src/cache.rs`, `src/translate/{documents, answers}.rs`,
new `src/translate/parts.rs`, `src/translate/parts/config.rs`,
`src/tests/{cache, translate}.rs`; the service's `src/{lib, limits}.rs`
(completions tagged for io or files, as smith's `crates/smith-agent-
service/src/service.rs:968` does); `crates/hearth/{Cargo.toml (libc),
src/lib.rs}` (the umask; `cache/` made 0700 and opened at startup);
`tests/builder/{Cargo.toml (libc), src/fixtures.rs, tests/real.rs}`.
Shape: the sketches. A tree entry's path is `/` and what follows
`tasks/<t>/tree/`; mode 0o644, or 0o755 for `Mode::Executable`; owner and
group `root`; the tree in its paths' byte order. A static file past
2.3's 64 MiB (`static_file` in render's limits) fails its target before
it is read (12.2 tests it, with its `TooLarge` at the path). The
temporary's 64 hex come from four `skein_lib::Rng` draws, seeded by the
shell (32.2). `Client::open` first sets the umask to 022, so that a
cache file is 0644 and a directory 0700 whatever umask the client was
started with (store.md, 2.2); skein's `Create` takes the umask off. Skein
has no safe `umask`: one `#[expect(unsafe_code, reason = …)]` on
`crates/hearth/src/lib.rs`'s `pub fn umask(mask: u16)` (`libc::umask`, the
old mask dropped), with its `// SAFETY:` comment (README, Rules); `Client::open`
and the tests call it. `crates/hearth`'s `[lints]` and `clippy.toml` are
skein-shell's (`unsafe_code = "deny"`, `allow_attributes = "deny"`: an
`allow` fails). The rename is `file::Request::Rename { owner, from_root,
to_root, from, to }`, both roots `cache.fd`, if `git -C ~/src/rust/skein
merge-base --is-ancestor <04.1's landing commit> <the commit Cargo.lock
names for skein>` succeeds, else `{ owner, root: cache.fd, from, to }`, which
23.1's repin edits to the first.
Test shape: unit: `an_executable_blob_is_0755_and_any_other_file_0644`,
`a_task_removed_by_null_in_the_base_env_gives_no_files`,
`the_config_digest_is_h_of_part_config_over_its_tree`
(`e2475ba7ef36cd9f9b7b38622b1f67c97f173151988357e2ba4e7226661c1fd2`),
`a_cached_blob_is_written_to_a_temporary_synced_then_renamed`,
`a_cache_file_already_present_keeps_it_and_removes_the_temporary`; world:
`a_malformed_task_json_fails_the_targets_that_have_the_task`,
`a_submodule_under_a_present_tasks_tree_fails_its_target`,
`a_template_and_a_link_are_left_out_of_the_tree`; real:
`hearth_render_prints_…` of 11.3, its literal grown to `fixtures::THIN_TEXT`,
```text
target       x1-prod   base x1   environment prod
placement    metal efi 7th-gen
spec         9970fb4d8f97
config       part e2475ba7ef36
  /etc/motd                file 0644 root:root 775324fcea7f
  /usr/local/bin/motd      file 0755 root:root f487088ec8b1
```
and `static_files_enter_the_cache_by_sha256_and_no_temporary_stays`
(run under umask 077, which the test sets first by `hearth::umask`: `<store>/cache`, mode
0700, lists exactly
`775324fcea7f2cdcbf11df25550e362669caceb6e69c8370374d2d436bb63123` and
`f487088ec8b18ff569dfede6f12e5fa91983f9f6dd2f3e0b6cee4349a231e745`, each
holding its file's bytes, mode 0644).
Proof: process, `hearth_render_prints_the_thin_repositorys_spec_through_real_git`:
the two files reach the spec by their SHA-256, streamed through real
git's `cat-file`. Run once: `heavy cargo nextest run -p hearth-builder-world
-E 'test(=hearth_render_prints_the_thin_repositorys_spec_through_real_git)'`.
Bound 0.5 s.
Don't: hold a static file's bytes beyond one piece; write its record or
take a lock (hearth 23); hash a copy; read a template or a link's target.
Size: at most 285 lines. Removes: 11.3's empty `records::Spec::parts`.

### 32.2 The binary: arguments, startup, the exit code
Scope: `hearth render [<target>…] [--at <rev>] [--repo <path>] [--store
<path>] [--json]` (client/README.md, 3.1, 3.2, 3.5): `--at` `HEAD` by
default; `--repo <path>` alone, to its git directory, absolute
(`<path>/.git`, a directory or a gitfile, else `<path>`; by default the
nearest directory upward holding `.git`: render.md 2.1, client/README.md
3.5); the store's default from `XDG_CACHE_HOME` then `HOME` (3.5); the
seed; what keeps the client from answering (anything else, `diff` and
`--resolve` until hearth 45 and 24, a store not made or opened) as 40.2's
`startup` record on standard error, 64 or 75 (3.2, 3.3); the loop;
`ExitCode::from` the code; `end_to_end_render` (testing.md, 2.7).
Needs: hearth 40.2 (`records::Startup`).
Files: `crates/hearth/{Cargo.toml, src/lib.rs}`, new `src/main.rs`,
`src/argv.rs`, `src/tests.rs`, `tests/{end_to_end, refusals}.rs`
(dev-dependency `hearth-builder-world`, code-organization.md, 4);
`tests/builder/src/fixtures.rs` (`binary(program, arguments, environment,
directory) -> Ended { exit, stdout, stderr }`, after temper's
`crates/temper/tests/support/mod.rs` (`serve` :85, `StdoutReader` :119,
`until_link_then` :338): `Binary::start` in `Mode::Pipes` and a reader
host on `skein_world::real::World`).
Shape: the sketches; `main` reads `std::env::args_os`, `var_os` and the
working directory and hands them to `argv::form` and `argv::invocation`,
which its tests call; `form` finds `--json` anywhere in argv, so that a
refusal before it is JSON too. A variable empty or relative counts as
unset; with neither, `Unopened { Store }`, 75; the store's missing
parents are made 0700 (3.5). The seed is eight
bytes of `skein_shell::seed` (no `getrandom` of its own; an `Err` is
`Unopened { Store }`, 75), so that a killed run's `.tmp` name is not drawn
again before hearth 23's scan (store.md, 2.1). git runs with
`HOME=/dev/null` (render.md 2.2): no home is made.
Test shape: unit (`src/tests.rs`, under 20 ms):
`render_s_options_become_the_call_and_its_form`,
`an_unknown_command_or_option_is_usage`,
`json_anywhere_in_argv_gives_the_refusal_its_form` (`--json --bogus`,
`--bogus --json`),
`a_malformed_revision_is_usage_before_the_loop` (`--at -x`),
`the_store_is_xdg_cache_home_s_then_home_s_cache`,
`an_empty_or_relative_variable_counts_as_unset`,
`with_neither_variable_the_store_is_unopened`,
`a_work_tree_s_repository_is_its_dot_git`,
`two_seeds_drawn_at_startup_differ`. End to end, under 0.5 s (`end_to_end_`
names only this one):
```rust
// crates/hearth/tests/end_to_end.rs
#[test]
fn end_to_end_render() {
    let repository = Repository::make("end-to-end", fixtures::THIN);
    fs::write(repository.work_tree().join("tasks/motd/tree/etc/motd"), "not committed\n").expect("an edit");
    let xdg = repository.scratch().join("xdg");
    let environment = [("XDG_CACHE_HOME", xdg.as_os_str()), ("HOME", repository.scratch().as_os_str())];
    let ended = fixtures::binary(env!("CARGO_BIN_EXE_hearth").as_ref(), &["render".as_ref()], &environment, repository.work_tree());
    assert_eq!(String::from_utf8_lossy(&ended.stdout), fixtures::THIN_TEXT, "3.1: what is not committed is not rendered");
    assert_eq!(ended.exit, Some(Exit::Code(0)), "stderr: {}", String::from_utf8_lossy(&ended.stderr));
    for made in ["hearth", "hearth/store", "hearth/store/cache"] {
        assert_eq!(fs::metadata(xdg.join(made)).expect("made").permissions().mode() & 0o7777, 0o700, "3.5: {made}");
    }
}
```
Then, in `crates/hearth/tests/refusals.rs`:
`an_unknown_option_is_refused_on_standard_error_in_the_call_s_form`
(`render --json --bogus` and `render --bogus --json`: 64, standard output
empty, standard error 40.2's two JSON lines);
`a_store_that_cannot_be_made_is_refused_with_75` (`--store` beneath a
file: its `hearth: cannot open the store …` line).
Proof: process, `end_to_end_render`: the shipped binary, run in the work
tree with no option, finds its repository and store by default and prints
the thin repository's committed spec. Run once: `heavy cargo nextest run
-p hearth -E 'test(=end_to_end_render)'`. Bound 0.5 s.
Don't: read any variable but `XDG_CACHE_HOME` and `HOME` (3.5); try a
socket; print in the shell but a `startup` record, by `form::print`.
Size: at most 230 lines. Removes: nothing.

## Interface sketches

Types derive as hearth 11's do. Each block adds to the type 11 sketches.

```rust
// crates/hearth-domain-render/src/vocabulary.rs (32.1): 11.1's enums gain these variants
pub enum Event { /* 11.1's, and: */ Cached { request: Token, oid: Oid, size: u64, sha256: Digest } } // a blob in the cache: the last ends the Read
pub enum As { /* Document, and: */ Declaration, Cached }
pub enum Decoded { /* 11.1's, and: */ Declaration } // a `task.json` that is one strict JSON object (hearth 13, 16: its declarations)
// crates/hearth-domain-render/src/revision.rs (32.1)
pub enum Class<'a> { /* 11.1's, and: */
    Declaration { task: &'a [u8] },           // tasks/<task>/task.json
    Tree { task: &'a [u8], path: &'a [u8] },  // tasks/<task>/tree/<path>: a template if it ends in .j2
}
pub struct Layer { pub placement: Option<Placement>, pub tasks: Box<[Task]> }
pub struct Task { pub name: Box<[u8]>, pub at: Position, pub removed: bool } // a task's key is present or removed by `null`
// crates/hearth-domain-render/src/parts.rs (32.1): 11.1's `PartRecord` gains `Config(Config)`
// parts/config.rs (32.1): render.md 4.2's `tree`, in its paths' byte order; hearth 18 the rest, `Link`, `Directory`
pub struct Config { pub tree: Box<[TreeEntry]> }
pub struct TreeEntry { pub path: Box<[u8]>, pub node: Node }
pub enum Node { File { content_digest: Digest, mode: u16, owner: Box<[u8]>, group: Box<[u8]> } }

// crates/hearth-protocol-builder/src/lib.rs (32.1)
pub enum Asked { Io(io::Request), File(file::Request) }  pub enum Told { Io(io::Event), File(file::Event) }
// `Protocol::new` gains `cache: Fd` (the cache's root) and `seed: u64` (the temporaries' generator's), after `root`; a cache file is a held write
pub fn part(part: render::parts::Part) -> records::Part; // translate/parts.rs (32.1)

// crates/hearth-service-builder/src/lib.rs (32.1)
pub struct Files { pub files: u32, pub read: u32, pub entries: u32, pub timeout: Duration } // FileIo's sizes
// `Limits` gains `files: Files` after `io`; `Roots` gains `cache: Fd`; `Service::new` gains `seed: u64` last
pub struct Roots { pub store: Fd, pub cache: Fd }

// crates/hearth/src/lib.rs (32.1), src/argv.rs (32.2): ordinary Rust; `Startup` is records' (40.2)
// `Client::open` gains `seed: u64` last and, first, sets the umask to 022 and opens `cache/`; it makes the store and its cache, with their missing parents, 0700
/// `libc::umask` behind the one `#[expect(unsafe_code)]` (32.1).
pub fn umask(mask: u16);
/// JSON lines if `--json` is anywhere in `args`, else text: a refusal's form too.
#[must_use] pub fn form(args: &[OsString]) -> Form;
/// argv and the two variables to an invocation, or what keeps the client from answering.
pub fn invocation(args: &[OsString], xdg_cache_home: Option<&OsStr>, home: Option<&OsStr>, cwd: &Path) -> Result<Invocation, Startup>;
```

## Questions to raise when reached

None.

## Done when

1. `git -C ~/src/rust/hearth log origin/main --grep='^Plan: render$' --format='%(trailers:key=Step,valueonly)' | sort -u | grep -c -x -E 'hearth-32\.[1-2]'` prints 2.
2. Once, in a worktree of hearth at `origin/main`: `heavy cargo nextest
   run -p hearth-render-world -p hearth-builder-world -p hearth` passes,
   its `Summary` naming at least 29 tests, and `heavy cargo nextest run
   -p hearth -E 'test(/^end_to_end_/)'` prints a `Summary` naming 1 test.
3. `git -C ~/src/rust/hearth ls-tree --name-only origin/main
   crates/hearth-protocol-builder/src/translate/` lists `parts` and
   `parts.rs`.
4. `git -C ~/src/rust/hearth grep -c 'var_os' origin/main -- crates/hearth/src`
   prints one line, `origin/main:crates/hearth/src/main.rs:2`.
