#!/bin/bash

set -e

# Git Navigator Remote Installer
# Usage: curl -sSL https://raw.githubusercontent.com/git-navigator/git-navigator/main/install.sh | bash

REPO="git-navigator/git-navigator"
INSTALL_DIR="$HOME/.local/bin"
BINARY_NAME="git-navigator"

echo "🚀 Installing git-navigator..."

# Detect platform and architecture
detect_platform() {
    local os arch target
    
    # Detect OS
    case "$OSTYPE" in
        linux*)
            os="linux"
            ;;
        darwin*)
            os="macos"
            ;;
        msys*|cygwin*|mingw*)
            os="windows"
            ;;
        *)
            echo "❌ Unsupported OS: $OSTYPE"
            exit 1
            ;;
    esac
    
    # Detect architecture
    arch=$(uname -m)
    case "$arch" in
        x86_64|amd64)
            arch="x64"
            ;;
        aarch64|arm64)
            arch="arm64"
            ;;
        *)
            echo "❌ Unsupported architecture: $arch"
            exit 1
            ;;
    esac
    
    # Map to GitHub release asset names
    case "$os-$arch" in
        linux-x64)
            target="x86_64-unknown-linux-gnu"
            ;;
        linux-arm64)
            target="aarch64-unknown-linux-gnu"
            ;;
        macos-x64)
            target="x86_64-apple-darwin"
            ;;
        macos-arm64)
            target="aarch64-apple-darwin"
            ;;
        windows-x64)
            target="x86_64-pc-windows-gnu"
            ;;
        *)
            echo "❌ No binary available for $os-$arch"
            exit 1
            ;;
    esac
    
    echo "Detected platform: $os-$arch (target: $target)"
    echo "$target"
}

# Get latest release version from GitHub
get_latest_version() {
    echo "Fetching latest release..."
    if command -v curl >/dev/null 2>&1; then
        curl -s "https://api.github.com/repos/$REPO/releases/latest" | \
            grep '"tag_name"' | \
            sed -E 's/.*"tag_name": "([^"]+)".*/\1/'
    elif command -v wget >/dev/null 2>&1; then
        wget -qO- "https://api.github.com/repos/$REPO/releases/latest" | \
            grep '"tag_name"' | \
            sed -E 's/.*"tag_name": "([^"]+)".*/\1/'
    else
        echo "❌ Neither curl nor wget found. Cannot download."
        exit 1
    fi
}

# Download and install binary
install_binary() {
    local target="$1"
    local version="$2"
    local binary_name
    local download_url
    local temp_file
    
    # Determine binary name based on platform
    if [[ "$target" == *"windows"* ]]; then
        binary_name="git-navigator-${target}.exe"
    else
        binary_name="git-navigator-${target}"
    fi
    
    download_url="https://github.com/$REPO/releases/download/$version/$binary_name"
    temp_file="/tmp/$binary_name"
    
    echo "Downloading $binary_name from $version..."
    
    # Download binary
    if command -v curl >/dev/null 2>&1; then
        curl -sL "$download_url" -o "$temp_file"
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$download_url" -O "$temp_file"
    else
        echo "❌ Neither curl nor wget found. Cannot download."
        exit 1
    fi
    
    # Verify download
    if [[ ! -f "$temp_file" ]] || [[ ! -s "$temp_file" ]]; then
        echo "❌ Download failed or file is empty"
        exit 1
    fi
    
    # Create install directory
    mkdir -p "$INSTALL_DIR"
    
    # Install binary
    if [[ "$target" == *"windows"* ]]; then
        mv "$temp_file" "$INSTALL_DIR/git-navigator.exe"
        chmod +x "$INSTALL_DIR/git-navigator.exe"
    else
        mv "$temp_file" "$INSTALL_DIR/$BINARY_NAME"
        chmod +x "$INSTALL_DIR/$BINARY_NAME"
    fi
    
    echo "✓ Binary installed to $INSTALL_DIR/$BINARY_NAME"
}

