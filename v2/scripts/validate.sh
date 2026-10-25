#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
# Runtime and tests use no network. Populate the Cargo cache once with cargo fetch
# and python3 scripts/beta-harness.py fetch --locked if needed on a new machine.
cargo fmt --check
rustfmt --edition 2021 --check adapter/oklch.rs
cargo test --locked --offline
cargo clippy --locked --offline --all-targets -- -D warnings
python3 scripts/beta-harness.py test --locked --offline
python3 scripts/beta-harness.py clippy --locked --offline --all-targets -- -D warnings
mkdir -p validation-output
python3 scripts/beta-harness.py run --locked --offline --quiet --example view > validation-output/view.json
node scripts/oracle.cjs
node scripts/renderer-contract.cjs
node scripts/schema.cjs
python3 scripts/provenance.py
