# Use ADRs for Technical Documentation

*   Status: accepted
*   Date: 2026-05-19

## Context and Problem Statement

As the Molock project grows, complex technical decisions are made regarding architecture, performance optimizations, and library choices. Without a formal way to record these decisions, the context and rationale behind them are lost over time, making it difficult for new contributors to understand the system and leading to repetitive discussions.

## Decision Drivers

- Maintain institutional memory.
- Provide transparency for the open-source community.
- Accelerate onboarding for new developers.
- Ensure architectural consistency.

## Considered Options

- **Option 1: README/Wiki pages**: Informal documentation scattered across READMEs.
- **Option 2: Git Commit Messages**: Rationale buried in the git history.
- **Option 3: Architectural Decision Records (ADRs)**: Standardized, version-controlled records in the repository.

## Decision Outcome

Chosen option: **Option 3: Architectural Decision Records (ADRs)**, because it provides a centralized, structured, and searchable history of technical decisions that lives alongside the code. We adopted the MADR format for its balance of simplicity and structure.

### Consequences

- **Good**: Clear historical context for technical choices.
- **Good**: Standardized format for proposals and reviews.
- **Bad**: Requires discipline from maintainers to document significant changes.
