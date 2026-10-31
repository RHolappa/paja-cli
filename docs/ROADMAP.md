# Roadmap and development

The Rust local control prototype is implemented; real-model execution and scoped workers are not. The delivery goal is “Build, test, and improve your AI agent” / “Rakenna, testaa ja kehitä tekoälyagenttiasi.”

## Current delivery

The complete owner-operated init/build/run/test/improve/results loop runs with the original meal-helper fixture. It preserves immutable versions, rejects stale mutations, replays duplicate requests, enforces fixture budgets, freezes comparisons, and retains regressing candidates without adopting them. Rust 1.99.1 and edition 2024 are pinned. See [Local control](LOCAL_CONTROL.md) for setup and [Validation](VALIDATION.md) for executed checks and measurements.

Milestone 1 is **partially complete**: its core local loop is implemented. The three separately authenticated worker sessions, scoped credentials, lease expiry, durable background execution, explicit cancellation, and their failure checks remain pending. The synchronous owner implementation is not the worker protocol or a first executable release.

## Voice prompting increment

The 2026-10-31 voice increment adds a native Mac app and a private browser client with generic Docker/VPS setup scripts. Both use the existing owner coordinator. The native app remains here; browser code, tests, and Docker setup have moved to the separate private [paja-web repository](https://github.com/RHolappa/paja-web). See [Voice](VOICE.md). This implements spoken prompt intake and the fixture loop, not problem-specific generation, worker collaboration, or target publication.

## Next user-facing capability: problem-to-agent generation

The 2026-10-26 product clarification makes the next useful capability explicit: a problem description should become an inspectable runnable agent, with its development and use managed through the CLI. See [Generation](GENERATION.md). The fixture alone does not deliver that outcome.

Implement one configured generation adapter and one compatible target runtime, beginning with synthetic support-drafting tasks and independently specified checks. Record proposed generated tests separately from authorized frozen suites. Validate problem-specific package generation, actual task execution, visible limits and unsupported capabilities, and a bounded evidence-based improvement. Synthetic adapter checks remain distinct from explicitly authorized live-model trials.

Worker collaboration and runtime specialist teams are different design choices. Their desired emphasis and whether Paja should start workers are still open. Generation work can progress on the owner path; the outstanding milestone-1 worker, durability, cancellation, and release requirements remain in force.

## Runnable repository checks

With Python 3.10 or newer:

```sh
python3 scripts/check_repository.py
```

The Python check covers local links, command-name consistency, required files, JSON syntax, and obvious private-path/credential mistakes. It does not execute the CLI or audit Git history. Run `cargo test --locked` for CLI control and failure tests; strict Rust parsing validates the implemented fixture inputs. No provider is tested.

## 1 Prove the development loop locally

Implement one original starter package, a small coordinator, a deterministic fixture runtime, and two meaningful cases. Support initialization, build, test, a bounded improvement, before/after comparison, and manual invocation with the proposed CLI.

Use three separately scoped worker sessions for build, test, and review. Test the protocol with scripts; paid worker models are not needed to prove concurrency and recovery.

**Acceptance:** a new contributor runs the complete example without cloud credentials, sees a known failure, produces a new candidate, reruns the locked cases, and inspects the comparison. The baseline stays intact. A regression is visible. All fixture execution is labelled as control evidence, not successful real AI behavior.

**Failure checks:** conflicting claims, expired worker completion, candidate change during tests, historical baseline evaluation, attempted suite tampering, fabricated pass reports, duplicate trial dispatch, budget exhaustion, restart, and cancellation. None may corrupt evidence, increase tool authority, or bypass limits.

## 2 Build and improve one real AI agent

Select and integrate one model/build/runtime adapter. Build the target's instructions/tool wiring from the declared job or import one compatible agent. Run fixture-backed tasks through the actual model, collect real outputs/tool calls, and repeat cases under recorded settings.

Improvement uses selected failures, one attempt by default, and explicit budgets. It produces a candidate and reruns comparable tests. Test authoring and target editing have separate permissions.

**Acceptance:** an owner with their own configured credentials creates a working agent, observes a real failure, evaluates a proposed change, and can explain regressions, uncertainty, usage, and latency. Missing model configuration fails honestly. Model/usage errors stay visible; no synthetic fallback is presented as live success.

Record provider/model/dependency versions and supported capabilities. This milestone proves only the tested adapter and tasks, not universal agent-framework compatibility.

## 3 Validate practical use and cooperating workers

Work with three external builders on their own tasks. Include a project where separate workers build, test, and review the same target. Observe whether people return for another improvement cycle and whether recorded evidence helps them make better changes.

Measure terminology friction, setup time, failures, misleading assessments, iteration cost, and recovery. If remote workers are needed, deploy one authenticated coordinator and measure its capacity; do not infer distributed scale from a local database.

**Acceptance:** independent users repeat the loop without maintainer coaching, worker races/crashes recover, and release notes accurately describe supported topology, adapters, and limits.

## Optional release path

After local development is useful, add a deployment adapter only for a concrete user need. Publication must freeze package, runtime/model configuration, tool grants, verified evidence, target, and exact-version approval. Reconcile uncertain activations and keep private tests/traces out of public assets.

Publication is not a dependency of the core loop. Visitor enrollment, web analytics, and a static-site platform have no milestones.

## First implementation work

| Work item | Deliverable |
| --- | --- |
| Contracts and starter | Validated schemas from the illustrative agent/case/status JSON, one package format, and two host-checkable cases |
| Shared authority | Owner/worker credentials, revisions, immutable artifacts, conditional tasks/leases, request receipts, and bounded cancellation |
| Runner and tests | Fixture mode, isolated target invocation, tool broker, deterministic assertions, trace provenance, and honest error outcomes |
| Build and improve | Version-producing adapters, locked evaluation inputs, one bounded candidate attempt, and baseline comparison |
| Human and agent journeys | Core-command walkthrough, structured output, schema discovery, and a three-worker script with failure checks |

One Rust package now contains CLI, engine, fixture runtime, examples, and tests. Toolchain and dependency versions are pinned in rust-toolchain.toml and Cargo.lock. Extend that package with the remaining coordinator and adapter work. Normal checks use synthetic data; live provider calls require explicit configuration and bounded authorization.

Follow [Releasing](RELEASING.md). The fixture CLI and voice clients are runnable; passing live target-model tests, worker capacity, and target publication are not established.
