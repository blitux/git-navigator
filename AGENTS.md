# Git Navigator - Agent Configuration

## Project Overview

Git Navigator is a lightweight Git navigation tool written in Go, inspired by SCM Breeze's workflow.

## Commands

```bash
gs   # git-navigator status  - Show numbered git status
ga   # git-navigator add     - Add files by index
gd   # git-navigator diff    - Show diff for indexed files
grs  # git-navigator reset   - Reset files by index
gco  # git-navigator checkout - Checkout files or branches
gb   # git-navigator branches - List numbered branches
```

## Project Structure

```
cmd/git-navigator/main.go      # Entry point with cobra
internal/
├── core/
│   ├── state.go          # GitStatus, FileEntry, BranchEntry, StateCache
│   ├── error.go         # Domain-specific errors
│   ├── index_parser.go  # Flexible index parsing
│   ├── output.go        # Colored output utilities
│   └── git.go           # Git operations via exec
└── commands/
    ├── status.go
    ├── add.go
    ├── diff.go
    ├── reset.go
    ├── checkout.go
    └── branches.go
```

## Build & Test

```bash
go build ./cmd/git-navigator
go test ./...
```

## Dependencies

- `github.com/spf13/cobra` - CLI framework
- `github.com/fatih/color` - Colored output

## Cache

State cache stored at `$XDG_CACHE_HOME/git-navigator/<hash>/files.json`

## Index Parsing

Supports formats: `1`, `1-3`, `1,3,5`, `1 3-5,8`
