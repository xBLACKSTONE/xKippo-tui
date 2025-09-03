#!/bin/bash

# xKippo-tui installation script
# This script helps you build and install xKippo-tui

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BOLD='\033[1m'
NC='\033[0m' # No Color

echo -e "${BLUE}${BOLD}xKippo-tui Installation Script${NC}"
echo -e "This script will build and install xKippo-tui on your system\n"

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: cargo is not installed.${NC}"
    echo -e "${YELLOW}Please install Rust and Cargo first:${NC}"
    echo -e "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Ensure pkg-config and other dependencies are installed
echo -e "${BLUE}Checking for required dependencies...${NC}"
missing_deps=()

# Check for pkg-config
if ! command -v pkg-config &> /dev/null; then
    missing_deps+=("pkg-config")
fi

# Check for OpenSSL development packages
if ! pkg-config --exists openssl 2>/dev/null; then
    missing_deps+=("openssl-dev or libssl-dev")
fi

# Check for Rust version
if ! command -v rustc &> /dev/null; then
    missing_deps+=("rust")
else
    # Get installed Rust version
    RUST_VERSION=$(rustc --version | cut -d' ' -f2)
    REQUIRED_VERSION="1.75.0"
    
    # Compare versions
    if ! command -v python3 &> /dev/null; then
        echo -e "${YELLOW}Warning: Python3 not found, cannot compare Rust versions precisely.${NC}"
        echo -e "${YELLOW}You may need Rust $REQUIRED_VERSION or later.${NC}"
    else
        VERSION_CHECK=$(python3 -c "from packaging import version; print(version.parse('$RUST_VERSION') < version.parse('$REQUIRED_VERSION'))" 2>/dev/null)
        if [ "$?" -ne 0 ]; then
            echo -e "${YELLOW}Warning: Could not compare Rust versions. Python 'packaging' module might be missing.${NC}"
            echo -e "${YELLOW}Please ensure you have Rust $REQUIRED_VERSION or later.${NC}"
        elif [ "$VERSION_CHECK" = "True" ]; then
            echo -e "${YELLOW}Warning: Rust $RUST_VERSION is older than required version $REQUIRED_VERSION${NC}"
            echo -e "${YELLOW}Please update your Rust installation:${NC}"
            echo -e "  rustup update stable"
            # Don't fail the build yet, just warn
        fi
    fi
fi

