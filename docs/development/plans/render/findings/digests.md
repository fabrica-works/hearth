# Digest literals of the render plan, recomputed (DG)

Encoder: store.md 3.1 as at hearth 31bed37, in `~/src/rust/tmp/plan/hearth-render-scratch/encoders/dg/` (`enc.py`; THIN's states in `thin.py`; the rows below come from `lits.py`). Run each command from that directory. The encoder was first checked against literals the answers did not change (H(empty), the 24 keys, the 33 closure digests); all matched.

Each literal is valid at its session's increment; later sessions re-paste under the README Rules. THIN's states: 11.1 meta; 32.1 +config{tree}; 17.2 +packages; 17.4 +boot; 18 config with empty fields +data; 19 packages with seeds, +secrets, software, live; 20 +exports.

Changed: 1 literal. `3972158f...` (24-resolution, test `an_empty_output_part_enters_as_its_record_with_empty_fields`) was the H49 stand-in H(`part:software`, map {}); render.md 4.1 has no such stand-in, so it is now `656d135b92c3...`, H(`part:software`, map { downloads: map {}, scripts: list [] }). Its other former citations (findings/base-B7) stay as the historical record.

Class: content = sha256 of bytes, a fixture or recording, kept; encoder = store.md 3.1 digest, recomputed and equal to the literal. Not recomputable, kept: `33b37c0f` (apt-only closure, depends on 08.2's fetched mirror), `TEST_MIN_KEY` (no input given), `6e80ba38` (MAWK_KEPT, a recording not given whole), `7b36e8a38247` (forgejo fixture, no input), `3f4edef7` (the design's own value, no correct encoder makes it), 10-client's made-up digests and the zero-padded `digest(prefix)` values (fixtures).

| 12-hex (old = new) | Class | Digest of | Cited by | Command |
|---|---|---|---|---|
| `775324fcea7f` | content | THIN's etc/motd (15 bytes) | h18-tree-users, h33-store-entries, h12-compose, h32-static-binary, findings/base-B7, findings/base-B6, findings/digests | `printf 'Welcome to x1.\n' | sha256sum` |
| `f487088ec8b1` | content | THIN's usr/local/bin/motd (29 bytes) | h18-tree-users, h33-store-entries, h12-compose, h32-static-binary, findings/base-B7, findings/base-B6, findings/base-B5, findings/digests | `printf '#!/bin/sh\nexec cat /etc/motd\n' | sha256sum` |
| `21c50adeb849` | content | hostname.j2 rendered | h15-files, findings/digests | `printf 'x1-prod\n' | sha256sum` |
| `e75f92b4b841` | content | issue.j2 rendered (32 bytes) | h15-files, findings/digests | `printf 'x1 runs on metal.\n1: one\n2: two\n' | sha256sum` |
| `5891b5b522d5` | content | greeting.j2 rendered | findings/digests, h15-files | `printf 'hello\n' | sha256sum` |
| `b0b0dd8b10a8` | content | greeting for dev | h18-tree-users, findings/base-B6, findings/digests | `printf 'Hello dev on x1-prod.\n' | sha256sum` |
| `e55838f39d99` | content | greeting for ops | h18-tree-users, findings/base-B6, findings/digests | `printf 'Hello ops on x1-prod.\n' | sha256sum` |
| `4cb3bd448645` | content | lab-ca/lab.crt | h18-tree-users, findings/base-B6, findings/digests | `printf 'lab certificate\n' | sha256sum` |
| `b40d08aa8c5c` | content | customize.sh | h18-tree-users, findings/base-B6, findings/digests | `printf 'update-ca-certificates\n' | sha256sum` |
| `4b1f03482366` | content | customize.sh, changed | h18-tree-users, findings/base-B6, findings/digests | `printf 'update-ca-certificates\nsystemctl daemon-reload\n' | sha256sum` |
| `1632d8753eb0` | content | software.sh | h19-secrets-software, findings/base-B6, findings/digests | `printf 'rg --version\n' | sha256sum` |
| `33165223a0a3` | content | ripgrep's sha256 (a fixture's value) | findings/base-B6, h19-secrets-software, findings/digests | `printf 'ripgrep' | sha256sum` |
| `d2571cab3997` | content | ripgrep 15.1.1's sha256 (a fixture's value) | h19-secrets-software, findings/base-B6, findings/digests | `printf 'ripgrep 15.1.1\n' | sha256sum` |
| `474a6f5bdf9c` | content | /etc/subuid and /etc/subgid | h22-collisions, findings/base-B6, findings/digests | `printf 'ci:100000:65536\nfree:165536:65536\n' | sha256sum` |
| `6cb7c9647dfa` | content | tiny-initramfs.conf (70 bytes) | findings/base-B6, findings/digests, h22-collisions | `printf 'DEBUG=no\nAUTO_MODULES=no\nMICROCODE=generic\nINCLUDE_MODULES=btrfs,nvme\n' | sha256sum` |
| `ca2dff161724` | content | the hearth task's binary | h21-reserved, findings/digests | `printf 'hearth 0.1.0\n' | sha256sum` |
| `afd16377136a` | content | vpn peers, lab | h20-exports, findings/digests, findings/base-B6 | `printf 'ion-lab=10.1.0.1;\n' | sha256sum` |
| `0ed834effddb` | content | vpn peers, prod | h20-exports, findings/base-B6, findings/digests | `printf 'ion-prod=10.0.0.1@194.164.162.157;\n' | sha256sum` |
| `01ba4719c80b` | content | /etc/containers, none | findings/base-B6, findings/digests, h20-exports | `printf '\n' | sha256sum` |
| `9e40aaeae922` | content | /etc/containers, lab | h20-exports, findings/base-B6, findings/digests | `printf 'mail-lab=container@rhi-lab;\n' | sha256sum` |
| `755c9769e418` | content | /etc/containers, prod | findings/digests, findings/base-B6, h20-exports | `printf 'chat-prod=container@rhi-prod;mail-prod=container@rhi-prod;\n' | sha256sum` |
| `f745f2a642a2` | content | /etc/debian | findings/base-B6, findings/digests, h17-packages | `printf 'forky amd64 20261001T030405Z contrib,main,non-free-firmware\n' | sha256sum` |
| `ad093de033f8` | content | 08's PIN digest (signed LF) | h08-mirror, findings/digests | `printf 'signed\n' | sha256sum` |
| `b44f7d6b5283` | content | 08's XZ_SHA256 (xz LF) | h08-mirror, findings/digests | `printf 'xz\n' | sha256sum` |
| `2c26b46b68ff` | content | 38's SHA256 field (foo) | h38-stanzas-recordings, findings/digests | `printf 'foo' | sha256sum` |
| `25dfd29c0961` | content | skein 03, a u16 as its two bytes | skein/03-state-digest, findings/base-B1, findings/digests | `printf '\x02\x01' | sha256sum` |
| `a8cdd76642f0` | content | 44's BEFORE | h44-fake-git, findings/digests | `printf 'a\nb\nc\nd\ne\nf\ng\nh\n' | sha256sum` |
| `890cc14fc1b2` | content | 44's AFTER | findings/digests, h44-fake-git | `printf 'a\nb\nc\nd\nE\nf\ng\nh\n' | sha256sum` |
| `a603a55b1648` | content | 28's MOTD_KIND_SHA256 | findings/digests | `printf 'Welcome, and be kind.\n' | sha256sum` |
| `e3b0c44298fc` | content | empty file (29) | h29-goldens, findings/digests | `printf '' | sha256sum` |
| `ba7816bf8f01` | content | skein 03, abc | findings/digests | `printf 'abc' | sha256sum` |
| `8f43cea6a4bf` | encoder | H(empty) (store.md 3.2) | h01-lib, findings/digests | `python3 -c 'from enc import *; print(H("empty"))'` |
| `22d5156ffefa` | encoder | packages key of digests 00..00, 01..01 (store.md 3.2) | h24-resolution, findings/digests | `python3 lits.py` |
| `f96c81da124d` | encoder | software key over that and 02..02 | h24-resolution, findings/digests | `python3 lits.py` |
| `a56d06bfeaff` | encoder | config key over that and 03..03 | findings/digests, h24-resolution | `python3 lits.py` |
| `01ea811afc99` | encoder | resolution key of 33.2's inputs() | findings/digests, findings/base-B7, h33-store-entries | `python3 lits.py` |
| `a33fbba1e283` | encoder | resolution key of variant minbase (K2) | h33-store-entries, findings/digests | `python3 lits.py` |
| `f2fbea8ecfaf` | encoder | closure digest, apt amd64 3.0.3 01..01 | h33-store-entries, findings/digests | `python3 lits.py` |
| `c1589a971c4c` | encoder | closure digest, empty closure | h33-store-entries, findings/digests | `python3 lits.py` |
| `706777e0f856` | encoder | THIN's meta part | h11-thin-story, findings/digests | `python3 thin.py` |
| `e2475ba7ef36` | encoder | THIN's config32 part | h32-static-binary, findings/base-B4, findings/base-B6, findings/digests | `python3 thin.py` |
| `369415fd653c` | encoder | THIN's config18 part | h18-tree-users, findings/digests | `python3 thin.py` |
| `de38a7ed6276` | encoder | THIN's packages17 part | h17-packages, findings/base-B6, findings/digests | `python3 thin.py` |
| `64b1b99b813a` | encoder | THIN's packages19 part | h19-secrets-software, h17-packages, findings/digests | `python3 thin.py` |
| `bc6cab417252` | encoder | THIN's boot part | h17-packages, findings/base-B6, findings/digests | `python3 thin.py` |
| `2d4765200bab` | encoder | THIN's data part | findings/digests, h18-tree-users | `python3 thin.py` |
| `ac870b5c968e` | encoder | THIN's secrets part | h19-secrets-software, findings/digests | `python3 thin.py` |
| `656d135b92c3` | encoder | THIN's software part | h19-secrets-software, h24-resolution, findings/digests | `python3 thin.py` |
| `6f1b7fda0c21` | encoder | THIN's live part | h19-secrets-software, findings/digests | `python3 thin.py` |
| `970579fd41be` | encoder | THIN's exports part | findings/digests | `python3 thin.py` |
| `d5942fa1a047` | encoder | THIN's spec, 11.1 meta | h11-thin-story, h17-packages, findings/base-B4, findings/digests, findings/base-B6 | `python3 thin.py` |
| `9970fb4d8f97` | encoder | THIN's spec, 32.1 +config | h32-static-binary, h17-packages, findings/base-B4, findings/base-B6, findings/digests | `python3 thin.py` |
| `cf667f7c3bf8` | encoder | THIN's spec, 17.2 +packages | h17-packages, findings/base-B6, findings/digests | `python3 thin.py` |
| `dd576929cc4d` | encoder | THIN's spec, 17.4 +boot | findings/base-B6, findings/digests, h17-packages | `python3 thin.py` |
| `af576004e1f8` | encoder | THIN's spec, 18 +config fields,+data | h18-tree-users, findings/digests | `python3 thin.py` |
| `405bf3bde392` | encoder | THIN's spec, 19 +seeds,+secrets,software,live | h19-secrets-software, findings/digests | `python3 thin.py` |
| `23656fdf46b7` | encoder | THIN's spec, 20 +exports | findings/digests | `python3 thin.py` |
| `3972158f...` -> `656d135b92c3` | encoder, changed | the empty software part | h24-resolution, h19-secrets-software | `python3 thin.py` |
