# Extreme Programming (XP) Practices

## Overview
This guide defines how Extreme Programming (XP) principles apply to AI-assisted and human development in the Molock project. Our focus is on maintaining high momentum without accumulating technical debt.

## Core XP Principles

### 1. Simple Design & YAGNI (You Aren't Gonna Need It)
- **Rule:** Write code to solve today's problems, not tomorrow's hypothetical problems.
- **AI Behavior:** When proposing solutions, choose the simplest, most direct implementation. Avoid introducing generic traits, complex class hierarchies, or "just-in-case" configuration options unless explicitly required by the current test case.
- **Refactoring:** Wait until you have three instances of a pattern before abstracting it (Rule of Three).

### 2. Continuous Integration (CI)
- **Rule:** Code must always be in a deployable state.
- **AI Behavior:** Before confirming a task is finished, the AI must ensure that all tests pass (`make test`) and the project compiles (`make build`). The complete validation pipeline (`make pre-push`) should be run before preparing a commit.

### 3. AI Pair Programming
- **Rule:** Treat the AI as a pairing partner.
- **Roles:**
  - **AI as Driver:** The user explains the conceptual goal, and the AI writes the TDD tests and implementation.
  - **AI as Navigator:** The user writes the code, and the AI reviews it for adherence to Simple Design, TDD compliance, and Rust idioms.
- **Feedback:** The AI should actively push back against requests that violate TDD or YAGNI principles, explaining why a simpler approach is preferred.

### 4. Relentless Refactoring
- **Rule:** Code is never "done." It must be continuously improved.
- **AI Behavior:** During the *Refactor* phase of the TDD cycle, look for opportunities to simplify names, extract methods to reduce cognitive load, and eliminate code duplication. Ensure no tests are broken during this phase.

## Anti-Patterns to Avoid
- **Over-Abstraction:** Creating generic handlers or wrappers for a single specific use case.
- **Premature Optimization:** Trying to optimize for CPU/Memory before profiling identifies a bottleneck (unless it clearly violates Molock's high-performance mandate).
- **Skipping Tests:** Writing implementation logic and then "backfilling" tests to hit coverage metrics. Test-first is non-negotiable.