---
name: code-quality
description: "Focuses on Clean Code, Refactoring, and Rust idiomatic patterns. Use this skill when refactoring code, improving readability, or ensuring idiomatic Rust standards."
---

# Code Quality Skill

This skill provides guidance and tools for maintaining high code quality in the Molock project, drawing from Clean Code and Refactoring best practices adapted for Rust.

## Core Workflows

### 1. Identify Code Smells
Before refactoring, identify specific "smells" (e.g., long functions, large structs, excessive cloning).
- **Tool**: Use `scripts/find_code_smells.sh` to run `clippy --pedantic` and identify potential improvements.

### 2. Safe Refactoring
Refactoring must never change the observable behavior of the code.
- **Protocol**:
  1. Run `make test` to ensure a green state.
  2. Apply a single, small refactoring (e.g., Extract Method, Rename Variable).
  3. Run `make test` again to ensure it remains green.
  4. Repeat.

### 3. Apply Rust Idioms
Ensure the code follows idiomatic Rust patterns to improve safety and performance.
- **Reference**: See [references/rust-idioms.md](references/rust-idioms.md) for specific patterns like avoiding `.clone()`, using `anyhow`/`thiserror`, and leveraging the Trait system.

## Principles
- **Clarity over Cleverness**: Write code that is easy to read and understand.
- **Small Functions**: Functions should do one thing and do it well.
- **Descriptive Naming**: Variable and function names should reveal their intent.
- **DRY (Don't Repeat Yourself)**: Eliminate duplication while avoiding premature abstraction.

## Supporting Tools
- `scripts/find_code_smells.sh`: Runs clippy with pedantic checks to find quality issues.