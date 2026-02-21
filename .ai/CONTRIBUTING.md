# Contributing to Molock

Thank you for your interest in contributing to Molock! This document provides guidelines for both human contributors and AI agents.

## Code of Conduct

Please be respectful and constructive in all interactions. We aim to create a welcoming environment for everyone.

## Getting Started

### Prerequisites
- Rust 1.70 or later
- Docker and Docker Compose (for deployment)
- Git

### Development Environment Setup
1. Clone the repository:
   ```bash
   git clone https://github.com/your-org/molock.git
   cd molock
   ```

2. Install dependencies:
   ```bash
   cargo build
   ```

3. Set up environment:
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

4. Run tests:
   ```bash
   make test
   ```

## Development Workflow

### For Human Contributors
1. Create a feature branch:
   ```bash
   git checkout -b feat/your-feature-name
   ```

2. Make your changes following the coding conventions.

3. Run tests and checks:
   ```bash
   make test
   make lint
   make fmt
   ```

4. Commit your changes with a descriptive message.

5. Push your branch and create a pull request.

### For AI Agents
1. Follow the guidelines in `.agent/Agents.md`
2. Use the Makefile for common tasks
3. Ensure all tests pass before completing work
4. Document your changes appropriately

## Reporting Issues

### Bug Reports
When reporting a bug, please include:
- Clear description of the issue
- Steps to reproduce
- Expected behavior
- Actual behavior
- Environment details (OS, Rust version, etc.)
- Relevant logs or error messages

### Feature Requests
When requesting a feature, please include:
- Use case and motivation
- Proposed implementation approach
- Any alternatives considered
- Impact on existing functionality

## Pull Request Process

### Checklist for Submitting PRs
- [ ] Code follows project conventions
- [ ] Tests added or updated
- [ ] Documentation updated
- [ ] All checks pass (CI)
- [ ] Commit messages follow conventions
- [ ] Changes are focused and minimal

### Review Process
1. PR is assigned to reviewers
2. Automated checks run (tests, linting, coverage)
3. Reviewers provide feedback
4. Address feedback and update PR
5. PR is merged after approval

## Test-Driven Development Requirements

### TDD Mandate
**All contributions MUST follow Test-Driven Development (TDD) principles:**
1. **Write tests first** before implementing any new functionality
2. **Fix bugs** by first writing a test that reproduces the bug
3. **Refactor** only when you have comprehensive test coverage
4. **Reject** any PR that doesn't include adequate tests

### Coverage Standards (Enforced)
- **MINIMUM**: 80% line coverage for all modified code
- **MINIMUM**: 80% branch coverage for critical paths  
- **REQUIRED**: All public APIs must have comprehensive tests
- **REQUIRED**: Integration tests for all major features
- **PENALTY**: PRs with <80% coverage will be automatically rejected

### TDD Workflow for Contributors
```bash
# 1. FIRST: Write failing tests for new feature/bug
cargo test --lib  # Should show failing tests

# 2. THEN: Implement minimal code to make tests pass
# ... write implementation code ...

# 3. VERIFY: Tests now pass
cargo test --lib  # All tests should pass

# 4. CHECK: Coverage meets requirements
make test-coverage  # Must show >=80% coverage

# 5. REFACTOR: Improve code while keeping tests green
# ... refactor implementation ...

# 6. FINAL VERIFICATION: All tests still pass
make test
```

### Test Gap Analysis
When working on existing code:
```bash
# Find untested functions in module
grep -n "fn [a-z_]" src/path/to/module.rs | grep -v "test_"

# Check coverage for specific file
cargo tarpaulin --lib --output-dir ./coverage --src src/path/to/module.rs

# Add tests for any untested functions before making changes
```

### Running Tests
```bash
# Run all tests
make test

# Run with coverage (required before PR submission)
make test-coverage

# Run specific test
cargo test test_name

# Run unit tests only
cargo test --lib

# Run integration tests only  
cargo test --tests
```

## Documentation

### Code Documentation
- Document all public items
- Include examples for complex functions
- Update documentation when changing APIs
- Use markdown in doc comments

### Project Documentation
- Keep README.md up to date
- Update `.agent/` documentation as needed
- Document breaking changes
- Include migration guides

## Release Process

### Versioning
We follow [Semantic Versioning](https://semver.org/):
- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Release Checklist
- [ ] All tests pass
- [ ] Documentation updated
- [ ] Changelog updated
- [ ] Version bumped in Cargo.toml
- [ ] Docker images built and tagged
- [ ] Release notes prepared

## Security

### Reporting Security Issues
Please report security issues privately to the maintainers. Do not disclose publicly until fixed.

### Security Best Practices
- Never commit secrets or credentials
- Validate all user input
- Use secure defaults
- Keep dependencies updated
- Follow principle of least privilege

## Performance

### Benchmarking
- Include benchmarks for performance-critical code
- Run benchmarks before and after changes
- Monitor memory usage and latency

### Optimization Guidelines
- Profile before optimizing
- Focus on bottlenecks
- Consider trade-offs (memory vs CPU)
- Document performance characteristics

## Maintenance

### Dependency Updates
- Regularly update dependencies
- Test thoroughly after updates
- Monitor for security advisories
- Consider breaking changes

### Code Quality
- Regular code reviews
- Address technical debt
- Refactor when needed
- Maintain test coverage

## Getting Help

### Resources
- Check `.agent/Skills.md` for technical guidance
- Review existing code for patterns
- Consult Rust documentation
- Ask in discussions or issues

### Questions
For questions about:
- **Usage**: Check documentation and examples
- **Development**: Review `.agent/Agents.md`
- **Architecture**: Check design documents
- **Specific issues**: Open a discussion

## Recognition

Contributors will be acknowledged in:
- Release notes
- Contributor list
- Documentation credits

We appreciate all contributions, whether code, documentation, bug reports, or feature suggestions!

## License

By contributing, you agree that your contributions will be licensed under the project's MIT OR Apache-2.0 license.