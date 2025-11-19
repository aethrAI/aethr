#!/bin/bash
# Aethr Quick Install Script
# Downloads, builds, and configures Aethr in one command

set -e  # Exit on any error

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Functions
print_header() {
    echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║  Aethr Quick Install                   ║${NC}"
    echo -e "${BLUE}║  Terminal Intelligence Tool            ║${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
    echo ""
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_info() {
    echo -e "${YELLOW}→${NC} $1"
}

check_prerequisites() {
    print_info "Checking prerequisites..."
    
    # Check Rust
    if ! command -v cargo &> /dev/null; then
        print_error "Rust not found. Installing..."
        curl --proto '=https' --tlsv1.2 -sSf https://rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
        print_success "Rust installed"
    else
        RUST_VERSION=$(rustc --version | awk '{print $2}')
        print_success "Rust $RUST_VERSION found"
    fi
    
    # Check Git
    if ! command -v git &> /dev/null; then
        print_error "Git not found. Please install Git and try again."
        exit 1
    fi
    print_success "Git found"
    
    # Check SQLite (usually pre-installed)
    if ! command -v sqlite3 &> /dev/null; then
        print_info "SQLite3 not found. Skipping (embedded in Aethr)."
    else
        print_success "SQLite3 found"
    fi
}

clone_repo() {
    print_info "Cloning Aethr repository..."
    
    if [ -d "aethr" ]; then
        print_info "aethr directory already exists, using existing..."
        cd aethr
    else
        git clone https://github.com/pinkabel/aethr.git
        cd aethr
        print_success "Repository cloned"
    fi
}

build_binary() {
    print_info "Building Aethr (this may take 1-2 minutes)..."
    cargo build --release 2>&1 | tail -5
    print_success "Build complete!"
}

setup_database() {
    print_info "Initializing database..."
    ./target/release/aethr init > /dev/null 2>&1
    print_success "Database initialized at ~/.aethr/aethr.db"
}

seed_moat() {
    print_info "Loading community moat (58+ fixes)..."
    ./target/release/aethr seed-moat > /dev/null 2>&1
    print_success "Community moat loaded"
}

add_to_path() {
    print_info "Setting up PATH..."
    
    # Create ~/.local/bin if it doesn't exist
    mkdir -p ~/.local/bin
    
    # Copy binary
    BINARY_PATH=$(pwd)/target/release/aethr
    cp "$BINARY_PATH" ~/.local/bin/aethr
    chmod +x ~/.local/bin/aethr
    
    # Add to shell config
    SHELL_RC=""
    if [ -n "$ZSH_VERSION" ]; then
        SHELL_RC="$HOME/.zshrc"
    elif [ -n "$BASH_VERSION" ]; then
        SHELL_RC="$HOME/.bashrc"
    elif [ -f "$HOME/.bashrc" ]; then
        SHELL_RC="$HOME/.bashrc"
    elif [ -f "$HOME/.zshrc" ]; then
        SHELL_RC="$HOME/.zshrc"
    fi
    
    if [ -n "$SHELL_RC" ]; then
        if ! grep -q '~/.local/bin' "$SHELL_RC"; then
            echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$SHELL_RC"
            print_success "Added ~/.local/bin to PATH in $SHELL_RC"
        else
            print_success "PATH already configured"
        fi
    fi
}

setup_history_logging() {
    print_info "Setting up optional shell history logging..."
    
    SHELL_RC=""
    if [ -n "$ZSH_VERSION" ]; then
        SHELL_RC="$HOME/.zshrc"
    elif [ -n "$BASH_VERSION" ]; then
        SHELL_RC="$HOME/.bashrc"
    elif [ -f "$HOME/.bashrc" ]; then
        SHELL_RC="$HOME/.bashrc"
    elif [ -f "$HOME/.zshrc" ]; then
        SHELL_RC="$HOME/.zshrc"
    fi
    
    if [ -n "$SHELL_RC" ]; then
        if ! grep -q 'Aethr command logging' "$SHELL_RC"; then
            cat << 'EOF' >> "$SHELL_RC"

# Aethr command logging
export HISTFILE="$HOME/.aethr/commands.log"
EOF
            print_success "Shell history logging configured"
        else
            print_success "Shell history logging already configured"
        fi
    fi
}

test_installation() {
    print_info "Testing installation..."
    
    # Test help
    if ~/.local/bin/aethr --help > /dev/null 2>&1; then
        print_success "Help command works"
    fi
    
    # Test version
    if ~/.local/bin/aethr --version > /dev/null 2>&1; then
        print_success "Version check works"
    fi
}

print_next_steps() {
    echo ""
    echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║  Installation Complete! 🎉              ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${BLUE}📝 Next steps:${NC}"
    echo ""
    echo "  1. Reload your shell:"
    echo "     ${YELLOW}source ~/.bashrc${NC}  (or ${YELLOW}source ~/.zshrc${NC})"
    echo ""
    echo "  2. See the interactive menu:"
    echo "     ${YELLOW}aethr${NC}"
    echo ""
    echo "  3. Search for a command:"
    echo "     ${YELLOW}aethr recall \"docker\"${NC}"
    echo ""
    echo "  4. Fix an error:"
    echo "     ${YELLOW}aethr fix \"permission denied\"${NC}"
    echo ""
    echo "  5. Get help:"
    echo "     ${YELLOW}aethr --help${NC}"
    echo ""
    echo -e "${BLUE}📖 Documentation:${NC}"
    echo "  - README.md (features & examples)"
    echo "  - INSTALLATION.md (full setup guide)"
    echo "  - BUSINESS_STRATEGY.md (vision & roadmap)"
    echo ""
    echo -e "${BLUE}🚀 Ready to use!${NC}"
}

main() {
    print_header
    
    # Run installation steps
    check_prerequisites
    clone_repo
    build_binary
    setup_database
    seed_moat
    add_to_path
    setup_history_logging
    test_installation
    
    print_next_steps
}

# Run main
main
