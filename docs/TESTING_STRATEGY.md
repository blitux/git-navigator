# Git Navigator Testing Strategy

This document outlines the comprehensive testing approach for git-navigator, including the separation of unit and integration tests for optimal development workflow.

## Test Organization Overview

### Test Categories

1. **Pure Unit Tests** - Fast, isolated, use mocks
2. **Integration Tests** - Slower, use real git repositories  
3. **End-to-End Tests** - Full CLI testing with real scenarios

### Directory Structure

```
tests/
├── common/                     # Shared test utilities
│   ├── mod.rs                  # Public API exports
│   ├── repository.rs           # Real git repository utilities
│   ├── mock.rs                 # Mock git operations
│   ├── persistent_repos.rs     # Persistent test repositories
│   ├── assertions.rs           # Test assertion helpers
│   └── fixtures.rs             # Test data generators
├── unit_tests/                 # Fast unit tests (planned)
├── integration_tests/          # Integration tests (planned)
└── *.rs                        # Current integration tests
src/
└── */tests/                    # Inline unit tests (current)
test-repo/                      # Persistent test repositories
├── basic/                      # Simple repository scenarios
├── mixed/                      # Complex repository states
└── ...
```

## Testing Approaches

### 1. Pure Unit Tests (Fast - ~1ms per test)

**Use mocks for:**
- Business logic validation
- Error handling paths
- File status processing
- Index parsing and validation
- Template rendering
- Color formatting

**Example:**
```rust
use common::{scenarios, TestGitContext};

#[test]
fn test_status_processing_logic() -> anyhow::Result<()> {
    // Mock repository with known state
    let mock_repo = scenarios::mixed_status_repo("/tmp/mock");
    let context = TestGitContext::mock(mock_repo);
    
    let files = context.get_status()?;
    
    // Test business logic without filesystem
    assert_eq!(files.len(), 4);
    assert!(files.iter().any(|f| f.staged));
    
    Ok(())
}
```

### 2. Integration Tests (Medium - ~50ms per test)

**Use real repositories for:**
- Git operations integration
- File system interactions
- Repository state changes
- Cross-platform compatibility

**Example:**
```rust
use common::{persistent_repos, TestGitContext};

#[test]
fn test_real_git_operations() -> anyhow::Result<()> {
    // Real git repository with known state
    let repo = persistent_repos::scenarios::mixed()?;
    let context = TestGitContext::real(repo.path())?;
    
    let files = context.get_status()?;
    
    // Test real git integration
    assert!(!files.is_empty());
    
    Ok(())
}
```

### 3. End-to-End CLI Tests (Slow - ~100ms per test)

**Use for:**
- Full command execution
- CLI argument parsing
- Output formatting
- Error message validation

**Example:**
```rust
use assert_cmd::prelude::*;
use common::persistent_repos;

#[test]
fn test_status_command_cli() -> anyhow::Result<()> {
    let repo = persistent_repos::scenarios::mixed()?;
    
    let mut cmd = Command::cargo_bin("git-navigator")?;
    cmd.arg("status")
       .current_dir(repo.path())
       .assert()
       .success()
       .stdout(predicates::str::contains("[1]"));
       
    Ok(())
}
```

## Test Infrastructure Components

### Mock Utilities (`tests/common/mock.rs`)

**GitOperations Trait:**
- Abstracts all git operations
- Enables swapping between real and mock implementations
- Consistent interface for testing

**MockGitOps:**
- In-memory git state simulation
- Configurable file states and errors
- Predictable test scenarios

**TestGitContext:**
- Unified interface for real/mock operations
- Easy switching between test modes

### Persistent Repositories (`test-repo/` + `persistent_repos.rs`)

**Repository Scenarios:**
- `basic/` - Simple, clean repository
- `staged/` - Files ready to commit
- `modified/` - Working directory changes
- `mixed/` - Complex multi-state scenario
- `untracked/` - New files not in git
- `conflicts/` - Merge conflict simulation
- `branches/` - Multi-branch repository
- `empty/` - Fresh git repository

**Benefits:**
- Consistent test data across runs
- Real git behavior validation
- Test isolation through recreation

## Performance Characteristics

| Test Type | Speed | Use Case | Repository |
|-----------|-------|----------|------------|
| Pure Unit | ~1ms | Logic validation | Mock |
| Integration | ~50ms | Git operations | Real |
| End-to-End | ~100ms | CLI behavior | Real |

## Best Practices

### Unit Test Guidelines

1. **Use mocks for business logic** - Test algorithms without I/O
2. **Focus on edge cases** - Error conditions, boundary values
3. **Keep tests fast** - No filesystem or network operations
4. **Test single concerns** - One logical unit per test

### Integration Test Guidelines

1. **Use persistent repos** - Consistent, predictable states
2. **Test real git behavior** - Verify actual git integration
3. **Cover happy paths** - Normal operation scenarios
4. **Validate state changes** - Before/after comparisons

### Test Organization

1. **Separate concerns clearly** - Unit vs integration vs E2E
2. **Use descriptive names** - `test_status_parsing_with_staged_files`
3. **Group related tests** - Module organization
4. **Document complex scenarios** - Why this test exists

## Current Status

### ✅ Implemented

- Consolidated test utilities
- Mock git operations framework
- Persistent test repositories
- Trait-based abstraction
- Performance infrastructure

### 📋 Recommended Next Steps

1. **Migrate existing tests** - Use new infrastructure
2. **Create unit test suite** - Fast development feedback
3. **Optimize CI pipeline** - Parallel test execution
4. **Add performance benchmarks** - Track test speed regression

## Usage Examples

### Development Workflow

```bash
# Fast unit tests for development
cargo test --lib unit_tests

# Integration tests for validation
cargo test --test integration_tests

# Full test suite
cargo test

# Specific scenario testing
cargo test mixed_status
```

### Test Infrastructure Usage

```rust
mod common;
use common::{scenarios, persistent_repos, TestGitContext};

// Fast unit test
let mock = scenarios::mixed_status_repo("/tmp/mock");
let context = TestGitContext::mock(mock);

// Integration test
let repo = persistent_repos::scenarios::mixed()?;
let context = TestGitContext::real(repo.path())?;
```

This testing strategy ensures comprehensive coverage while maintaining fast development feedback loops through proper test categorization and infrastructure.