# Rust Idiomatic Patterns & Clean Code

## Project Legibility Standards

### 1. File Size Limits
- **Mental Lint**: Aim for a maximum of **1,000 lines of code (LOC)** per file.
- **Action**: If a file exceeds this limit, look for logical sub-concepts to move into a directory-based module (e.g., `auth.rs` becomes `auth/mod.rs`).

### 2. Function Complexity & Length
- **Cognitive Complexity**: Maintain a maximum cognitive complexity of **25** (enforced by Clippy).
- **Function Length**: Functions should ideally fit on one screen (**~40-60 lines**). If a function requires extensive scrolling to understand, it should be decomposed.
- **Parameter Count**: Functions taking more than **7 arguments** should use a configuration struct or the "Builder" pattern.

## Memory Management & Performance

### 1. Avoid Excessive Cloning
- **Smell**: Frequent calls to `.clone()` or `.to_owned()`.
- **Refactoring**: Use references (`&T`), Cow (Clone-on-Write), or restructure ownership to minimize data duplication. In Molock, performance is critical; unnecessary clones should be eliminated.

### 2. Borrow Checker as a Guide
- Use lifetime annotations only when necessary.
- Prefer composition over complex reference cycles.

## Error Handling

### 1. Library vs. Application Errors
- **Library**: Use `thiserror` for defining domain-specific error types that users of a library should handle.
- **Application**: Use `anyhow` for top-level application logic where you just need to propagate and log errors with context.

### 2. Meaningful Context
- Always use `.context()` or `.with_context()` when propagating errors with `anyhow` to provide a clear audit trail of what failed.

## Abstraction & Traits

### 1. Traits over Inheritance
- Rust does not have traditional inheritance. Use Traits to define shared behavior.
- Use `impl Trait` for return types to hide implementation details when appropriate.

### 2. Newtype Pattern
- Use the "Newtype" pattern (e.g., `struct UserId(String)`) to provide type safety for simple types and avoid "primitive obsession."

## Functional Style
- Leverage Rust's powerful iterator API (`map`, `filter`, `fold`, `collect`) for clear and concise data transformations.
- Prefer `if let` and `match` over nested `if` statements for better readability.

## Naming Conventions
- **Modules/Files**: `snake_case`
- **Structs/Enums/Traits**: `PascalCase`
- **Functions/Variables/Macros**: `snake_case`
- **Constants/Statics**: `SCREAMING_SNAKE_CASE`
- **Generics**: `UpperCamelCase` (usually single letters like `T`, `U`)