# render: later

What the design names that this plan does not do, and when it should be
done.

| Item | Design | When |
|---|---|---|
| Conformance's "resolve, both modes": the same requests simulated chrootless and unshare, for every variant, closures compared (unshare needs the subordinate range) | hearth's testing.md, 5.2; render.md, 10 | M2's plan, with real builds' prerequisites (testing.md, 6.4) |
| `cargo-fuzz` targets for each decoder; this plan fuzzes by seeded sweeps (decisions.md, 10) | hearth's testing.md, 2 (machine worlds: "fuzz targets"); render.md, 9.4 | when skein builds its own (skein's json.md lists `fuzz/` as not built) |
| The closing repins of jig and temper, if temper's client-api is not `done` when this run ends (README.md, Repin) | skein's development.md, 5.7 | once client-api's row reads `done` in jig's and temper's `STATUS.md` |
| The client's in-process builder's configuration beyond M1's built-in defaults (a proxy, larger bounds): from a file beside the store, options, or the environment | hearth's client/README.md, 3.5 and section 5 | M2's plan, when in-process builds fetch |
| Fake git's fetch and its remote | hearth's testing.md, 4.4 | M3's plan, with the steward's clone |
| The fake machine held to the kernel's conformance | hearth's testing.md, 5.4 | M2's plan |
| The fake machine's inode numbers, times and extended attributes, and the rest of `openat2`'s flags | hearth's testing.md, 10 (the M2 row) | M2's plan |
| `Inst` lines with several origins checked against recordings: M1's mirror has no updates or security pockets, so no M1 recording shows one | hearth's render.md, 10; testing.md, 6.2 | the plan that adds pockets to the test mirror (M2's real builds) |
| The fake machine's FIFOs and device nodes | hearth's testing.md, 3.2 | the plan whose code first opens one (M2's builds) |
| The checks part and its `bless` (format.md, 5.13: units that may fail are held "from M3"): M1 decodes `bless` and renders nothing for it (hearth 19, Choices 1) | hearth's format.md, 5.13 | M3's plan |
| The reserved rows of containers and snapshots: a preset or link enabling a container's `systemd-nspawn@` unit; `snapshots` keys but `core` and `data`, and 5.12's bucket counts ("hourly, daily, weekly, monthly, each at least 1") (hearth 21, Choices 8) | hearth's format.md, 5.12 | M5's plan, with containers |
| The identity kinds `wireguard-key` and `backup-key`, which M1's render refuses as `Enum` faults (hearth 19, Choices 6) | hearth's format.md, 9.3 | `backup-key`: M6's plan; `wireguard-key`: the plan that takes in-process keys from skein |
| Transition coverage (llvm-cov of the fake machine's and the worlds' state transitions): no session measures it | hearth's testing.md, 8.4 | M2's plan, when the gate runs llvm-cov |
| apt's and mmdebstrap's versions in a resolution's `made`, from each tool's `--version`: empty in M1, a simulate run printing neither | hearth's render.md, 5.4; store.md, 6.4 | M2's first plan (build) |
