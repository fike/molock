# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability, please report it responsibly:

1. **Do NOT** create a public GitHub issue for security vulnerabilities
2. Email the maintainers directly with details
3. Include steps to reproduce, potential impact, and suggested fixes
4. Allow time for a response before disclosing publicly

We aim to acknowledge reports within 48 hours and provide a timeline for fixes.

---

# Security Best Practices for Code

This document outlines security best practices that all contributors must follow when writing code for this project.

## 1. Never Commit Secrets, Tokens, or Passwords

**This is the most critical rule.**

- :x: **NEVER** commit API keys, tokens, passwords, or secrets to version control
- :x: **NEVER** commit private keys, certificates, or credentials
- :x: **NEVER** commit database connection strings with passwords
- :white_check_mark: Use `.env` files (already gitignored) for local development
- :white_check_mark: Use environment variables for sensitive configuration
- :white_check_mark: Use `.env.example` to document required environment variables (without real values)

### How to Prevent Accidental Secret Commits

```bash
# Install git-secrets to prevent commits containing secrets
brew install git-secrets
git secrets --install
git secrets --register-aws

# Or use truffleHog to scan for exposed secrets
cargo install trufflehog
trufflehog filesystem .
```

### If You Accidentally Exposed a Secret

1. **Immediately rotate the exposed credential**
2. Revoke the token/key and generate a new one
3. Remove the secret from git history:
   ```bash
   git filter-branch --force --index-filter \
     'git rm --cached --ignore-unmatch <file-with-secret>' \
     --prune-empty --tag-name-filter cat -- --all
   ```
4. Force push (with team coordination)

---

## 2. Input Validation & Sanitization

- Validate all input data at API boundaries
- Use type-safe parsing (Rust's type system helps here)
- Reject invalid input early with clear error messages
- Sanitize data before using in:
  - Database queries (use parameterized queries)
  - Command execution (avoid shell commands when possible)
  - Log output (prevent log injection)

**Example:**
```rust
// Bad
let user_input = req.param("id");
let query = format!("SELECT * FROM users WHERE id = {}", user_input);

// Good - validate first
let user_id = req.param("id").parse::<u64>()
    .map_err(|_| BadRequest("Invalid user ID"))?;
```

---

## 3. Authentication & Authorization

- Implement proper authentication for protected endpoints
- Use secure session management
- Enforce authorization checks on every protected operation
- Follow principle of least privilege

---

## 4. SQL & Command Injection Prevention

- **Always use parameterized queries** - never build SQL from user input
- Avoid `format!` or string concatenation for queries
- Use an ORM or query builder that handles escaping
- If shell commands are necessary, use `std::process::Command` with explicit arguments (not shell expansion)

**Example:**
```rust
// Bad
let query = format!("SELECT * FROM users WHERE name = '{}'", username);

// Good - use parameterized queries
let query = "SELECT * FROM users WHERE name = $1";
let result = client.query(query, &[&username]).await?;
```

---

## 5. Rate Limiting & DoS Protection

- Implement rate limiting for API endpoints
- Set appropriate timeouts on network operations
- Limit request body sizes (already configured via `max_request_size`)
- Use connection pooling to prevent resource exhaustion

---

## 6. Secure Logging

- **Never log secrets, tokens, or passwords**
- **Never log PII (Personally Identifiable Information)** without consent
- Sanitize log output to prevent log injection attacks
- Use structured logging for better analysis

**Example:**
```rust
// Bad
info!("User login: password={}", password);

// Good
info!("User login attempt: user_id={}", user_id);
```

---

## 7. TLS/HTTPS Enforcement

- Always use HTTPS in production
- Use strong TLS configurations (TLS 1.2+ minimum)
- Set secure cookie flags (`Secure`, `HttpOnly`, `SameSite`)
- Implement HSTS (HTTP Strict Transport Security)

---

## 8. Dependency Security

### Regular Vulnerability Scanning

```bash
# Run cargo-audit to check for known vulnerabilities
cargo install cargo-audit
cargo audit

# Check for dependencies with known vulnerabilities
cargo deny check advisories
```

### Dependency Best Practices

- Review dependencies before adding them
- Prefer well-maintained, popular crates
- Keep dependencies updated
- Minimize the number of dependencies

---

## 9. Rust-Specific Security

### Memory Safety

Rust's ownership system prevents many memory safety issues, but be careful with:

- **Unsafe code** - minimize its use, isolate it, and document safety invariants
- **Raw pointers** - avoid unless absolutely necessary
- **FFI** - validate all data crossing FFI boundaries

### Concurrency Safety

- Use proper synchronization primitives (`Mutex`, `RwLock`, channels)
- Avoid deadlocks by consistent lock ordering
- Be aware of data races in concurrent code

---

## 10. Error Handling & Information Leakage

- Return appropriate HTTP status codes (don't expose internal details)
- Don't leak stack traces or internal file paths in production errors
- Log detailed errors internally, return generic messages to users
- Handle errors gracefully - don't let the application crash unexpectedly

**Example:**
```rust
// Bad - leaks internal details
Err(anyhow!("Database connection failed: {}", internal_error))

// Good - logs details internally, returns safe message to user
tracing::error!("Database error: {}", internal_error);
Err(anyhow!("Internal server error"))
```

---

## 11. Security Code Review Checklist

Before submitting a PR, verify:

- [ ] No secrets, tokens, or passwords in code
- [ ] All inputs are validated
- [ ] No SQL injection vulnerabilities
- [ ] No command injection vulnerabilities  
- [ ] Sensitive data not logged
- [ ] Proper authentication/authorization in place
- [ ] Rate limiting implemented for public endpoints
- [ ] Dependencies are secure (run `cargo audit`)
- [ ] Error messages don't leak sensitive information

---

## 12. Security Tools

Recommended tools for this project:

| Tool | Purpose |
|------|---------|
| `cargo-audit` | Scan for vulnerable dependencies |
| `cargo-deny` | Enforce dependency policies |
| `git-secrets` | Prevent secret commits |
| `trufflehog` | Scan for exposed secrets |
| `clippy` | Lint for common issues |

Run security checks before committing:
```bash
cargo audit
cargo deny check advisories
cargo clippy -- -D warnings
```

---

## Incident Response

If a security vulnerability is discovered:

1. **Report** - Contact maintainers privately
2. **Assess** - Evaluate severity and impact
3. **Fix** - Develop and test the fix
4. **Release** - Publish fix with security advisory
5. **Communicate** - Notify users and provide upgrade path

We follow a 90-day disclosure timeline.

---

*Last updated: 2026-02-21*
