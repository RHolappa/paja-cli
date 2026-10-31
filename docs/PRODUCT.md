# Product

## Implementation status

The Rust control prototype runs init/build/run/test/improve and records immutable evidence and comparisons locally. Its deterministic meal helper is a demonstration of the development mechanics. The goal and instructions are stored metadata; no model builds an agent from them. Only the filesystem owner can operate this prototype. Real-model execution and worker collaboration below are intended product behavior, not current capabilities. See [CLI](CLI.md), [local control](LOCAL_CONTROL.md), and [validation](VALIDATION.md).

## The promise

**Insight → Idea → Prototype.**

Paja's intended journey uses multi-agent development workflows to turn an observed problem into a testable idea and a runnable agent prototype. The concrete development loop remains:

**Build, test, and improve your AI agent.**

**Rakenna, testaa ja kehitä tekoälyagenttiasi.**

Paja CLI is an open-source project of Innostium Oy. Its first user wants an agent that performs a concrete task reliably and needs a practical way to build, inspect, test, and improve it.

The user should be able to start with a problem description and expected outcome. Paja should generate the instructions, supported tool wiring, and runnable package, then manage execution and evidence through the CLI. Manually configuring a prebuilt fixture does not fulfill this product goal. The intended journey and open collaboration choices are in [Generate an agent from a problem](GENERATION.md).

```text
Define the job → Build a version → Test it → Inspect failures
                     ↑                           ↓
                     └──── Improve and compare ──┘
```

Trying the agent yourself is part of the loop. Publishing it is a separate optional step. Neither a public website nor recruited visitors is required to get value from Paja.

## Insight → Idea → Prototype

| Stage | Intended retained output | Transition |
| --- | --- | --- |
| Insight | A problem brief with observations, supporting evidence, context, and open questions | Select a problem worth addressing; keep assumptions distinguishable from observations |
| Idea | A proposed agent job, approach, allowed tools, constraints, and success criteria | Review the approach and separately authorize the evaluation cases |
| Prototype | An inspectable agent version, task outputs, frozen test results, and baseline comparisons | Run, test, review, and make a bounded improvement |

The prototype is the target AI agent. The pipeline should retain links from the original problem and selected idea to the built version and its evidence. These stage records and transitions are proposed product behavior, not new commands or implemented schemas.

Multi-agent workflows describe development workers collaborating through the coordinator: a builder proposes versions, a tester evaluates authorized frozen cases, and a reviewer examines comparisons. Humans may perform these roles. Worker handoffs require scoped authority and immutable inputs; role names alone grant no permissions. Runtime specialists within the target remain a separate design choice.

## Two meanings of agent

| Term | Meaning |
| --- | --- |
| Target agent | The versioned instructions, model/runtime settings, code, and allowed tools being built and tested |
| Worker | A separately authenticated coding, testing, or reviewing agent helping develop that target |
| Test run | Execution of one target version against a frozen suite, fixtures, tool policy, and run configuration |
| Improvement | A bounded change producing a new candidate and a comparison against a baseline |

A person may do all the work. Several workers may share the same project through one coordinator. Paja owns versions, work records, execution limits, and evidence; it does not require workers to share a chat history or writable directory.

## First use

Ask what the target agent should accomplish, what information/tools it may use, and examples of good or unacceptable results. Save an editable project specification. Start with one template or import an existing compatible agent. The first implementation supports one bounded agent runtime adapter.

The planned generation adapter turns that specification into inspectable artifacts using explicitly configured resources. It reports missing inputs and unsupported tools before presenting a package as ready. Proposed acceptance cases receive separate authorization and are frozen before evaluation. The initial generated target uses one compatible runtime; adding specialist runtime roles remains a separate collaboration design choice.

Build produces a concrete version. Tests show which cases passed, failed, were blocked, or were skipped, with outputs and tool observations. Improvement uses selected failures to propose a new version and reruns the locked tests. The original version and evidence stay available.

