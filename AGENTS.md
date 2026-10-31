# Working in Paja CLI

Paja CLI is an open-source project of Innostium Oy focused on “Build, test, and improve your AI agent.” Finnish: “Rakenna, testaa ja kehitä tekoälyagenttiasi.”

Frame the intended product journey as “Insight → Idea → Prototype,” with planned multi-agent development workflows. A prototype is a versioned target AI agent. Keep the runnable control fixture distinguishable from proposed generation, linked pipeline stages, and worker collaboration.

## Current phase

Read `README.md`, `docs/PRODUCT.md`, and `docs/DECISIONS.md` before substantive work. The Rust local control prototype implements init/build/test/improve/run plus status/results using deterministic fixtures. Real-model adapters, worker sessions, and publication remain proposals. See `docs/CLI.md` and `docs/VALIDATION.md` for implemented scope.

For planning requests, keep all affected documents and examples consistent. When implementation is requested, begin with the complete local build/test/improve control example in `docs/ROADMAP.md` and record actual validation.

This repository contains the Rust coordinator and native Mac voice companion. Browser UI, Python service, server transcription, and Docker setup belong to the separate private `RHolappa/paja-web` repository, which consumes a pinned CLI revision. Speech recognition is separate from the still-proposed real target-agent runtime. See `docs/VOICE.md` for native setup and client boundaries.

## Scope

The target agent is the versioned program being built and tested. Workers are separately authenticated coding/testing/review agents collaborating on that project. Keep these roles distinct.

Focus on the target's job, instructions/tools, runnable versions, test evidence, bounded improvements, and baseline comparisons. Use one local coordinator and one starter/runtime adapter first. Read `docs/AGENT_PROTOCOL.md` for worker concurrency.

The earlier static-app experiment, visitor recruitment, collector, and hosting-first design is superseded. Removed community/platform capabilities have no deferred backlog. Optional publication follows useful local agent development and separate authorization.

## Boundaries

- Keep this repository independent of private reference projects. Inspection does not authorize code imports or source modifications.
- Use Paja CLI/Innostium Oy branding, generic domains, and synthetic data. Exclude reference identities, private paths, production identifiers, credentials, user records, and assets.
- The user-authorized public demo URL may appear in README testing instructions. Keep the hosted demo's infrastructure, provider/account/resource details, operational receipts, and credentials in ignored local files; public self-hosting instructions remain generic.
- Establish provenance and redistribution rights before imports and preserve required notices.
- Treat repository publication, infrastructure deployment, paid provider tests, and target-agent publication as distinct actions.

## Invariants

- One coordinator enforces versions, transitions, task ownership, and authority for human and worker clients.
- Workers have separate credentials, private immutable inputs, and fenced leases; they never share a writable checkout or direct database access.
- The target agent has only declared runtime capabilities and cannot control its evaluator or development coordinator.
- Test suites, fixtures, and graders remain frozen during improvement. Their changes require separate authority and a new comparison.
- Stale results cannot verify a new candidate. Authorized historical tests remain attached to their original run.
- Model assertions, fixture control runs, deterministic checks, real model trials, and production observations remain distinguishable.
- Improvement is bounded and preserves the baseline. Report regressions, execution failures, variation, and usage gaps honestly.
- Retries retain logical trial identity and completed receipts; uncertain paid/external effects require reconciliation.
- Disconnecting is not cancellation. Cancellation reports actual state and fences late commits.
- Private code, credentials, inputs, traces, and lessons do not become public implicitly.
- Optional release authorization binds the exact verified package, runtime/model configuration, tool permissions, policy, and target.
- Machine output is versioned and structured, with logs separate. Noninteractive calls do not prompt or infer consent.

## Validation

Run `python3 scripts/check_repository.py` for repository changes. For Rust changes, run `cargo fmt --all -- --check`, `cargo clippy --locked --all-targets -- -D warnings`, and `cargo test --locked`. Check local links, command consistency, JSON examples, removed-scope references, and claims of implemented behavior.

As code arrives, test meaningful outcomes with synthetic fixtures. Real-model tests need explicit credentials and bounded authorization. Never present control-fixture results as agent quality, invent passing tests, or install dependencies solely to validate documents.
