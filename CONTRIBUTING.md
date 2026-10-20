# Contributing

Paja CLI is an open-source project of Innostium Oy. Contributions should help users build, test, and improve their AI agent using observable evidence and repeatable comparisons.

Read [Product](docs/PRODUCT.md), [Decisions](docs/DECISIONS.md), and [Roadmap](docs/ROADMAP.md). The local control loop runs in Rust with labelled deterministic fixtures and one filesystem owner. Scoped worker sessions remain to be implemented. [Agent protocol](docs/AGENT_PROTOCOL.md) defines the future worker contract; [CLI](docs/CLI.md) defines the current executable subset.

## Make a focused change

Describe the user problem, the affected workflow step, and an observable acceptance condition. A public issue must be understandable without private source access. Start from the first implementation tasks in the roadmap.

Use a focused branch and PR. Keep command examples, scope, and implemented status consistent. Explain the actual checks run and remaining limitations. Avoid empty packages, mock commands that imply real completion, or unrelated platform features.

Use the pinned Rust 1.99.1 toolchain (2024 edition) and Python 3.10 or newer:

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
python3 scripts/check_repository.py
```

Cargo fetches dependencies for compilation; commit Cargo.lock and use --locked for reproducible resolution. The Python repository check needs no extra dependencies. Product tests execute the CLI against synthetic fixture projects, including conflicting owner processes, retries, immutable evidence, budgets, and comparisons. Agent/case examples are compiled into the starter and strictly validated by the implementation. Credentials, model calls, and paid services are not needed. Worker authentication, leases, and cancellation tests belong to the next coordinator increment. See [Validation](docs/VALIDATION.md) for actual results and remaining limits.

## Provenance and privacy

Contribute original material or identify its source, exact files/version, redistribution permission, license, changes, and required notices. No external application source or assets have been imported. Reference inspection does not establish permission to copy.

Contributions use Apache-2.0; preserve attribution. No additional contributor license agreement is currently required. AI-assisted contributions receive the same correctness, privacy, provenance, and test review as other work.

Exclude private project/association branding, production configuration, real records, recordings, credentials, and machine paths. If required attribution conflicts with the independent identity, use an original implementation rather than deleting notices.

## Review and release

Follow [Governance](GOVERNANCE.md), the [Code of conduct](CODE_OF_CONDUCT.md), and [Security](SECURITY.md). See [Releasing](docs/RELEASING.md) for public history and releases. A contribution does not authorize deployment, provider spending, app publication, or repository publication.
