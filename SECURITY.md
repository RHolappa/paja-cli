# Security policy

## Supported versions

There is no released product or supported production runtime yet. The current development branch accepts reports affecting the local owner/control prototype, repository tooling, and proposed security boundaries. A supported-version table will accompany the first executable release.

## Private reporting

On the public GitHub repository, use **Security → Advisories → Report a vulnerability**. Private vulnerability reporting must be enabled before public launch; writing this file does not enable the GitHub setting.

If that option is unavailable, ask a maintainer for a private reporting channel without disclosing the vulnerability in a public issue. Include affected versions, impact, reproduction with synthetic data, and any suggested mitigation through the private channel. Never attach real credentials, private user records, or exploitable production access.

Maintainers will acknowledge and investigate reports as availability allows, coordinate a fix and disclosure with the reporter, and identify affected versions. No response-time guarantee or bounty program is currently offered.

## Relevant boundaries

The native Mac companion processes speech locally and passes reviewed prompts through private files to the Rust coordinator. Model downloads require network access during explicit setup. See [Voice data handling](docs/VOICE.md#privacy-and-scope). The separately maintained private [paja-web repository](https://github.com/RHolappa/paja-web) owns browser authentication, server transcription, and deployment security; its service is one shared owner workspace.

Reports about coordinator access, worker impersonation, stale leases, revision races, target execution isolation, unauthorized tools, test/grade tampering, credential exposure, unbounded model calls, fabricated evidence, stale release approval, duplicate external actions, or unsafe trace export are especially useful. See [Architecture](docs/ARCHITECTURE.md) and [Agent protocol](docs/AGENT_PROTOCOL.md) for the proposed boundaries.

Tests should target local synthetic environments or explicitly authorized systems. Repository contribution does not authorize testing a third-party deployment.
