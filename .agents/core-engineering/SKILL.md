---
name: core-engineering
description: "Specialized engineering workflows for Molock. Use this skill for TDD, XP (Extreme Programming) enforcement, code style compliance, maintenance scripts, and preparing pull requests."
---

# Core Engineering Skill

This skill provides the procedural workflows and specialized tools for developing and maintaining the Molock project.

## Development Workflows

### Makefile Commands
Use the following `make` commands to manage the lifecycle:
- `make build`: Build the project.
- `make test`: Run unit tests.
- `make lint`: Run clippy and check formatting.
- `make test-coverage`: Run tarpaulin and verify >80% coverage.
- `make pre-push`: Run the full validation pipeline (MANDATORY before pushing).

### Commit Message Convention
Follow Conventional Commits:
- `feat(scope): ...`
- `fix(scope): ...`
- `docs(scope): ...`
- `test(scope): ...`

## Extreme Programming (XP) & TDD Enforcement
Test-Driven Development (TDD) and XP principles are mandatory in this repository.
1. **Red**: Write a failing test first.
2. **Green**: Implement minimal code to pass.
3. **Refactor**: Clean up while keeping tests green.
4. **Simple Design (YAGNI)**: Never implement features you "might" need later.
5. **Continuous Integration**: Validate locally with `make pre-push` before any commit.

For detailed steps and legacy code handling, see [references/tdd-guide.md](references/tdd-guide.md).
For rules on avoiding over-engineering and applying XP principles, see [references/xp-practices.md](references/xp-practices.md).

## Maintenance Scripts
Execute these via the shell as needed:
- `scripts/add_license_headers.sh`: Ensures all files have required license headers.
- `scripts/find_untested_functions.sh`: Identifies functions lacking unit tests.

## Quality Standards
- **Security**: Follow the checklists in [references/security.md](references/security.md).
- **Contributions**: Adhere to the PR guidelines in [references/contributing.md](references/contributing.md).
- **Line Length**: Max 100 characters.
- **Indentation**: 4 spaces.
