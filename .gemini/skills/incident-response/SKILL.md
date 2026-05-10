---
name: incident-response
description: Structured debugging and incident resolution using the OODA Loop (Observe, Orient, Decide, Act). Use this skill when investigating failing tests, production-like bugs, performance regressions, or mysterious system behavior.
---

# Incident Response Skill (OODA Loop)

This skill guides the AI and the developer through a structured cognitive process to resolve complex issues without jumping to premature conclusions.

## The OODA Workflow

### 1. Observe (Gather the Data)
Before forming a hypothesis, gather raw data. Do not look for "causes" yet; look for "signals".
- **Action**: Run `scripts/fetch_latest_logs.sh` to see recent errors.
- **Action**: Check `target/` for test outputs.
- **Action**: Read the exact error message or stack trace multiple times.
- **Constraint**: Do not modify any code in this phase.

### 2. Orient (Contextualize & Analyze)
Filter the observations through your understanding of the Molock architecture.
- **Action**: Locate the affected component (e.g., `src/rules/`, `src/telemetry/`).
- **Action**: Check `GEMINI.md` for related architectural mandates.
- **Action**: Formulate a hypothesis of the root cause.
- **Question**: "What changed since it last worked?"

### 3. Decide (Formulate a Plan)
Choose a course of action.
- **Action**: Draft a focused plan to fix the issue.
- **Action**: Prioritize **Mitigation** (stop the bleeding) if it's a critical failure, followed by **Resolution** (the permanent fix).
- **Mandate**: Present the plan to the user for approval.

### 4. Act (Execute & Verify)
Implement the decision and immediately loop back to Observe.
- **Action**: Apply the code change or configuration update.
- **Action**: Run the relevant tests (`make test`).
- **Validation**: If the tests pass, **Observe** the system logs again to ensure no new errors were introduced.
- **Failure**: If the fix fails, do not "patch the patch". Restart the OODA loop from **Observe**.

## Supporting Tools
- `scripts/fetch_latest_logs.sh`: Quick extraction of error signals from the workspace.
