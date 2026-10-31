# Decisions

Updated 2026-10-31 for voice prompting and the browser repository separation.

## Confirmed direction

| ID | Decision | Consequence |
| --- | --- | --- |
| D1 | Independent source and operation | No private checkout, maintainer account, inherited branding, or production data is required |
| D2 | Shared workflow authority | Human and worker clients receive the same state transitions and authorization outcomes |
| D3 | Local control prototype implemented | Core commands, status/results, deterministic fixture execution, and owner-mode persistence run; real models and workers remain unimplemented |
| D4 | Paja CLI is a project of Innostium Oy | Preserve independent identity and stewardship |
| D5 | Full-platform feature parity is withdrawn | Unrelated community/platform capabilities stay removed |
| D6 | Public material uses synthetic data and independent branding | Exclude reference identities, private paths, production resources, credentials, and assets |
| D7 | Publication binds exact verified inputs | Changed instructions, code, model/runtime settings, tools, policy, or target invalidate release authorization |
| D8 | Retain Apache-2.0 and attribution | Establish rights and provenance before external imports |
| D9 | Superseded by D14 | The existing-static-app experiment and visitor-analytics focus is withdrawn |
| D10 | Superseded in part by D14 | Target-agent building and bounded execution are now core; external orchestrators still start worker agents |
| D11 | Start with a complete local control example | Use labelled deterministic fixtures to prove mechanics, then validate one real model/runtime adapter |
| D12 | Several agents cooperate on one project | One coordinator, scoped workers, private inputs, bounded tasks, and conflict-safe evidence remain required |
| D13 | Human clarity and unattended use are first-class | A short guided path and discoverable machine contracts share the same engine |
| D14 | “Build, test, and improve your AI agent.” | Versioned target agents, test evidence, bounded improvements, and regression comparisons define the product |
| D15 | “Rakenna, testaa ja kehitä tekoälyagenttiasi.” | Use the user's exact Finnish wording alongside the English promise |
| D16 | Rust, stable toolchain, and measured performance | Pin Rust 1.99.1, use edition 2024, standard file locks and let chains, and record release-build timings |
| D17 | Owner-only synchronous control slice first | SQLite transactions and receipts protect the complete local fixture loop; authenticated worker IPC and durable jobs remain required before a collaborative release |
| D18 | Keep rejected candidates and explicit evidence | Only a regression-free observed improvement becomes current; preserve baseline versions, locked tests, and source files |
| D19 | Generate useful agents from a described problem | The intended build path must turn the job, inputs, success criteria, and allowed tools into an inspectable runnable target; Paja manages its lifecycle and evidence |
| D20 | Voice prompting on Mac and in a cloud-hosted browser | Independent SwiftUI/WhisperKit and browser/server Whisper inputs produce editable drafts; using a draft is a separate explicit action through the Rust coordinator |
| D21 | Portable Docker/VPS setup in Paja Web | Keep SQLite on persistent private storage, protect the single-owner web workspace, and provide HTTPS; infrastructure deployment remains distinct from target-agent publication |
| D22 | Separate browser source from the CLI and Mac app | Browser UI, Python service, server transcription, tests, and Docker setup live in the private [paja-web repository](https://github.com/RHolappa/paja-web); consume this public CLI at an immutable revision |

D14–D15 reflect the latest user direction. D12 remains relevant: several worker agents can help develop the same target agent. These are different roles, and documentation must name them clearly. Visitor recruitment, event collection, and static-site hosting are not prerequisites or delivery tracks.

## Direction and implementation boundary

| Proposal | Rationale |
| --- | --- |
| Core commands init, build, test, improve, run | The interface directly expresses the promised development loop |
| One starter template and one compatible runtime/build adapter initially | Produce a concrete agent before generalizing to many frameworks |
| Frozen agent packages, test suites, tool fixtures, and run settings | Compare changes without moving the evaluation criteria |
| One improvement attempt by default | Preserve a baseline and bound generation, test retries, and spending |
| Separate task scopes for building, testing, reviewing, and releasing | A candidate cannot grant itself tools, alter its judge, or approve publication |
| SQLite/private artifacts behind one local coordinator first | Concurrent worker processes share one authority; remote topology requires separate validation |
| Optional publication after useful local agent development | Testing and improving an agent should not require hosting or visitors |

The command loop, one original fixture starter, immutable packages/suites, bounded fixture repair, and owner-mode coordinator are implemented. Worker scopes, real-model adapters, and optional publication are still proposals. No external compatibility promise or executable release has been made. See [Validation](VALIDATION.md).

## Resolve with implementation

- Clarify whether multi-agent collaboration primarily means development workers, runtime specialists solving the user's task, or both. These layers have different authority and lifecycle requirements; neither is implemented. See [Generation](GENERATION.md). Whether Paja starts worker processes is also open; D10 remains the current external-orchestrator proposal.
- Select one generation backend and compatible target runtime so a problem brief changes runnable behavior. Existing fixture goals/instructions are metadata and do not satisfy D19.

- Validate Linux in hosted CI and extend beyond local macOS validation; implement local IPC, credential issuance/revocation, worker leases, and durable recovery. Rust/Cargo and owner file-lock/SQLite serialization are chosen.
- Starter agent package format, input/output contract, one model/build adapter, and declared capability discovery.
- Target execution isolation, allowed tools, fixture/live separation, credential broker, and side-effect approval.
- Complete schemas using the illustrative [agent spec](../examples/agent.json), [cases](../examples/cases.json), [status](../examples/status.json), and [worker protocol](AGENT_PROTOCOL.md).
- Host assertion types, separately scoped model graders if used, repetitions, held-out cases, comparable-run criteria, and treatment of failed/skipped work.
- Token/call/runtime budgets, provider usage estimates versus actuals, improvement termination, and uncertain paid-operation reconciliation.
- Trace privacy, retention/deletion, export selection, and permission to turn real inputs into future test cases.
- Optional deployment adapter, exact-version approval/expiry/revocation, and publicly granted tool/cost limits.
- Measured worker capacity, supported topology, queue/lease/cursor limits, and private GitHub reporting contacts.

Repository publication, infrastructure deployment, paid live-model tests, and agent publication require their own authorization. Do not describe the new direction as deployed or live.
