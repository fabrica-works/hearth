# What of os the plan uses, and where it is given

There is no os checkout on this machine (`~/src/rust/` holds skein, smith,
jig, hearth and temper), so os's source (`os-mmdebstrap/src/lib.rs`,
`os-proc/src/lib.rs`, `os-manifest/src/resolve.rs`) cannot be read here.
What the plan follows of it is what hearth's design says of it, below; the
first session to run where os exists compares (07's `[Q?] 07.3, os's
lists`; answered (H61): build.md 10.1 and 5.5 are the contract, below).

## The failure tail (hearth build.md, 10.1; render.md, 5.5)

- Kept (allow, in this order): `dpkg: error`, `E: `, `Err:`,
  `No space left on device`, `Hash Sum mismatch`, `Failed to fetch`,
  `Unable to `, `Could not `.
- Never kept (deny, tested first): `I: PID n exited with exit code`,
  `E: mmdebstrap failed to run`.
- Each kept line is cut at 160 bytes; the last 20 are kept; the bound on a
  line read is 4096 bytes (build.md, 10.1).
- Not given by the design, and chosen by 07.3: the continuation rule
  (indented lines after a kept `dpkg: error` or `E: ` line), tab and CR
  as spaces, stderr's tail before stdout's.

## What hearth 38 takes (38.1)

The design (build.md, 5.4 and 10.1; store.md, 6.1) gives the name forms
and the `Packages` and list-file rules 38 uses; the two os functions that
38's lines named (`os-manifest/src/resolve.rs:80` and `:397`) are not
given anywhere on this machine, and 38 cites the design instead.
