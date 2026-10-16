#!/bin/sh
# The gate (docs/development/workflow.md, section 1): formatting, lints as
# errors, then both test suites in full: the default one, then the fuzzy
# one. skein's `land` runs it through the machine's queue.
set -eu

cd "$(dirname "$0")/.."

# A workspace with no crate has nothing to check, and cargo refuses it.
if cargo metadata --no-deps --format-version 1 | grep -q '"workspace_members":\[\]'; then
    echo "check.sh: the workspace has no crate yet: nothing to check"
    exit 0
fi

# Until the first world's fuzzy test lands, the fuzzy profile selects none(),
# and nextest fails a run that selects no test unless told otherwise. That
# landing gives the profiles their filters (.config/nextest.toml).
fuzzy_none=warn
if ls tests/*/tests/fuzzy_*.rs >/dev/null 2>&1; then
    if grep -qx 'default-filter = "none()"' .config/nextest.toml; then
        echo "check.sh: a fuzzy test exists, but .config/nextest.toml's profiles still lack their filters (workflow.md, section 1)" >&2
        exit 1
    fi
    fuzzy_none=fail
fi

cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
# skein's testing-strategy.md, section 8: a test is a process.
export NEXTEST_DOUBLE_SPAWN=0
cargo nextest run --workspace
cargo nextest run --workspace --profile fuzzy --no-tests="$fuzzy_none"
