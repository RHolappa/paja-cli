# Changelog

## Unreleased

- Add a native SwiftUI voice companion with local WhisperKit transcription and an editable draft.
- Move the browser companion, server speech transcription, tests, and Docker/Caddy setup to the separate private [paja-web repository](https://github.com/RHolappa/paja-web). Keep the CLI and Mac app here.
- Add `paja identity`, reviewed prompt files for init/run, and request fingerprints based on file contents. Both companions use the existing Rust coordinator.

- Clarify the intended problem-to-agent generation experience and managed CLI lifecycle; distinguish proposed runtime specialists from development workers. No generation backend or team runtime is implemented by this design update.

- Implement the local control loop in Rust 2024 with pinned stable Rust 1.99.1 and locked dependencies.
- Add init/build/run/test/improve/status/results with deterministic meal fixtures and host assertions.
- Preserve immutable SQLite packages/runs, candidate revisions, retry receipts, locked suites, and rejected candidates.
- Expose actual fixture outputs/tool observations, blocked execution, regression comparisons, and versioned JSON errors.
- Add CLI integration tests, release-build benchmarks, and Rust CI configuration.
- Keep real-model adapters, authenticated workers, durable jobs, cancellation, and publication explicitly unimplemented.

### Earlier design work

- Focus the product on “Build, test, and improve your AI agent.”
- Use the Finnish promise “Rakenna, testaa ja kehitä tekoälyagenttiasi.”
- Replace static-app experiments and visitor analytics with agent versions, frozen test suites, bounded improvement, and regression comparisons.
- Distinguish the target agent from worker agents collaborating on its development.
- Define init/build/test/improve/run as the core proposed CLI path, with shared tasks and optional release operations.
- Add synthetic agent, test-case, and status examples, subsequently aligned with the implemented fixture contract.
- Retain Innostium Oy stewardship, Apache-2.0 attribution, contribution/security policies, GitHub templates, and repository checks.

No package release or GitHub publication is included. The current executable is a local control prototype, not evidence of AI-agent quality.
