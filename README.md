# Git Navigator

A lightweight and efficient Git navigation tool written in Go.

## Installation

```bash
go install github.com/blitux/git-navigator@latest
```

Or build from source:

```bash
git clone https://github.com/blitux/git-navigator.git
cd git-navigator
go build -o git-navigator ./cmd/git-navigator
```

## Shell Aliases

Add to your `.bashrc`, `.zshrc`, or `.fishrc`:

```bash
alias gs='git-navigator status'
alias ga='git-navigator add'
alias gd='git-navigator diff'
alias grs='git-navigator reset'
alias gco='git-navigator checkout'
alias gb='git-navigator branches'
```

## Commands

### Status (`gs`)

Show numbered git status grouped by staged/unstaged/untracked:

```bash
git-navigator status
# or
gs
```

### Add (`ga`)

Add files by index (supports ranges):

```bash
git-navigator add 1 2 3
git-navigator add 1-5
git-navigator add 1,3,5
```

### Diff (`gd`)

Show diff for indexed files:

```bash
git-navigator diff 1 2
```

### Reset (`grs`)

Reset files by index:

```bash
git-navigator reset 1 2
```

### Checkout (`gco`)

Checkout files or branches:

```bash
git-navigator checkout 1
git-navigator checkout -b new-branch
```

### Branches (`gb`)

List numbered branches:

```bash
git-navigator branches
git-navigator branches --remote
```

## Features

- **Numbered output**: All files and branches are numbered for quick reference
- **Flexible index parsing**: Supports `1`, `1-3`, `1,3,5`, `1 3-5,8` formats
- **Colored output**: Clear visual distinction between file statuses
- **State caching**: Persists state to avoid re-running git commands

## Cache Location

State cache stored at `$XDG_CACHE_HOME/git-navigator/<hash>/files.json`

## Dependencies

- Go 1.21+
- spf13/cobra
- fatih/color

## License

MIT
