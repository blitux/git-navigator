# Git Navigator 🧭

<small>(Although name may change in the future)</small>

**A lightweight and efficient Git navigation and productivity tool inspired by SCM Breeze**

Git Navigator is a modern reimagining of SCM Breeze's core workflow, built in Rust for speed and reliability. It provides numbered file and branch navigation for Git operations, making your Git workflow faster and more intuitive. 

A huge thanks to the SCM Breeze team for their pioneering work in this space! It was a core part of my daily workflow, and this project aims to carry that legacy forward as a clean, lean and fast git productivity tool.

I may be wrong, of course. So, feel free to reach out if you have any suggestions or improvements!

**Disclaimer:** this tool has been put together by using LLM tools. I'm by no means a rust expert, so there may be some gaps. 

## ✨ Philosophy

- **🧹 Clean**: Simple, intuitive commands without bloat
- **⚡  Fast**: Rust performance with efficient Git operations and smart caching  
- **🪶 Lean**: Zero runtime dependencies, single static binary
- **🎯 Best UX**: Optimized for the 80% use case that developers need daily

## 📦 Installation

### Quick Install (Recommended)
```bash
curl -sSL https://raw.githubusercontent.com/blitux/git-navigator/main/install.sh | bash
```

This one-liner will:
- 🔍 **Auto-detect** your platform (Linux/macOS/Windows, x64/ARM64)
- 📥 **Download** the latest release binary
- 📁 **Install** to `~/.local/bin` and add to PATH
- ⚡ **Setup** shell aliases automatically

### Alternative Installation Methods

#### From Source (Development)
```bash
git clone https://github.com/git-navigator/git-navigator
cd git-navigator
./install.sh
```

