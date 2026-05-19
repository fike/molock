# Architectural Decision Records (ADRs)

This directory contains a collection of Architectural Decision Records (ADRs) for the Molock project.

## What is an ADR?

An Architectural Decision Record (ADR) is a document that captures an important architectural decision made along with its context and consequences. We follow the [MADR (Markdown Architectural Decision Records)](https://adr.github.io/madr/) format.

## Why use ADRs?

- **Institutional Memory**: Capture the "why" behind decisions for future maintainers.
- **Alignment**: Ensure the team and community understand the technical direction.
- **Onboarding**: Help new contributors understand the system's evolution.
- **Transparency**: Make the governance and decision-making process open.

## Process

1.  **Draft**: Create a new Markdown file in this directory following the template below.
2.  **Numbering**: Use a sequential four-digit number (e.g., `0001-description.md`).
3.  **Review**: Submit the ADR as part of a Pull Request for review by maintainers.
4.  **Acceptance**: Once the PR is merged, the decision is considered final.

## Template (MADR)

```markdown
# [Title of the Decision]

*   Status: [proposed | accepted | superseded by ADR-XXXX | deprecated]
*   Date: [YYYY-MM-DD]

## Context and Problem Statement

[Describe the context and the specific problem we are trying to solve.]

## Decision Drivers

- [Driver 1]
- [Driver 2]

## Considered Options

- [Option 1]
- [Option 2]

## Decision Outcome

Chosen option: [Option X], because [justification].

### Consequences

- **Good**: [Positive outcome]
- **Bad**: [Trade-off or negative consequence]
```

## Index

*   [0000-use-adrs-for-technical-documentation.md](0000-use-adrs-for-technical-documentation.md)
*   [0001-use-actix-web-for-high-performance-mocking.md](0001-use-actix-web-for-high-performance-mocking.md)
