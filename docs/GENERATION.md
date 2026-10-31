# Generate an agent from a problem

Design direction recorded on 2026-10-26. This capability is not implemented. The current Rust executable runs the labelled local control fixture described in [Local control](LOCAL_CONTROL.md).

## Intended experience

The user describes a problem and the outcome they need. Paja turns that brief into a runnable, inspectable agent project and manages its build, execution, testing, and improvement. The user should be able to start without manually designing prompts, tool wiring, or agent roles.

The product journey is **Insight → Idea → Prototype**. An insight records an observed problem and its evidence; an idea proposes the agent's approach, capabilities, and success criteria; a prototype is a versioned runnable target with attributable test results. The generation adapter connects the reviewed idea to that prototype. Linking these stages is intended behavior; the current fixture only stores goal metadata.

A synthetic starting problem:

> Read incoming support requests and the supplied support policy. Draft a grounded answer, flag requests that require a person, and return the policy passages supporting each draft. Leave sending to the user.

Paja captures the available inputs, expected outputs, allowed tools, examples of acceptable and unacceptable behavior, and resource limits. It identifies missing information that materially changes the solution. A guided human flow may ask for it; unattended requests receive structured missing-input results and never infer permissions or spending limits.

The problem remains editable and versioned. Generated instructions, configuration, and any supported code are inspectable artifacts tied to that problem version. `paja build` must eventually produce a different, appropriate package for a different problem; changing a descriptive label on a fixed fixture does not satisfy this requirement.

## The CLI manages the lifecycle

| Operation | Intended responsibility |
| --- | --- |
| `paja init` | Capture the problem, inputs, success criteria, and declared constraints into a project |
| `paja build` | Use a configured generation adapter to create instructions, tool wiring, and a runnable version; explain capabilities and unresolved setup |
| `paja run` | Give a concrete task to that version and return its result, tool observations, usage, and any need for human input |
| `paja test` | Evaluate real behavior against separately authorized, frozen cases and assertions |
| `paja improve` | Use selected failures to create a bounded candidate and compare it with the preserved baseline |
| `paja status` / `paja results` | Explain what exists, what ran, what failed, what changed, and the next useful action |

These are the intended meanings of the existing commands, not additional implemented capabilities or new flags. See [CLI](CLI.md) for what runs today.

Managed operation means Paja retains the problem and package versions, task ownership, execution state, outputs, usage, limits, and comparison evidence. The Rust coordinator is the authority for those records. A generation backend proposes artifacts; it cannot make itself the evaluator or grant additional tools. Durable runs, cancellation, and resume behavior require the remaining coordinator work before they can be advertised.

## Two possible collaboration layers

Development workers collaborate on building, testing, and reviewing a target agent. This is the existing [worker protocol](AGENT_PROTOCOL.md). Their credentials, inputs, leases, and task authority are separate from the target's runtime tools.

Runtime specialists would collaborate while solving the user's actual problem. For example, a support target could delegate policy retrieval, draft preparation, and escalation detection. Those roles would be components of the versioned target package. They would have declared inputs/outputs, handoffs, tool grants, and a shared bounded run budget. They would not acquire development-coordinator or evaluator authority by collaborating.

The public pipeline description highlights development workers collaborating on a prototype. Whether runtime specialist teams are also needed remains an open product decision. Development workers are not proof that runtime teams exist. The current fixture implements neither layer. External orchestration remains the current worker-starting proposal; having Paja start workers through an adapter requires an explicit lifecycle and resource-ownership decision.

A useful initial proposal is one generated target with one runtime adapter. Specialist roles should be added when a concrete task benefits from their division of responsibility, and evaluated against the simpler baseline. Role count, handoff count, and retries need limits; more agents alone are not evidence of better results.

## Generation and evaluation stay distinct

A generation backend may draft acceptance cases alongside an agent proposal. Such cases remain proposed tests until separately authorized from the problem requirements. Freeze the suite, fixtures, and grading policy before evaluation and improvement. Keep test authorship and any model judgments visible. A generator's own passing examples do not establish general reliability.

The build output must record its generation inputs, adapter/model identity where available, configuration, declared capabilities, and actual or unavailable usage. Missing model configuration is a setup error. An unsupported tool remains unresolved. Neither condition silently falls back to fixture success.

Provider configuration and bounded authorization are prerequisites for paid generation or runtime trials. Building the integration and testing its protocol with synthetic recordings does not authorize live calls. Unknown external outcomes require reconciliation before retrying consequential work.

## Next useful increment

Implement one compatible generation adapter and one target runtime path on top of the existing version/evidence engine. Start with the synthetic support-drafting problem, explicit policy inputs, a structured draft/escalation output, and a small independently specified suite. It should be possible to inspect the generated package, run new requests, observe a concrete failure, and compare a bounded change.

Acceptance requires the problem and constraints to affect the generated runnable behavior, a real configured model to execute the tasks, visible unsupported capabilities and failures, and retained before/after evidence. Record separately what synthetic integration checks establish and what authorized real-model trials actually demonstrate. This does not complete the outstanding worker, durability, cancellation, or release gates in [Roadmap](ROADMAP.md).
