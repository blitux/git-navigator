# Test Repository Directory

This directory contains persistent test repositories used for integration testing.
These repositories are automatically created and managed by the test suite.

## Repository Scenarios

Each subdirectory represents a different git repository state:

- **`basic/`** - Simple repository with 3 files, 2 commits, clean working directory
- **`staged/`** - Repository with files in staging area ready to commit  
- **`modified/`** - Repository with unstaged modifications to tracked files
- **`mixed/`** - Complex scenario with staged, unstaged, untracked, and deleted files
- **`untracked/`** - Repository with multiple untracked files and directories
- **`conflicts/`** - Repository with simulated merge conflict markers
- **`branches/`** - Repository with multiple branches and history
- **`empty/`** - Freshly initialized empty repository

## Usage

The repositories are managed automatically by the test suite through the
`PersistentRepoManager` in `tests/common/persistent_repos.rs`.

```rust
use common::persistent_repos::scenarios;

// Get or create a test repository
let repo = scenarios::mixed()?;
// Use repo.path() for git operations
```

## Maintenance

- Repositories are recreated automatically when tests need them
- All repositories are ignored by git (added to .gitignore)
- Use `scenarios::clean_all()` to remove all test repositories
- Individual repositories can be cleaned with `repo.clean()`

## Notes

- These repositories provide consistent, predictable states for integration testing
- They complement the mock utilities for comprehensive test coverage
- Each repository is recreated on demand to ensure test isolation