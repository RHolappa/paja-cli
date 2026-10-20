# Governance

Paja CLI is a project of Innostium Oy, which stewards its direction and appoints maintainers. Its scope is the agent build/test/improve workflow in [Product](docs/PRODUCT.md). Contributors and users do not need a company-hosted service.

## Decisions and reviews

Contributors propose work through issues and pull requests. Maintainers review correctness, scope, tests, provenance, usability, and operational impact. Significant choices are recorded in [Decisions](docs/DECISIONS.md), including rationale and consequences. Routine fixes do not require a separate design process.

At least one authorized maintainer reviews each PR before merge. Security boundaries, public API changes, data migrations, publication rules, and license changes require explicit review of their consequences. If there is disagreement, document the alternatives and evidence; Innostium Oy's designated maintainer makes and records the project decision. Private credentials and vulnerability details do not belong in public deliberations.

## Maintainer responsibilities

Maintainers keep scope and implementation claims accurate, triage contributions, enforce the code of conduct, manage private reports, and authorize releases. A maintainer may remove harmful content or restrict participation with an explanation where sharing it would not expose private information. Conduct appeals go to another designated Innostium Oy representative when available.

The GitHub repository owner, maintainer handles, and private conduct contact have not been configured. Record verified identities and a monitored contact before public launch, then add real CODEOWNERS entries. This repository deliberately contains no invented GitHub organization, team, email address, or ownership rule.

## Releases

Repository publication, package releases, infrastructure deployment, and app publication are separate actions. Follow [Releasing](docs/RELEASING.md). Public release notes must state the implemented subset and material limitations. Existing copyright notices and the Apache-2.0 license remain in distributions.
