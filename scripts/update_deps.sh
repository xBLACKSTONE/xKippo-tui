#!/bin/bash

# xKippo-tui dependency update script
# This script helps update and verify dependencies for xKippo-tui

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BOLD='\033[1m'
NC='\033[0m' # No Color

echo -e "${BLUE}${BOLD}xKippo-tui Dependency Update Script${NC}"
echo -e "This script will help update and verify dependencies for xKippo-tui\n"

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: cargo is not installed.${NC}"
    echo -e "${YELLOW}Please install Rust and Cargo first:${NC}"
    echo -e "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check rust version matches the required version in rust-toolchain.toml
TOOLCHAIN_VERSION=""
if [ -f "rust-toolchain.toml" ]; then
    TOOLCHAIN_VERSION=$(grep "channel" rust-toolchain.toml | cut -d'"' -f2 || grep "channel" rust-toolchain.toml | cut -d"'" -f2 || grep "channel" rust-toolchain.toml | awk '{print $3}')
    if [ -n "$TOOLCHAIN_VERSION" ]; then
        echo -e "${BLUE}Rust toolchain version required: ${TOOLCHAIN_VERSION}${NC}"
    else
        echo -e "${YELLOW}Could not determine Rust version from rust-toolchain.toml${NC}"
    fi
else
    echo -e "${YELLOW}rust-toolchain.toml not found${NC}"
fi

# Check current rust version
CURRENT_VERSION=$(rustc --version | cut -d' ' -f2)
echo -e "${BLUE}Current Rust version: ${CURRENT_VERSION}${NC}"

# Compare versions if needed
if [ -n "$TOOLCHAIN_VERSION" ] && [ "$TOOLCHAIN_VERSION" != "$CURRENT_VERSION" ]; then
    echo -e "${YELLOW}Warning: Current Rust version ($CURRENT_VERSION) doesn't match toolchain version ($TOOLCHAIN_VERSION)${NC}"
    echo -e "You can update your Rust installation to the correct version with:"
    echo -e "  rustup install $TOOLCHAIN_VERSION"
    echo -e "  rustup default $TOOLCHAIN_VERSION"
    
    read -p "Would you like to install the required Rust version now? (y/n): " install_rust
    if [[ $install_rust =~ ^[Yy] ]]; then
        rustup install $TOOLCHAIN_VERSION
        rustup default $TOOLCHAIN_VERSION
        echo -e "${GREEN}✓ Rust updated to version $TOOLCHAIN_VERSION${NC}"
    fi
fi

echo -e "\n${BLUE}Checking for dependency issues...${NC}"

# Run cargo check to verify dependencies
cargo check
if [ $? -ne 0 ]; then
    echo -e "${RED}There are issues with dependencies.${NC}"
    
    # Suggest some common fixes
    echo -e "\n${YELLOW}Suggested fixes:${NC}"
    echo -e "1. Make sure you're using Rust version $TOOLCHAIN_VERSION"
    echo -e "2. Update dependencies with: cargo update"
    echo -e "3. Clean build artifacts: cargo clean"
    echo -e "4. Check for specific version conflicts in Cargo.lock"
    
    # Ask if they want to try to fix automatically
    read -p "Would you like to try to fix dependencies automatically? (y/n): " fix_deps
    if [[ $fix_deps =~ ^[Yy] ]]; then
        echo -e "\n${BLUE}Running cargo update...${NC}"
        cargo update
        echo -e "\n${BLUE}Running cargo clean...${NC}"
        cargo clean
        echo -e "\n${BLUE}Checking dependencies again...${NC}"
        cargo check
        
        if [ $? -eq 0 ]; then
            echo -e "${GREEN}✓ Dependencies fixed successfully!${NC}"
        else
            echo -e "${RED}Automatic fix failed. You may need to manually resolve dependency issues.${NC}"
            echo -e "Try examining the Cargo.lock file and comparing with specific version requirements."
        fi
    fi
else
    echo -e "${GREEN}✓ All dependencies look good!${NC}"
fi

# Offer to update specific dependencies that are causing issues
echo -e "\n${BLUE}Common dependency troubleshooting:${NC}"
echo -e "1. Update specific dependency"
echo -e "2. Check dependency tree"
echo -e "3. Generate Cargo.lock"
echo -e "4. Exit"

read -p "Choose an option (1-4): " dep_option

case $dep_option in
    1)
        read -p "Enter dependency name to update: " dep_name
        if [ -n "$dep_name" ]; then
            cargo update $dep_name
            echo -e "${GREEN}✓ Updated $dep_name${NC}"
        fi
        ;;
    2)
        if ! command -v cargo-tree &> /dev/null; then
            echo -e "${YELLOW}cargo-tree not found. Installing...${NC}"
            cargo install cargo-tree
        fi
        cargo tree
        ;;
    3)
        echo -e "${BLUE}Generating Cargo.lock file...${NC}"
        cargo generate-lockfile
        echo -e "${GREEN}✓ Generated new Cargo.lock file${NC}"
        ;;
    *)
        echo -e "Exiting dependency checker."
        ;;
esac

echo -e "\n${GREEN}${BOLD}Dependency check completed!${NC}"
echo -e "You can now build xKippo-tui with: ./scripts/install.sh"
exit 0