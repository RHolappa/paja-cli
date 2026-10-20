# Proposed worker protocol

This is the proposed protocol for several workers developing one target agent. Authenticated worker sessions, IPC, tasks, leases, watches, and cancellation are not implemented. The Rust local owner prototype implements the core JSON envelope, immutable versions, expected revisions, locked suites, and idempotent receipts. See [CLI](CLI.md) for its precise executable subset. The [project spec](../examples/agent.json) and [cases](../examples/cases.json) are runnable control inputs; [status](../examples/status.json) illustrates the current draft-state response.

## One project authority

This protocol concerns development workers. Agents collaborating while solving the target's actual job would be runtime components of that target, not workers with access to this protocol. See [Generation](GENERATION.md) for the open collaboration scope decision.

The owner configures one coordinator and separately scoped credentials. External orchestrators start workers. Workers receive private inputs and submit artifacts; they never share owner tokens, a writable checkout, or direct database access.

| Capability | Permitted work |
| --- | --- |
| Build/improve | Propose agent packages using assigned inputs and allowed failure feedback |
| Execute tests | Run assigned frozen cases through the trusted runner and submit linked evidence |
| Author tests | Propose a separately reviewed suite version; cannot silently alter an active improvement run |
| Review | Read authorized traces/comparisons and submit an attributed recommendation |
| Approve release | Review one verified package and create/revoke exact-version authorization |
| Publish | Consume a valid approval using separately scoped publisher credentials |

The target agent has only its runtime tools, not these coordinator capabilities. Worker labels are not authentication. Task types and dependencies are host-controlled; model output cannot create permissions or rewrite required evaluation policy.

## Versions and tasks

A project ID identifies the agent being developed. Candidate revisions identify immutable package instructions/code/configuration. Suites, fixtures, grader policies, and run settings have independent versions. A run freezes these references before dispatch.

Candidate-changing operations compare against the expected current revision. Historical test/manual runs may deliberately address retained immutable revisions. Task completion binds to its assigned run/revision and current lease. An authorized old-version evaluation can be stored as historical evidence; it cannot mark the current candidate verified.

The engine deduplicates work by project, input package, suite/settings, task kind, and declared trial/attempt ID. Each intentionally repeated model trial has its own identity; a network retry of that trial reuses its receipt. New request keys alone must not duplicate the same logical trial.

A task carries dependencies, input references, output shape, attempt/deadline limits, and required scope. Its states are ready, claimed, completed, failed, cancelled, and superseded. Only one worker obtains its current lease through an atomic claim.

Leases contain an authenticated owner, ID, increasing generation, and expiry. Heartbeats renew within a fixed deadline. Expiry fences the old lease before bounded reassignment. Completion rechecks authority, lease generation, run eligibility, and exact inputs. A late worker cannot write through its successor's lease.

Completing a task means its output was accepted. The host separately verifies whether the output satisfies a required test. A text report that says “passed” is not executable evidence.

## Typical cooperation

1. A builder receives the target goal, allowed tools, and editable package inputs and proposes a candidate.
2. The coordinator accepts the candidate against an expected revision.
3. Independent functionality and tool-boundary checks run in parallel over that frozen package.
4. A reviewer reads the comparison. A bounded improvement task receives selected failures and proposes a new candidate.
5. The trusted runner tests candidate and baseline under the locked suite/settings; the result preserves failures and limitations.
6. Optional release follows a separate exact-version approval.

Example calls do not run:

```sh
paja tasks claim --project prj_demo --task task_tools --revision 4 --request-id claim-001 --json
paja tasks heartbeat --project prj_demo --task task_tools --lease lease_demo --revision 4 --request-id beat-001 --json
paja tasks complete --project prj_demo --task task_tools --lease lease_demo --revision 4 --request-id complete-001 --result ./test-result.json --json
```

Real clients generate request keys, persist them before dispatch, and reuse them on retry. IDs in these examples are synthetic.

## Retries and cancellation

