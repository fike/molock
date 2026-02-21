# Test-Driven Development (TDD) Enforcement Guide

## Overview
This guide provides concrete steps for AI agents to enforce TDD practices in the Molock project. All contributions MUST follow these guidelines.

## Core TDD Principles

### 1. Test-First Development
- **NEVER** write production code without a failing test
- **ALWAYS** write tests that describe desired behavior
- **ONLY** write enough production code to make tests pass

### 2. Red-Green-Refactor Cycle
```
RED: Write a failing test
GREEN: Write minimal code to make test pass  
REFACTOR: Improve code while tests stay green
```

### 3. Coverage Enforcement
- **MINIMUM**: 80% line coverage for all modified code
- **TARGET**: 90%+ coverage for new features
- **REJECTION**: PRs with <80% coverage will be rejected

## Practical TDD Workflow

### Step 1: Analyze Current Test Coverage
```bash
# Check overall coverage
make test-coverage

# Check specific module coverage
cargo tarpaulin --lib --src src/path/to/module.rs

# Find untested functions in a module
grep -n "^\s*pub fn\|^\s*fn [a-z_]" src/path/to/module.rs | \
  grep -v "test_" | \
  grep -v "#\[test\]" | \
  grep -v "#\[cfg(test)\]"
```

### Step 2: Write Failing Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_new_feature_behavior() {
        // Arrange
        let input = "test_input";
        
        // Act
        let result = new_feature(input);
        
        // Assert
        assert_eq!(result, expected_output);
    }
    
    #[test]
    #[should_panic(expected = "error message")]
    fn test_new_feature_error_case() {
        // Test error conditions
        new_feature("invalid_input");
    }
}
```

### Step 3: Implement Minimal Solution
```rust
pub fn new_feature(input: &str) -> String {
    // Minimal implementation to make tests pass
    if input == "invalid_input" {
        panic!("error message");
    }
    "expected_output".to_string()
}
```

### Step 4: Refactor with Confidence
```rust
pub fn new_feature(input: &str) -> Result<String, Error> {
    // Refactored implementation
    validate_input(input)?;
    process_input(input)
}
```

## Automatic Test Gap Detection

### Script to Find Untested Functions
```bash
#!/bin/bash
# find_untested_functions.sh
MODULE=$1

echo "Checking for untested functions in $MODULE..."
echo "=============================================="

# Find all functions (excluding tests)
grep -n "^\s*pub fn\|^\s*fn [a-z_]" "$MODULE" | \
  grep -v "test_" | \
  grep -v "#\[test\]" | \
  grep -v "#\[cfg(test)\]" | \
  while read -r line; do
    echo "UNTESTED: $line"
  done

echo "=============================================="
echo "Run: cargo tarpaulin --lib --src $MODULE for coverage details"
```

### Common Test Patterns

#### 1. Unit Test Template
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_function_name() {
        // Arrange - setup test data
        let input = "test";
        
        // Act - call the function
        let result = function_name(input);
        
        // Assert - verify results
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_value);
    }
    
    #[test]
    fn test_function_name_error_case() {
        // Test error handling
        let result = function_name("invalid");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "expected error");
    }
    
    #[test]
    #[should_panic(expected = "panic message")]
    fn test_function_name_panic() {
        // Test panic conditions
        function_name("panic_input");
    }
}
```

#### 2. Async Test Template
```rust
#[tokio::test]
async fn test_async_function() {
    let result = async_function().await;
    assert_eq!(result, expected_value);
}
```

#### 3. Integration Test Template
```rust
// In tests/integration_test.rs
#[actix_web::test]
async fn test_endpoint() {
    let app = test::init_service(App::new().route("/test", web::get().to(handler))).await;
    
    let req = test::TestRequest::get().uri("/test").to_request();
    let resp = test::call_service(&app, req).await;
    
    assert_eq!(resp.status(), 200);
}
```

## TDD Enforcement Commands

### Required Checks Before Commit
```bash
# 1. Run all tests
cargo test --lib

# 2. Check formatting
cargo fmt --check

# 3. Check linting
cargo clippy -- -D warnings

# 4. Check coverage (must be >=80%)
make test-coverage

# 5. Verify no untested functions added
./.agent/scripts/check_test_coverage.sh src/path/to/modified_file.rs
```

### Coverage Analysis Commands
```bash
# Detailed coverage report
cargo tarpaulin --lib --out Html

# Coverage for specific files
cargo tarpaulin --lib --src src/telemetry/mod.rs src/telemetry/metrics.rs

# Line-by-line coverage
cargo tarpaulin --lib --line
```

## Handling Legacy Code

### Protocol for Untested Legacy Code
1. **DO NOT MODIFY** the legacy code directly
2. **First write tests** that capture current behavior
3. **Run tests** to ensure they pass with existing code
4. **Only then** make changes or improvements
5. **Verify** tests still pass after changes

### Example: Adding Tests to Legacy Function
```rust
// Legacy code (untested)
pub fn legacy_function(x: i32) -> i32 {
    x * 2
}

// Add tests FIRST
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_legacy_function_basic() {
        assert_eq!(legacy_function(2), 4);
        assert_eq!(legacy_function(0), 0);
        assert_eq!(legacy_function(-3), -6);
    }
    
    #[test]
    fn test_legacy_function_edge_cases() {
        assert_eq!(legacy_function(i32::MAX / 2), i32::MAX - 1);
    }
}

// Now you can safely refactor or modify legacy_function
```

## Common TDD Pitfalls to Avoid

### ❌ DON'T:
- Write implementation before tests
- Skip tests for "simple" functions
- Assume existing tests are sufficient
- Modify code without test coverage

### ✅ DO:
- Write failing tests first
- Test all public functions
- Check coverage before and after changes
- Refactor only when tests are green

## TDD Success Metrics

### Quantitative Metrics
- **Test Coverage**: >=80% line coverage
- **Test Count**: Tests should outnumber functions 2:1
- **Build Status**: All tests pass 100% of the time
- **CI/CD**: Automated test runs on every commit

### Qualitative Metrics
- **Confidence**: Can refactor without fear of breaking things
- **Documentation**: Tests serve as living documentation
- **Design**: TDD leads to better, more modular design
- **Maintenance**: Bugs are caught early by failing tests

## Resources

### Tools
- `cargo test` - Run tests
- `cargo tarpaulin` - Coverage analysis
- `cargo fmt` - Code formatting
- `cargo clippy` - Linting

### Documentation
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [TDD by Example](https://www.oreilly.com/library/view/test-driven-development/0321146530/)
- [Molock Testing Examples](../src/telemetry/attributes.rs) - See comprehensive test patterns

### Quick Reference
```bash
# TDD Quick Start
1. cargo test --lib                    # Check current state
2. Write failing test                  # RED
3. cargo test --lib                    # Verify test fails  
4. Implement minimal solution          # GREEN
5. cargo test --lib                    # Verify test passes
6. Refactor                            # REFACTOR
7. cargo test --lib                    # Verify still passes
8. make test-coverage                  # Check coverage >=80%
```