if [ ${#missing_deps[@]} -gt 0 ]; then
    echo -e "${RED}Missing required dependencies: ${missing_deps[*]}${NC}"
    echo -e "${YELLOW}Please install them using your distribution's package manager:${NC}"
    
    if command -v apt-get &> /dev/null; then
        echo -e "  sudo apt-get update && sudo apt-get install -y pkg-config libssl-dev"
    elif command -v dnf &> /dev/null; then
        echo -e "  sudo dnf install -y pkgconfig openssl-devel"
    elif command -v yum &> /dev/null; then
        echo -e "  sudo yum install -y pkgconfig openssl-devel"
    elif command -v brew &> /dev/null; then
        echo -e "  brew install pkg-config openssl"
    else
        echo -e "Please install pkg-config and OpenSSL development packages."
    fi
    
    echo -e "\n${YELLOW}Would you like to try to install these dependencies automatically? (y/n)${NC}"
    read -r install_deps
    if [[ $install_deps =~ ^[Yy] ]]; then
        if command -v apt-get &> /dev/null; then
            sudo apt-get update
            sudo apt-get install -y pkg-config libssl-dev
        elif command -v dnf &> /dev/null; then
            sudo dnf install -y pkgconfig openssl-devel
        elif command -v yum &> /dev/null; then
            sudo yum install -y pkgconfig openssl-devel
        elif command -v brew &> /dev/null; then
            brew install pkg-config openssl
        else
            echo -e "${RED}Could not automatically install dependencies.${NC}"
            echo -e "${RED}Please install them manually and run this script again.${NC}"
            exit 1
        fi
    else
        echo -e "${RED}Please install the required dependencies and run this script again.${NC}"
        exit 1
    fi
fi

echo -e "${GREEN}✓ All dependencies are installed.${NC}"

# Build the project with pinned dependency versions
echo -e "\n${BLUE}Building xKippo-tui with correct dependency versions...${NC}"

# Check for rust-toolchain.toml to ensure correct Rust version
if [ -f "rust-toolchain.toml" ]; then
    echo -e "${GREEN}✓ Using rust-toolchain.toml for Rust version${NC}"
else
    echo -e "${YELLOW}Warning: rust-toolchain.toml not found. This may cause dependency version issues.${NC}"
fi

# Ensure exact versions from Cargo.toml are used
cargo build --release --locked

if [ $? -ne 0 ]; then
    echo -e "${RED}Build failed. Please check the error messages above.${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Build successful!${NC}"

# Check for the binary
if [ ! -f "target/release/xkippo-tui" ]; then
    echo -e "${RED}Binary not found at target/release/xkippo-tui${NC}"
    exit 1
fi

# Install options
echo -e "\n${BLUE}Installation options:${NC}"
echo -e "1. Install to system path (/usr/local/bin)"
echo -e "2. Install to user path (~/.local/bin)"
echo -e "3. Create a symlink in /usr/local/bin"
echo -e "4. Create a symlink in ~/.local/bin"
echo -e "5. Skip installation (just build)"

read -p "Choose an option (1-5): " install_option

case $install_option in
    1)
        echo -e "${YELLOW}Installing to /usr/local/bin (requires sudo)...${NC}"
        sudo cp "target/release/xkippo-tui" "/usr/local/bin/"
        echo -e "${GREEN}✓ Installed to /usr/local/bin/xkippo-tui${NC}"
        ;;
    2)
        echo -e "${YELLOW}Installing to ~/.local/bin...${NC}"
        mkdir -p "$HOME/.local/bin"
        cp "target/release/xkippo-tui" "$HOME/.local/bin/"
        echo -e "${GREEN}✓ Installed to ~/.local/bin/xkippo-tui${NC}"
        
        # Add ~/.local/bin to PATH if not already there
        if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
            echo -e "${YELLOW}Consider adding ~/.local/bin to your PATH:${NC}"
            echo -e "  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc"
            echo -e "  source ~/.bashrc"
        fi
        ;;
    3)
        echo -e "${YELLOW}Creating symlink in /usr/local/bin (requires sudo)...${NC}"
        sudo ln -sf "$(pwd)/target/release/xkippo-tui" "/usr/local/bin/xkippo-tui"
        echo -e "${GREEN}✓ Created symlink at /usr/local/bin/xkippo-tui${NC}"
        ;;
    4)
        echo -e "${YELLOW}Creating symlink in ~/.local/bin...${NC}"
        mkdir -p "$HOME/.local/bin"
        ln -sf "$(pwd)/target/release/xkippo-tui" "$HOME/.local/bin/xkippo-tui"
        echo -e "${GREEN}✓ Created symlink at ~/.local/bin/xkippo-tui${NC}"
        
        # Add ~/.local/bin to PATH if not already there
        if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
            echo -e "${YELLOW}Consider adding ~/.local/bin to your PATH:${NC}"
            echo -e "  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc"
            echo -e "  source ~/.bashrc"
        fi
        ;;
    5)
        echo -e "${YELLOW}Skipping installation. The binary is available at target/release/xkippo-tui${NC}"
        ;;
    *)
        echo -e "${RED}Invalid option. Skipping installation.${NC}"
        echo -e "${YELLOW}The binary is available at target/release/xkippo-tui${NC}"
        ;;
esac

echo -e "\n${BLUE}${BOLD}Installation Complete!${NC}"
echo -e "You can now run xKippo-tui to monitor your Cowrie honeypot logs."
echo -e "Run the setup script first to configure the tool:"
echo -e "  ./scripts/setup.sh"

exit 0