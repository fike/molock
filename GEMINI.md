# Molock Project Guidelines

This file contains the foundational mandates and project context for all AI agents working on Molock.

## ⛔ CRITICAL MANDATE: BRANCH PROTOCOL
- **NEVER COMMIT DIRECTLY TO `main`**: Any direct commit or push to the `main` branch is a violation of core project integrity.
- **ALWAYS USE FEATURE BRANCHES**: Every change, no matter how small, MUST be developed on a feature branch (`feature/` or `fix/`) and merged via a Pull Request.
- **CHECK BRANCH BEFORE COMMIT**: Always run `git branch --show-current` before committing to ensure you are NOT on `main`.

## Core Mandates

- **Mandatory Workflow**: All agents MUST strictly follow the tracking and workflow rules defined in `AGENTS.md`. This includes using `bd` (beads) for issue tracking and adhering to the session completion protocol.