# Add install directory to PATH if needed
setup_path() {
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        echo "Adding $INSTALL_DIR to PATH..."
        
        # Detect shell and add to appropriate config file
        local shell_config
        case "$SHELL" in
            */bash)
                if [[ "$OSTYPE" == "darwin"* ]] && [[ -f "$HOME/.bash_profile" ]]; then
                    shell_config="$HOME/.bash_profile"
                else
                    shell_config="$HOME/.bashrc"
                fi
                ;;
            */zsh)
                shell_config="$HOME/.zshrc"
                ;;
            */fish)
                shell_config="$HOME/.config/fish/config.fish"
                mkdir -p "$(dirname "$shell_config")"
                ;;
            *)
                shell_config="$HOME/.profile"
                ;;
        esac
        
        # Add PATH export
        if [[ "$SHELL" == */fish ]]; then
            echo "set -gx PATH $INSTALL_DIR \$PATH" >> "$shell_config"
        else
            echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$shell_config"
        fi
        
        echo "✓ Added $INSTALL_DIR to PATH in $shell_config"
        export PATH="$INSTALL_DIR:$PATH"
    else
        echo "✓ $INSTALL_DIR already in PATH"
    fi
}

# Function to check if aliases already exist in a file
check_aliases_exist() {
    local config_file="$1"
    if [[ -f "$config_file" ]]; then
        if grep -q "git-navigator status" "$config_file" 2>/dev/null; then
            return 0  # Aliases exist
        fi
    fi
    return 1  # Aliases don't exist
}

# Function to add aliases to a config file
add_aliases_to_file() {
    local config_file="$1"

    if check_aliases_exist "$config_file"; then
        echo "✓ git-navigator aliases already exist in $config_file"
        return 0
    fi

    echo "Adding aliases to $config_file..."
    cat >> "$config_file" << 'EOF'

# Git Navigator aliases - Clean, lean and fast git productivity tool
alias gs="git-navigator status"
alias ga="git-navigator add"
alias gd="git-navigator diff"
alias grs="git-navigator reset"
alias gco="git-navigator checkout"
alias gb="git-navigator branches"
alias gcb="git-navigator checkout-branch"
alias gl="git log --graph --pretty=format:'%Cred%h%Creset -%C(yellow)%d%Creset %s %Cgreen(%cr) %C(bold blue)<%an>%Creset' --abbrev-commit"
EOF

    echo "✓ Aliases added to $config_file"
}

# Special function for fish shell (uses different alias syntax)
setup_fish_aliases() {
    local config_file="$1"
    
    if check_aliases_exist "$config_file"; then
        echo "✓ git-navigator aliases already exist in $config_file"
        return 0
    fi
    
    echo "Adding fish aliases to $config_file..."
    
    cat >> "$config_file" << 'EOF'

# Git Navigator aliases - Clean, lean and fast git productivity tool
alias gs='git-navigator status'
alias ga='git-navigator add'
alias gd='git-navigator diff'
alias grs='git-navigator reset'
alias gco='git-navigator checkout'
alias gb='git-navigator branches'
alias gcb='git-navigator checkout-branch'
alias gl="git log --graph --pretty=format:'%Cred%h%Creset -%C(yellow)%d%Creset %s %Cgreen(%cr) %C(bold blue)<%an>%Creset' --abbrev-commit"
EOF
    
    echo "✓ Fish aliases added to $config_file"
}