A deterministic fixture adapter supports contributor tests without credentials. It must be labelled as a control fixture, not demonstrated AI quality. Trying a real model requires explicit provider configuration, budgets, and visible usage reporting.

## Scope

| Capability | Outcome |
| --- | --- |
| Define and build | Goal, instructions, input/output contract, allowed tools, model/runtime settings, and resource limits become a runnable version |
| Try | Run one task interactively or through structured input and inspect its actual actions |
| Test | Frozen cases, fixtures, assertions, and optional reviewed graders produce attributable evidence |
| Improve | A bounded attempt creates a candidate from selected failures without modifying the baseline or weakening its tests |
| Compare | Baseline and candidate results use declared comparable conditions and expose regressions, variance, cost, and latency |
| Coordinate workers | Independent claims, private inputs, revision checks, and scoped authority protect concurrent work |
| Recover and export | Interrupted clients, failed workers, and provider uncertainty preserve receipts; exports omit credentials and private traces unless explicitly selected |
| Release optionally | Only an explicitly approved, verified package and runtime/tool configuration may become public |

## A concrete example

The owner builds a lunch assistant. A test gives it a fictional catalog and asks it to choose two lunches within eight euros. The host checks the selected items against fixture prices and records the actual tool calls. Another case checks that the agent does not buy anything when purchasing is outside its allowed tools.

A builder worker changes the instructions or implementation after a failure. A testing worker runs the same suite against the new candidate; a reviewer examines the comparison. A faster or cheaper candidate that breaks a required case is a visible regression, not an overall improvement hidden by an average score.

The target agent cannot grant itself tools. A builder cannot make its change pass by editing expected answers. Test changes require separate authority, a new suite version, and a new comparison of both agent versions.

## Evidence rules

Keep deterministic assertions, model-graded assessments, fixture-only control runs, manual trials, and later production observations distinct. Store the candidate digest, suite/fixture/tool versions, model settings, requested and resolved model identifier when available, repeat count, and actual outputs.

A failed provider call is blocked or failed execution, not a zero-quality answer or a passing case. Repeating model tests may produce different outputs; report sample size and variation. A single passing run does not prove reliability, general intelligence, or real demand.

The first useful result is a specific next change supported by a recorded failure and regression comparison. No universal quality score is promised.

## Invariants and boundaries

- One coordinator owns transitions, task authority, immutable versions, and accepted evidence.
- Workers use separate credentials and immutable inputs; stale leases/results cannot alter a current candidate.
- Target execution has an explicit tool allowlist and bounded time, calls, tokens, and spending where enforceable. Default tests use fixtures without consequential external actions.
- Improvement cannot rewrite its locked tests or run an unlimited self-improvement loop.
- Retries reuse completed steps; uncertain paid/provider work is reconciled or reported before a new attempt.
- Disconnecting a client does not cancel durable work. Local work pauses when its coordinator is unavailable.
- Credentials, private inputs, traces, and lessons do not become public implicitly.
- Optional publication binds exact verified package bytes, test evidence, model/runtime settings, tools, target, and authorization.

The earlier static-app experiment, visitor enrollment, web analytics, and hosting-first workflow is removed. Community accounts, competitions, galleries, characters, media generation, and an independent coding-agent service remain outside scope. The 2026-10-31 request adds voice prompting through native Mac and optionally deployed browser companions. These are owner interfaces to the development loop, not a restored community platform. The native app remains here; browser source and deployment tooling live in the separate private [paja-web repository](https://github.com/RHolappa/paja-web). See [Voice](VOICE.md). Speech input does not implement problem-to-agent generation.

## Practical validation

A builder should create an agent, discover a meaningful failure, make a bounded change, and explain the before/after evidence without learning internal IDs. Three separately scoped workers must be able to collaborate without corrupting versions or self-authorizing release.

Observe external builders using their own tasks and voluntarily returning for another improvement cycle. Measure recovery and concurrency on the implemented coordinator before advertising scale. Fixture tests prove control behavior; real model tests and repeated real use supply different evidence.
