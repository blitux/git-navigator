#!/bin/bash

set -e

# Git Navigator Remote Installer
# Usage: curl -sSL https://raw.githubusercontent.com/blitux/git-navigator/main/install.sh | bash

REPO="blitux/git-navigator"
INSTALL_DIR="$HOME/.local/bin"
BINARY_NAME="git-navigator"

# Detect platform and architecture
detect_platform() {
    local os arch

    # Detect OS
    case "$OSTYPE" in
        linux*) os="linux" ;;
        darwin*) os="macos" ;;
        msys*|cygwin*|mingw*) os="windows" ;;
        *) echo "❌ Unsupported OS: $OSTYPE"; exit 1 ;;
    esac

    # Detect architecture
    arch=$(uname -m)
    case "$arch" in
        x86_64|amd64) arch="x64" ;;
        aarch64|arm64) arch="arm64" ;;
        *) echo "❌ Unsupported architecture: $arch"; exit 1 ;;
    esac

    echo "$os-$arch"
}

# Get latest release version from GitHub
get_latest_version() {
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
    local platform="$1"
    local version="$2"
    local binary_name="git-navigator-${platform}"
    local download_url="https://github.com/$REPO/releases/download/$version/$binary_name"
    local temp_file="/tmp/$binary_name"

    echo "Downloading $binary_name from version $version..."
    echo "Download URL: $download_url"

    if command -v curl >/dev/null 2>&1; then
        curl -sL "$download_url" -o "$temp_file"
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$download_url" -O "$temp_file"
    else
        echo "❌ Neither curl nor wget found. Cannot download."
        exit 1
    fi

    if [[ ! -f "$temp_file" ]] || [[ ! -s "$temp_file" ]]; then
        echo "❌ Download failed or file is empty"
        exit 1
    fi

    mkdir -p "$INSTALL_DIR"

    if [[ "$platform" == windows-* ]]; then
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
        local shell_config
        case "$SHELL" in
            */bash)
                [[ "$OSTYPE" == "darwin"* && -f "$HOME/.bash_profile" ]] && shell_config="$HOME/.bash_profile" || shell_config="$HOME/.bashrc"
                ;;
            */zsh) shell_config="$HOME/.zshrc" ;;
            */fish)
                shell_config="$HOME/.config/fish/config.fish"
                mkdir -p "$(dirname "$shell_config")"
                ;;
            *) shell_config="$HOME/.profile" ;;
        esac

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

# Setup shell aliases
setup_aliases() {
    echo "Setting up git-navigator aliases..."
    local config_file=""
    case "$SHELL" in
        */bash) config_file="$HOME/.bashrc" ;;
        */zsh) config_file="$HOME/.zshrc" ;;
        */fish) config_file="$HOME/.config/fish/config.fish" ;;
        *) config_file="$HOME/.profile" ;;
    esac

    if grep -q "git-navigator status" "$config_file" 2>/dev/null; then
        echo "✓ Aliases already set in $config_file"
        return
    fi

    echo "Adding aliases to $config_file..."
    cat >> "$config_file" << 'EOF'

# Git Navigator aliases
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

# Verify installation
verify_installation() {
    if command -v git-navigator >/dev/null 2>&1; then
        echo "✓ git-navigator is available in PATH"
    else
        echo "⚠️ git-navigator not found in PATH. You may need to restart your terminal."
    fi
}

# Main installation flow
main() {
    if [[ -f "Cargo.toml" ]] && grep -q "git-navigator" "Cargo.toml" 2>/dev/null; then
        echo "🔧 Local development detected, installing via cargo..."
        if command -v cargo >/dev/null 2>&1; then
            cargo install --path .
            setup_aliases
            echo "\n🎉 git-navigator installed successfully from local source!"
        else
            echo "❌ cargo not found. Please install Rust first."
            exit 1
        fi
    else
        local platform version
        platform=$(detect_platform)
        version=$(get_latest_version)

        if [[ -z "$version" ]]; then
            echo "❌ Could not fetch latest version"
            exit 1
        fi

        echo "🚀 Installing git-navigator..."
        echo "Detected platform: $platform"
        echo "Installing version: $version"

        install_binary "$platform" "$version"
        setup_path
        setup_aliases
        verify_installation

        echo -e "\n🎉 git-navigator installed successfully!"
        echo -e "\nAvailable commands:"
        echo "  gs    - Show numbered git status"
        echo "  ga    - Add files by index"
        echo "  gd    - Show diff by index" 
        echo "  grs   - Reset files by index"
        echo "  gco   - Checkout files by index"
        echo "  gb    - Show numbered branches"
        echo "  gcb   - Checkout branch by index"
        echo "  gl    - Visual git log"
        echo -e "\nExample usage:"
        echo "  gs              # Show numbered file status"
        echo "  ga 1 3-5        # Add files 1, 3, 4, 5"
        echo "  gd 2,4          # Show diff for files 2 and 4"
        echo -e "\n💡 Restart your terminal or run 'source ~/.bashrc' (or your shell config) to use aliases"
        echo -e "\nClean, lean and fast git productivity tool! 🚀"
        echo ""
    fi
}

main "$@"
