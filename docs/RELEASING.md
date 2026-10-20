# Publication and releases

The local Rust control prototype executes, with validation recorded in [Validation](VALIDATION.md). GitHub publication, hosted checks, releases, and repository settings have not been established. The worker acceptance requirements below remain pending.

## Public source

Run `python3 scripts/check_repository.py` and review the complete diff, untracked files, and intended archive. Inspect for credentials, private names/domains, machine paths, production identifiers, user data, and assets. The checker is not a complete secret scanner.

Review the Git history, refs, tags, commit messages, author metadata, and large-file objects intended for publication. The original local history contains obsolete reference branding. **Do not push that history as the public repository.** Use a separate directory containing the reviewed current files and fresh public history. Keep the existing checkout intact unless rewriting it is separately requested.

Include LICENSE and NOTICE. Exclude Git internals, private state, credentials, caches, and generated evidence. Regenerate publication archives from the current scope; earlier snapshots are obsolete. Preserve required attribution.

## GitHub setup

After repository publication is authorized:

- Choose the verified owner and use “Build, test, and improve your AI agent. A project of Innostium Oy.” Use “Rakenna, testaa ja kehitä tekoälyagenttiasi.” for the Finnish promise.
- Record maintainer handles and a monitored private conduct contact in [Governance](../GOVERNANCE.md); use verified accounts for CODEOWNERS.
- Enable the supplied issue/PR templates and create work items from [Roadmap](ROADMAP.md).
- Enable [private vulnerability reporting](https://docs.github.com/en/code-security/how-tos/report-and-fix-vulnerabilities/configure-vulnerability-reporting/configure-for-a-repository).
- Run **Repository checks**, require its **repository** job and maintainer review, and protect the default branch from force pushes/deletion.
- Keep ordinary PR checks read-only and free of provider/deployment secrets. Enable available secret scanning and dependency alerts.
- Check README rendering, license recognition, templates, links, and the reporting channel in a fresh public checkout.

The workflow pins its checkout action and configures the Python check plus Rust formatting, Clippy, tests, and release builds on Linux/macOS. A local passing result is not a hosted CI result.

## First executable release

Release the local control example only when a fresh checkout can complete init/build/test/improve/run and the shared-project worker example without cloud credentials. Label fixture execution as control evidence. Verify scoped credentials, conflicting claims, stale results, locked tests, bounded improvement, baseline preservation, restart, and cancellation.

Before an AI-agent alpha, demonstrate one actual model/runtime adapter using explicitly authorized bounded calls. Record real outputs, tool observations, case outcomes, repeated-trial limitations, comparable baseline/candidate runs, and usage gaps. Include setup, supported package/tool contracts, isolation, budgets, trace privacy/retention, cancellation, export, and recovery instructions. Do not substitute fixture success for real-model evidence.

Identify the release commit, schema versions, supported coordinator/worker topology, and known limitations. Verify package contents, dependencies, licenses, checksums, and the independent install. Claim only concurrency and capacity demonstrated by tests or measurements. Keep publishing credentials out of PR jobs. Repository publication, package release, infrastructure deployment, and app publication need their own authorization.

Optional publication needs its own exact-version approval and deployment-reconciliation checks. Evaluate usefulness through external builders repeating the improvement loop. Documentation checks and fixture execution do not establish agent quality or usefulness.