# Setup shell aliases
setup_aliases() {
    echo "Setting up git-navigator aliases..."
    
    local shell_detected=false
    local config_file=""
    local shell_name=""
    
    # Primary detection: Use $SHELL environment variable
    if [[ -n "$SHELL" ]]; then
        case "$SHELL" in
            */bash)
                shell_name="bash"
                config_file="$HOME/.bashrc"
                # Also check for .bash_profile on macOS
                if [[ "$OSTYPE" == "darwin"* ]] && [[ -f "$HOME/.bash_profile" ]]; then
                    config_file="$HOME/.bash_profile"
                fi
                shell_detected=true
                ;;
            */zsh)
                shell_name="zsh"
                config_file="$HOME/.zshrc"
                shell_detected=true
                ;;
            */fish)
                shell_name="fish"
                config_file="$HOME/.config/fish/config.fish"
                # Create fish config directory if it doesn't exist
                mkdir -p "$(dirname "$config_file")"
                shell_detected=true
                ;;
            */tcsh|*/csh)
                shell_name="csh/tcsh"
                config_file="$HOME/.cshrc"
                shell_detected=true
                ;;
            *)
                echo "Unknown shell: $SHELL"
                ;;
        esac
    fi
    
    # Fallback detection: Use shell-specific environment variables
    if [[ "$shell_detected" == false ]]; then
        if [[ -n "$ZSH_VERSION" ]]; then
            shell_name="zsh"
            config_file="$HOME/.zshrc"
            shell_detected=true
        elif [[ -n "$BASH_VERSION" ]]; then
            shell_name="bash"
            config_file="$HOME/.bashrc"
            # Also check for .bash_profile on macOS
            if [[ "$OSTYPE" == "darwin"* ]] && [[ -f "$HOME/.bash_profile" ]]; then
                config_file="$HOME/.bash_profile"
            fi
            shell_detected=true
        fi
    fi
    
    # Setup aliases if shell was detected
    if [[ "$shell_detected" == true ]]; then
        echo "Detected $shell_name shell"
        
        # Special handling for fish shell (different alias syntax)
        if [[ "$shell_name" == "fish" ]]; then
            setup_fish_aliases "$config_file"
        else
            add_aliases_to_file "$config_file"
        fi
    else
        echo "⚠️  Could not detect shell automatically."
        echo "Please manually add these aliases to your shell configuration:"
        echo ""
        echo "alias gs=\"git-navigator status\""
        echo "alias ga=\"git-navigator add\""
        echo "alias gd=\"git-navigator diff\""
        echo "alias grs=\"git-navigator reset\""
        echo "alias gco=\"git-navigator checkout\""
        echo "alias gb=\"git-navigator branches\""
        echo "alias gcb=\"git-navigator checkout-branch\""
        echo ""
        echo "Common configuration files:"
        echo "  Bash: ~/.bashrc or ~/.bash_profile"
        echo "  Zsh: ~/.zshrc"
        echo "  Fish: ~/.config/fish/config.fish"
    fi
}

# Verify installation
verify_installation() {
    if command -v git-navigator >/dev/null 2>&1; then
        echo "✓ git-navigator is available in PATH"
        echo "Version: $(git-navigator --version 2>/dev/null || echo 'unknown')"
    else
        echo "⚠️  git-navigator not found in PATH. You may need to restart your terminal."
    fi
}

# Main installation flow
main() {
    # Check for local installation (development mode)
    if [[ -f "Cargo.toml" ]] && grep -q "git-navigator" "Cargo.toml" 2>/dev/null; then
        echo "🔧 Local development detected, installing via cargo..."
        if command -v cargo >/dev/null 2>&1; then
            cargo install --path .
            setup_aliases
            echo ""
            echo "🎉 git-navigator installed successfully from local source!"
        else
            echo "❌ cargo not found. Please install Rust first."
            exit 1
        fi
    else
        # Remote installation
        local target version
        target=$(detect_platform)
        version=$(get_latest_version)
        
        if [[ -z "$version" ]]; then
            echo "❌ Could not fetch latest version"
            exit 1
        fi
        
        echo "Installing version: $version"
        
        install_binary "$target" "$version"
        setup_path
        setup_aliases
        verify_installation
        
        echo ""
        echo "🎉 git-navigator installed successfully!"
    fi
    
    echo ""
    echo "Available commands:"
    echo "  gs    - Show numbered git status"
    echo "  ga    - Add files by index"
    echo "  gd    - Show diff by index" 
    echo "  grs   - Reset files by index"
    echo "  gco   - Checkout files by index"
    echo "  gb    - Show numbered branches"
    echo "  gcb   - Checkout branch by index"
    echo "  gl    - Visual git log"
    echo ""
    echo "Example usage:"
    echo "  gs              # Show numbered file status"
    echo "  ga 1 3-5        # Add files 1, 3, 4, 5"
    echo "  gd 2,4          # Show diff for files 2 and 4"
    echo ""
    echo "💡 Restart your terminal or run 'source ~/.bashrc' (or your shell config) to use aliases"
    echo ""
    echo "Clean, lean and fast git productivity tool! 🚀"
}

# Run main function
main "$@"