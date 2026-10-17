# Findings: the suites at the plan's base

Measured 2026-10-16 on x1 (Debian sid; rustc 1.95.0, cargo-nextest
0.9.137, git 2.53.0, mmdebstrap 1.5.7), idle, through `heavy --gate`.

## skein at 7e8fb70

One run of the gate in README.md's Rules (check.sh's commands, the browser
run filtered), in a detached worktree at `7e8fb70`, with
`SKEIN_TEST_CHROMIUM` set to Chromium 147: exit 0, 188 s in all
(`~/src/rust/tmp/logs/20261016T123753-0.log`, outside the plan).

| Suite | `Summary` |
|---|---|
| focused | 6.459 s, 2452 tests run: 2452 passed |
| fuzzy | 41.843 s, 99 tests run: 99 passed (5 slow) |
| browser | 23.273 s, 37 tests run: 37 passed, 1 skipped |

One run, not the median of three: the aims in README.md (Rules, "Test
time") rest on it.

## hearth at b3438da

No crate: `scripts/check.sh` prints that there is nothing to check and
passes. Both suites start at 0 s.

## skein at f981746, on debian-8gb-hel1-1 (2026-10-17)

The run's machine (4 cores, 7 GB; Debian 13; rustc 1.95.0, cargo-nextest
0.9.137, git 2.47.3, mmdebstrap 1.5.7, Chromium 147), through `heavy --gate`
from a detached worktree at `f981746` (`~/src/rust/tmp/plan/hearth-render-skein`),
`SKEIN_TEST_CHROMIUM` set: README.md's gate command (Rules, "skein's gate")
without `land --gate`, as `heavy --gate -- bash gate.sh`: exit 0, 235 s
(`~/src/rust/tmp/logs/20261017T125112-0.log`, outside the plan).

| Suite | `Summary` |
|---|---|
| focused | 6.270 s, 2476 tests run: 2476 passed, 0 skipped |
| fuzzy | 30.036 s, 103 tests run: 103 passed, 0 skipped |
| browser | 22.037 s, 37 tests run: 37 passed, 1 skipped (the filtered test) |

## mmdebstrap's simulate, on debian-8gb-hel1-1 (2026-10-17)

hearth 07.1's vector (`the_vector_is_render_s_for_a_forky_main_request`),
run by hand with 5.2's environment, against snapshot.debian.org:

```sh
printf 'deb http://snapshot.debian.org/archive/debian/20261001T030405Z/ forky main\n' > sources.list
env -i LC_ALL=C PATH=/usr/sbin:/usr/bin:/sbin:/bin TMPDIR="$PWD" /usr/bin/mmdebstrap --verbose --simulate --mode=chrootless --variant=apt --architectures=amd64 --include=apt,mmdebstrap '--aptopt=Acquire::Check-Valid-Until "false"' '--aptopt=APT::Install-Recommends "false"' --skip=cleanup/apt/lists forky "$PWD/root" "$PWD/sources.list"
```

Two runs through `heavy --gate`, each from an empty `root`: exit 0, 5 s of
run each (7 s and 6 s with heavy's queueing), 111 `Inst` lines, the first
`Inst gcc-16-base (16.2.0-3 Debian:testing [amd64])`. Against the
conformance profile's 2-minute ceiling (testing.md, 9.2), a real run costs
about 5 s, the lists fetched from snapshot.debian.org each time: hearth
41.1's seven cases and its `several_includes_resolve_as_one` run make eight
runs, about 40 s against 41.1's aim of 60 s; hearth 24's live runs add
about 5 s each.

## MiniJinja's source (2026-10-17)

`heavy cargo fetch` of a scratch crate
(`~/src/rust/tmp/plan/hearth-render-scratch/fetch-minijinja`, `minijinja = {
version = "=2.20.0", features = ["json"] }`): exit 0;
`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/` now holds
`minijinja-2.20.0`, `memo-map-0.3.4` and `serde_json-1.0.154` (hearth 06,
30 and 36 cite `~/.cargo/registry/src/*/minijinja-2.20.0/`).