#### Manual Binary Installation
1. Download the appropriate binary from [releases](https://github.com/git-navigator/git-navigator/releases)
2. Extract and move to a directory in your PATH
3. Add shell aliases manually

#### Local Development Install
```bash
# After making changes
make install-local    # Builds and installs to ~/.local/bin
```

### Shell Aliases (Auto-added by installer)
```bash
alias gs='git-navigator status'
alias ga='git-navigator add'
alias gd='git-navigator diff' 
alias grs='git-navigator reset'
alias gco='git-navigator checkout'
alias gb='git-navigator branches'
alias gcb='git-navigator checkout-branch'
alias gl="git log --graph --pretty=format:'%Cred%h%Creset -%C(yellow)%d%Creset %s %Cgreen(%cr) %C(bold blue)<%an>%Creset' --abbrev-commit"
```

### Supported Platforms
- **Linux**: x64, ARM64 (glibc and musl)
- **macOS**: Intel (x64), Apple Silicon (ARM64)  
- **Windows**: x64

### Shell Compatibility
- **Bash** (Linux/macOS/Windows)
- **Zsh** (with auto PATH and alias setup)
- **Fish** (with proper fish alias syntax)
- **Other POSIX shells**

## 🚀 Features

### Current (v0.1)
- ✅ **`gs`** - Numbered git status with colored, grouped output
- ✅ **`ga [indices]`** - Add files by index with error handling
- ✅ **`gd [indices]`** - Show diff for indexed files with color output
- ✅ **`grs [indices]`** - Reset files by index from staging area
- ✅ **`gco [indices]`** - Checkout files by index or create branches (-b flag)
- ✅ **`gb [index]`** - Numbered branch list with optional checkout
- ✅ Smart caching for improved performance
- ✅ Cross-shell compatibility (bash, zsh, fish)
- ✅ Flexible index syntax (`1`, `1-3`, `1,3,5`, `1 3-5,8`)
- ✅ Modern UI with section grouping and color-coded arrows
- ✅ Domain-specific error handling with clear user messages
- ✅ Sub-50ms startup time performance


## 🎮 Usage

### Git Status with Numbers
```bash
gs
# Output:
Branch: main
Parent: a1b2c3d Initial commit

➤ Staged:
   (new)       [1] newfile.txt
   (modified)  [2] src/main.rs

➤ Not staged:
   (modified)  [3] src/lib.rs
   (deleted)   [4] oldfile.py

➤ Untracked:
   (untracked) [5] temp.txt
```

### Adding Files by Index
```bash
# Add single files
ga 1              # Add file [1]
ga 3              # Add file [3] 

# Add multiple files  
ga 1 2 5          # Add files [1], [2], [5]
ga 1,3,5          # Add files [1], [3], [5]

# Add ranges
ga 2-4            # Add files [2], [3], [4]
ga 1-3,7          # Add files [1], [2], [3], [7]

# Mixed syntax
ga 1 3-5,8        # Add files [1], [3], [4], [5], [8]
```

### All Index Operations Available
```bash
# File operations by index
ga 1 3-5,8        # Add files [1], [3], [4], [5], [8]
gd 3              # Diff file [3]  
grs 1-3,7         # Reset files [1], [2], [3], [7]
gco 1 5           # Checkout files [1], [5]

# Branch operations
gb                # List numbered branches
gb 2              # Checkout branch [2]
gco -b new-branch # Create and switch to new branch
```

## 🏗️ Architecture

```
src/
├── commands/           # Command implementations
│   ├── status.rs       # gs command (✅ 674 lines)
│   ├── add.rs          # ga command (✅ 220 lines)
│   ├── diff.rs         # gd command (✅ 300 lines)
│   ├── reset.rs        # grs command (✅ 161 lines)
│   ├── checkout.rs     # gco command (✅ 229 lines)
│   ├── branches.rs     # gb command (✅ 447 lines)
│   └── mod.rs          # Module exports
├── core/               # Core functionality  
│   ├── colors.rs       # Unified color system with GitStatus enum
│   ├── git.rs          # Git operations via git2 library and git commands
│   ├── git_status.rs   # GitStatus enum for type safety  
│   ├── index_parser.rs # Flexible index parsing logic
│   ├── state.rs        # JSON caching and state management
│   ├── templates.rs    # Template-based output formatting
│   ├── args_parser.rs  # Centralized argument parsing
│   ├── error.rs        # Domain-specific error types
│   └── output.rs       # Output utilities
└── main.rs             # CLI entry point with clap
```

## 🧪 Development

### Quick Start with Make
```bash
# Build release binary
make build

# Run all tests  
make test

# Run the status command
make run

# Development workflow
make format lint test build
```

### Manual Commands
```bash
# Build
cargo build --release

# Test
cargo test

# Format and lint
cargo fmt
cargo clippy
```

### Available Make Targets
```bash
make help        # Show all available commands
make build       # Build release binary
make test        # Run all tests
make test-int    # Run integration tests only
make run         # Run git-navigator status
make install     # Install binary and aliases
make format      # Format code
make lint        # Run clippy linter
make all         # Format, lint, test, and build
```

## 🎯 Comparison with SCM Breeze

| Feature | SCM Breeze | Git Navigator |
|---------|------------|---------------|
| **Language** | Ruby/Shell | Rust |
| **Performance** | Slow startup | Sub-50ms startup |
| **Dependencies** | Ruby, complex shell setup | Single binary |
| **Cross-shell** | bash/zsh only | bash/zsh/fish/etc |
| **Features** | 100+ commands | Essential 20% that matter |

## 🤝 What I keep from SCM Breeze
- ✅ Numbered file/branch navigation 
- ✅ Flexible index syntax
- ✅ Colored, readable output
- ✅ Fast Git workflow

## 🎨 What I'm aming to improve
- ⚡ Native performance (Rust vs Ruby/Shell)
- 🔒 Reliable state management (JSON cache vs environment variables)  
- 🌐 Cross-shell compatibility
- 🔧 Zero configuration required

## 🛠️ Configuration

Git Navigator works out of the box with no configuration required. State is automatically cached in:
- `$XDG_CACHE_HOME/git-navigator/` (Linux/macOS)
- Per-repository cache using path hashes

## 🚀 Roadmap

### Phase 1: Core Commands ⚡ (COMPLETED)
- [x] **`gs`** - Git status with numbering ✅
- [x] **`ga [indices]`** - Add files by index ✅  
- [x] **`gd [indices]`** - Show diff for indexed files ✅
- [x] **`grs [indices]`** - Reset files by index ✅
- [x] **`gco [indices]`** - Checkout files by index ✅
- [x] **`gb [index]`** - Numbered branch list with checkout ✅
- [x] **Domain-specific error handling** - Clear user messages ✅
- [x] **GitStatus enum** - Type-safe status system ✅
- [x] **Template system** - Consistent output formatting ✅
- [x] **String optimizations** - Performance improvements ✅

### Phase 2: Future Enhancements 🔗
- [ ] Custom color themes and configuration
- [ ] TUI mode for interactive navigation
- Maybe more!

## 🤝 Contributing

Contributions welcome! Please see the development setup above.

## 📄 License

MIT License - see [LICENSE](./LICENSE) file for details.

## 🙏 Acknowledgments

Inspired by [SCM Breeze](https://github.com/scmbreeze/scm_breeze) - Thank you for pioneering numbered Git navigation!
