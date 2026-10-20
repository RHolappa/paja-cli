# Paja CLI

**Build, test, and improve your AI agent.**

**Rakenna, testaa ja kehitä tekoälyagenttiasi.**

An open-source project of **Innostium Oy**, implemented in Rust.

The intended experience is to describe a problem and have Paja generate an agent you can run, inspect, test, and improve. Paja manages the project, versions, execution limits, and evidence. See [problem-to-agent generation](docs/GENERATION.md) for the design and the distinction between development workers and agents collaborating on the actual task.

**Status: local control prototype.** The core commands run using a deterministic meal-helper fixture. They prove versioning, test evidence, bounded changes, and comparisons. They do not build or improve a real AI agent yet. Real-model adapters and authenticated worker collaboration are the next implementation stages.

## Try it in your browser

The browser demo is at **[paja.innostium.com](https://paja.innostium.com)**. Ask a maintainer for an access key. Record a short English or Finnish prompt, review the transcript, and create a project. Then try **Build → Test → Improve → Results**, or use the lunch example and **Run prompt**.

The demo uses the same deterministic control fixture described below. Voice recognition is real; generating an arbitrary agent from the spoken brief is not implemented yet. Use synthetic prompts for testing.

For your own installation, see the generic [Mac and browser setup](docs/VOICE.md). Details of the hosted demo's infrastructure and access credentials are kept out of this public repository.

## Try the complete loop

Install Rust with [rustup](https://rustup.rs/). The repository pins stable Rust 1.99.1 with the 2024 edition. Cargo downloads dependencies on the first build; the example itself needs no network, credentials, or provider spending.

```sh
cargo build --release --locked
export PATH="$PWD/target/release:$PATH"
paja init meal-helper --goal "Help people choose lunches within their budget"
cd meal-helper
paja build
paja run --input "Choose two lunches for a total of eight euros"
paja test
paja improve
paja results
paja run --input "Choose two lunches for a total of eight euros"
```

The first version chooses lunches costing 11 euros against an 8-euro limit. Two of four repeated control trials fail. One deterministic repair chooses two soups costing 8 euros; all four trials then pass on the same frozen tests. This is **control evidence, not AI quality**.

- **Build:** validate the fixture specification and freeze an immutable package.
- **Test:** execute host assertions and retain outputs, tool observations, failures, and usage.
- **Improve:** make one supported fixture repair, rerun the locked suite, and compare with the baseline. A regressing candidate stays available for inspection and does not become current.
- **Run:** invoke any retained version. `run` is available immediately after `build`.

`paja status` shows the current version and its evidence. `paja results` shows a test or comparison. Each command also supports versioned `--json` output with explicit machine identities and retry keys. See the [CLI reference](docs/CLI.md) and [local walkthrough](docs/LOCAL_CONTROL.md).

## Implementation and checks

One local owner uses a serialized coordinator backed by SQLite transactions and content digests. The fixed fixture runtime has no network, credentials, arbitrary code, or external tool effects. Rust's standard file locks fence competing CLI processes. The release profile uses thin LTO and one code-generation unit; [validation](docs/VALIDATION.md) records measured performance and its limits.

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
python3 scripts/check_repository.py
```

Normal tests use synthetic fixtures. Live model tests require a future adapter, explicit credentials, and bounded authorization.

## Direction and contribution

The **target agent** is the versioned program being developed. **Workers** are separately authenticated coding, testing, or reviewing agents. The [worker protocol](docs/AGENT_PROTOCOL.md) describes future collaboration; workers are not supported by this owner-only prototype.

- [Product](docs/PRODUCT.md): intended outcome, scope, and quality evidence.
- [Generation](docs/GENERATION.md): problem intake, generated packages, and managed execution.
- [Architecture](docs/ARCHITECTURE.md): current implementation and intended boundaries.
- [Roadmap](docs/ROADMAP.md): remaining coordinator work, one real model adapter, and practical validation.
- [Decisions](docs/DECISIONS.md): confirmed direction and remaining choices.

The [agent specification](examples/agent.json) and [test suite](examples/cases.json) are the runnable starter inputs. The [status response](examples/status.json) illustrates the implemented envelope with a synthetic project ID.

See [Contributing](CONTRIBUTING.md), [Governance](GOVERNANCE.md), [Security](SECURITY.md), and [Releasing](docs/RELEASING.md). Licensed under [Apache License 2.0](LICENSE), with attribution in [NOTICE](NOTICE).