Mutation keys are scoped to principal, project, and operation kind. Persist normalized input fingerprints and operation records before dispatch. Identical retries return the recorded run/receipt; differing inputs produce `idempotency_conflict`.

Deduplicate an accepted request before retry-time revision checks, while checking current access. A lost response must not turn a successful mutation into new work. A caller cannot read another worker's private receipt by guessing a key.

Revision conflicts require rereading and deciding whether new work is appropriate. Do not automatically rebase a mutation onto the newest candidate. Reassigned workers retain logical step identities, budgets, and completed receipts.

Unknown paid/model/tool outcomes produce `reconciliation_required` when they cannot be safely resolved. Do not promise exactly-once arbitrary external effects. Report uncertain usage separately; do not silently hide it behind a retry.

`paja stop` records cancellation for a named run, fences further task commits/calls, and reports actual pending or terminal state. A disconnected watcher is not cancellation. Calls already issued can finish or incur usage; their late results cannot change the cancelled candidate.

## Improvement boundaries

An improvement task freezes its baseline, feedback selection, suite, fixtures, graders, comparison settings, attempt limit, and resource budget. Builder access does not include changing expected answers, removing a failed case, increasing tool privileges, or approving release.

An authorized suite change produces a new version. Comparing across different suites/settings is labelled non-comparable unless both agent versions are rerun under the new conditions. Held-out cases remain outside builder feedback permissions when used. An average score cannot conceal a required-case regression.

Report cases as passed, failed, blocked, or skipped, including repeated-trial counts. Keep deterministic assertions, model judgments, fixture control runs, and real model results distinct. Neither the target nor a reviewing worker can substitute its own unsupported quality claim for observed evidence.

## Machine contract

One `--json` response has `schema_version`, `ok`, `project`, `run`, `data`, `next_actions`, and `error`. Nullable fields stay present. The version is `paja.result.v1`. A project reports `id`, `candidate_revision`, and `state`; a run reports ID, kind, revision, and actual saved state.

Project display states are draft, building, built, testing, tested, improving, and blocked. They describe current candidate activity, not an overall quality guarantee. Run states are queued, running, waiting_for_input, succeeded, failed, and cancelled. A test run can succeed as an execution while containing failed cases; machine clients inspect the outcomes and comparison.

Errors contain stable code, message, retryable flag, optional retry delay, and typed details. Next actions name known operations with typed arguments and required scopes, never arbitrary shell strings. Returned artifact paths must stay within permitted boundaries.

| Exit code | Meaning |
| --- | --- |
| 0 | Request accepted or read succeeded; inspect run state and test outcomes |
| 2 | Invalid input or unsupported schema/capability |
| 3 | Authentication or permission failure |
| 4 | Revision, lease, idempotency, or cursor conflict |
| 5 | Non-retryable execution/provider failure |
| 6 | Retryable temporary failure; respect guidance and retain the request key |
| 7 | Required input, budget, or approval absent; no requested side effect dispatched |

`paja schema` will advertise versioned project, package, case, run, task, error, and action schemas. Unknown required versions fail before mutation. The fixture agent/case examples are strictly validated by the Rust implementation. Complete worker/action schema discovery remains future work.

## Watching and limits

Snapshots include a consistent event cursor. Watch events use `paja.event.v1` and identify project, relevant revision/run, event ID, cursor, kind, and typed data. Clients deduplicate events. Expired cursors require a fresh snapshot, not an assumption that no changes occurred.

Paginate task/results lists and fetch large traces separately. Bound worker concurrency, trials, attempts, model/tool calls, tokens, runtime, artifact sizes, and applicable spend before dispatch. Unavailable usage information is labelled. Budgets for externally launched worker sessions remain with their orchestrator; Paja cannot claim to bound spending it does not control.

Validate using separately authenticated sessions: conflicting claims, crashed workers, late completions, simultaneous independent results, candidate changes during testing, locked-suite violations, invented check evidence, exhausted budgets, lost responses, cancellation, and expired cursors. Measure actual capacity before advertising scale.
