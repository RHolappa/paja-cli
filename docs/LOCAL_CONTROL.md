# Local control walkthrough

This example exercises the complete development loop without a model, credentials, or external tools. Its deterministic adapter is labelled `control_fixture` in every result. It is not evidence of AI quality or natural-language understanding.

## Build and initialize

From the repository root, with [rustup](https://rustup.rs/) installed:

```sh
cargo build --release --locked
export PATH="$PWD/target/release:$PATH"
paja init meal-helper --goal "Help people choose lunches within their budget"
cd meal-helper
paja build
```

The repository pins [Rust 1.99.1](https://blog.rust-lang.org/releases/1.99.1) and [edition 2024](https://doc.rust-lang.org/edition-guide/rust-2024/index.html). Building initially needs network access to download the toolchain and dependencies; after fetching, the checks also work with Cargo's `--offline` option. No provider spending is involved.

Init creates an editable `paja.json`, a two-case `cases.json`, and private `.paja` state. The name must be a single directory name; existing directories are never overwritten. The goal and instructions are metadata in this adapter. A real model/build adapter will later use them to develop a target agent.

## Observe a failure

```sh
paja run --input "Choose two lunches for a total of eight euros"
paja test
```

The starter's `catalog_order` strategy selects soup (400 cents) and wrap (700 cents). The host independently computes 1100 cents, so the 800-cent assertion fails. The purchase-boundary case passes. Each case runs twice by default, producing two passing and two failing trials. Repetition here tests the mechanics; deterministic repeats measure no model variation.

Results record the complete frozen suite, input, output, tool observations, each assertion, elapsed time, package digest, and usage. There are zero model calls and zero provider cost. A tool denial or exhausted execution budget is distinguishable from an assertion failure.

## Improve and compare

```sh
paja improve
paja results
paja run --input "Choose two lunches for a total of eight euros"
```

The supported fixture repair changes `runtime.strategy` to `cheapest`. It chooses two soups for 800 cents. The exact baseline suite is rerun; four trials pass, and the comparison identifies `within_budget` as improved. The previous version and its failed test run remain queryable.

The candidate becomes current only when a comparable run improves at least one case, regresses none, and has no execution failures. A rejected candidate remains stored under its own revision and improvement run. Inspect the run ID printed by improve with `paja results --run <id>`. A separately added variety requirement demonstrates a real regression in the integration tests: selecting the cheapest item twice loses distinct-item variety, so that candidate is retained without replacing the baseline.

Improve never edits `paja.json` or `cases.json`. To continue editing an adopted fixture strategy, apply the shown runtime change to `paja.json` before the next build. Building unchanged source intentionally creates the original strategy again. Runtime history and editable source are distinct.

## Try retained versions and new tests

```sh
paja run --revision 1 --input '{"count":2,"budget_cents":800,"purchase":false}'
paja run --revision 2 --input '{"count":2,"budget_cents":800,"purchase":false}'
paja test --revision 1
paja status
```

Revision 1 still returns the original 1100-cent result. Historical tests attach to revision 1 and do not replace current-revision evidence. `status` reports only the current candidate's latest test.

Default tests use the suite frozen at build time. Edit the working suite separately, increment its version, and use `paja test --suite cases.json` to explicitly select it. Comparing that run with the old suite is non-comparable. An improvement started from an older saved run still uses that run's frozen suite, fixtures, and assertions.

Machine callers use the explicit project ID, revision, and request key described in [CLI](CLI.md). A completed retry returns the original response and trial identities even after the current candidate advances. The request must retain its original arguments and normalized input contents.

## Limits and recovery

- One local filesystem owner is supported. Worker credentials, shared worker checkouts, remote access, and direct worker database access are not supported.
- SQLite transactions persist versions, run evidence, and completed retry receipts atomically. Standard-library file locks fence other CLI processes; a busy response is retryable. Do not remove or replace the lock file while operations are running.
- The fixed fixture cannot run arbitrary target code or access the network. Only catalog reads and calculation can be granted. There is no purchasing integration.
- Input files are limited to 1 MiB. Suites allow 1–64 cases and 1–128 catalog items; trial, tool-call, and elapsed-time budgets are enforced. No trial fan-out or unlimited improvement loop runs.
- Runs are short synchronous operations. There is no durable background scheduler, disconnect-resumption protocol, `stop` command, or paid-effect reconciliation yet. Uncommitted database work rolls back on connection loss; a committed operation can be replayed using its saved request key. Partial initialization after a crash requires inspection and a new destination; init never overwrites it.
- Private traces and suites stay in `.paja`. No export or publication occurs. Unix directory permissions protect against other accounts under normal filesystem permissions; they do not authenticate multiple workers or defend against the filesystem owner editing the database.

## Performance and checks

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
python3 scripts/check_repository.py
python3 scripts/benchmark_cli.py --binary target/release/paja --iterations 30
```

Run these from the repository root. The benchmark creates and removes synthetic temporary projects, checks the failure/repair outcomes, and reports process-level timings including persistence. See [Validation](VALIDATION.md) for the actual local result and remaining acceptance work.
