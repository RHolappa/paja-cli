# CLI

The eight core/supporting commands below are implemented for the **local control fixture only**. Default help leads with “Build, test, and improve your AI agent.” The intended Finnish promise is “Rakenna, testaa ja kehitä tekoälyagenttiasi.” No command calls a live model or starts a worker.

## Implemented commands

| Command | Current behavior |
| --- | --- |
| `paja init <name> --goal <text>` | Create a new directory with `paja.json`, `cases.json`, and private coordinator state; never overwrite an existing project |
| `paja build [--file <spec-file>] [--dry-run]` | Validate and freeze the fixture specification and default suite into an immutable version |
| `paja test [--suite <file>] [--repeat <count>] [--dry-run]` | Execute a retained version against its default frozen suite or an explicitly selected owner-authored suite |
| `paja improve [--from <test-run-id>] [--dry-run]` | Apply one supported fixture repair, test against the baseline's locked suite, and record a comparison |
| `paja run --input <text-or-json>` | Invoke a retained fixture version and record outputs and actual tool observations |
| `paja identity` | Discover the local owner project identity without mutating state; accepts `--json` without an existing project ID |
| `paja status` | Show current revision and the latest evidence for that revision |
| `paja results [--run <id>] [--compare <baseline-run-id>]` | Inspect retained outcomes, traces, changes, and comparisons |

Global options: `-C/--directory <path>`, `--json`, `--project <id>`, `--revision <n>`, and `--request-id <key>`. `--help` and `--version` print ordinary CLI help/version text. Running without a command shows status, or an initialization hint if no project exists.

Init also accepts `--goal-file <path>` instead of `--goal`; run accepts `--input-file <path>` instead of `--input`. These mutually exclusive inputs read bounded UTF-8 text, trim surrounding whitespace, and never execute it. Prompt paths resolve relative to `--directory` or may be absolute. File contents, rather than filenames, determine request identity; changed contents conflict with a previously completed request key. The native and browser companions use this handoff after explicit user submission.

All calls are noninteractive. Init requires an explicit goal; the goal and instructions are recorded metadata in this adapter, not inputs to a language model. Build reads `paja.json` by default. Later file edits cannot alter a built package. Relative configuration and suite paths must remain inside the project. Inputs are limited to 1 MiB and reject unknown fields and schema versions.

Test defaults to the suite frozen at build time. To evaluate a separately edited suite, explicitly pass `--suite cases.json`; its full content is frozen into that run. Changed suites, repeat counts, runtime/model settings, tools, or budgets produce a non-comparable result against earlier conditions.

Improve selects the latest test of the current candidate in human mode, or requires `--from` in JSON mode. It freezes the baseline and uses its stored suite even if the working file has changed. It can change `runtime.strategy` from `catalog_order` to `cheapest` after a budget assertion fails, and disable `request_purchase` after a purchase-boundary failure. It cannot modify tools, budgets, assertions, or fixtures. No failed cases means no arbitrary change. Unsupported repairs produce a visible no-change result.

An improvement becomes current only when at least one case improves, no case regresses, and neither run has execution failures. Other candidates remain inspectable through their run and revision. The baseline and working source files remain intact. A later build uses the working specification; copy a desired change from the comparison into `paja.json` before rebuilding.

## Fixture input

The fixture accepts exactly these demo phrases (case-insensitive, optional trailing period):

```sh
paja run --input "Choose two lunches for a total of eight euros"
paja run --input "Choose a lunch and buy it for me"
paja run --input '{"count":2,"budget_cents":800,"purchase":false}'
```

Arbitrary language is rejected. Prices use integer cents. Repeated catalog items are allowed. This models no inventory or purchasing service. Only `catalog.read` and `calculate` may be granted; purchase requests are recorded and denied.

The first phrase also accepts `8 euros` in place of `eight euros`, accommodating the observed voice transcript without adding arbitrary language interpretation.

## Machine use

`--json` emits one `paja.result.v1` envelope with `schema_version`, `ok`, `project`, `run`, `data`, `next_actions`, and `error`. Nullable fields remain present. Parser and validation errors use the same envelope. No command prompts, launches a browser, or infers paid consent.

Init requires `--request-id`; it creates the project identity and accepts no supplied project/revision. Project reads require `--project`, except for the explicit local-owner discovery command `paja identity --json`; results also requires `--run`. Mutations require explicit `--project`, `--revision`, and `--request-id`. Improve also requires `--from`. Dry runs require project and revision but no request key. Supplying a request key in human mode also requires explicit project/revision for mutations.

Build and improve compare the expected revision with the current candidate. Test and manual run may target a retained historical revision; their results cannot verify the current candidate. Unknown or mismatched project identities fail.

```sh
paja build --project prj_demo --revision 0 --request-id build-001 --json
paja test --project prj_demo --revision 1 --request-id test-001 --json
paja improve --project prj_demo --revision 1 --from run_000002 --request-id improve-001 --json
paja results --project prj_demo --run run_000003 --json
```

Replace the synthetic project ID with the ID returned by init. Persist a request key before dispatch and reuse it with the same inputs after losing a response. Completed receipts are replayed before current-revision checks. A reused key with changed inputs returns `idempotency_conflict`. Retrying a build also requires the same source contents. Read errors use a null project/run rather than implying a state change.

Operations execute synchronously inside an exclusive coordinator lock and SQLite transaction. A response means the result and receipt committed. Test execution can succeed while assertions fail: exit 0 means the request succeeded, **not that every case passed**. Inspect `run.state`, `report.counts`, and `comparison`. Budget-blocked trials are execution failures. A busy coordinator returns retryable exit 6; use the same request key. There are no durable background jobs, worker leases, or cancellation command yet.

| Exit code | Meaning |
| --- | --- |
| 0 | Completed operation or successful read; inspect outcomes |
| 2 | Invalid input or unsupported capability/schema |
| 3 | Project identity mismatch (worker authentication is not implemented) |
| 4 | Revision, destination, or idempotency conflict |
| 5 | Storage, I/O, or integrity failure |
| 6 | Coordinator busy; retry with the same key |
| 7 | Required input or trial budget absent; no operation dispatched |

## Intended generation experience

The next product capability is [problem-to-agent generation](GENERATION.md): `paja init` captures a concrete problem and constraints; `paja build` uses a configured generation adapter to produce an inspectable runnable agent; `paja run` executes it; and test/improve retain evidence and bounded changes. The current fixture stores the goal as metadata and does not implement this behavior.

Managed execution includes visible setup needs, resource limits, progress, outputs, and recovery state. Guided intake and model configuration need explicit contracts before new flags are advertised. Development workers and specialist agents solving the actual task are separate collaboration layers; their intended emphasis remains an open choice. No new commands or model/team flags are implemented by this design update.

## Proposed later operations

These commands and options **do not exist yet**. They remain design contracts, not installable features.

| Command | Proposed behavior |
| --- | --- |
| `paja tasks <action>` | Authenticated worker list, claim, heartbeat, complete, and fail operations with fenced leases |
| `paja stop --run <id>` | Durable cancellation, reporting confirmed versus pending effects |
| `paja export --output <directory>` | Explicitly selected private-safe export |
| `paja schema <resource> --json` | Discover complete versioned schemas and supported scopes |
| `paja approve --project <id> --revision <n>` | Authorize an exact verified release with expiry |
| `paja publish` | Publish only an explicitly authorized verified package to a supported target |

Existing-agent import (`init --from`), guided prompts, watching (`status --watch --jsonl`), pagination, remote workers, and live model/provider capabilities are also future work. Publication is optional and separately authorized. See [Agent protocol](AGENT_PROTOCOL.md).
