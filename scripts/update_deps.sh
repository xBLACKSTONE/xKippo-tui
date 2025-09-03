#!/bin/bash

# xKippo-tui dependency update script
# This script helps update and verify dependencies for xKippo-tui
#
# This script will automatically fix common dependency issues:
# - Downgrade icu_normalizer_data for Rust 1.75.0 compatibility
# - Fix problematic dependencies with version conflicts
# - Handle Rust version requirements

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
cargo check 2>&1 | tee /tmp/cargo_check_output.log
CHECK_RESULT=$?

# Known dependency issues to automatically fix
FIX_ICU_NORMALIZER="false"
OTHER_FIXES_NEEDED="false"

# Parse error output for known issues
if [ $CHECK_RESULT -ne 0 ]; then
    echo -e "${RED}There are issues with dependencies.${NC}"
    
    # Check for icu_normalizer_data error
    if grep -q "package .icu_normalizer_data.*cannot be built because it requires rustc" /tmp/cargo_check_output.log; then
        echo -e "${YELLOW}Detected icu_normalizer_data version issue. Will attempt to fix.${NC}"
        FIX_ICU_NORMALIZER="true"
    fi
    
    # Look for other package version issues
    PACKAGE_VERSION_ISSUES=$(grep -o "package .\+@[0-9]\+\.[0-9]\+\.[0-9]\+.* cannot be built" /tmp/cargo_check_output.log | sed 's/package `\(.*\)` cannot be built.*/\1/')
    if [ -n "$PACKAGE_VERSION_ISSUES" ]; then
        echo -e "${YELLOW}Detected version issues with these packages:${NC}"
        echo "$PACKAGE_VERSION_ISSUES" | while read -r pkg; do
            echo -e "- $pkg"
        done
        OTHER_FIXES_NEEDED="true"
    fi
    
    # Apply known fixes or suggest manual intervention
    echo -e "\n${YELLOW}Applying automatic fixes:${NC}"
    
    if [ "$FIX_ICU_NORMALIZER" = "true" ]; then
        echo -e "${BLUE}Fixing icu_normalizer_data dependency...${NC}"
        cargo update -p icu_normalizer_data --precise 1.0.0
        if [ $? -eq 0 ]; then
            echo -e "${GREEN}✓ Successfully downgraded icu_normalizer_data to version compatible with Rust 1.75.0${NC}"
        else
            echo -e "${RED}Failed to downgrade icu_normalizer_data automatically${NC}"
        fi
    fi
    
    if [ "$FIX_ICU_NORMALIZER" = "true" ] || [ "$OTHER_FIXES_NEEDED" = "true" ]; then
        echo -e "\n${BLUE}Cleaning and checking again with fixed dependencies...${NC}"
        cargo clean
        cargo check
        
        if [ $? -eq 0 ]; then
            echo -e "${GREEN}✓ Dependencies fixed successfully!${NC}"
        else
            echo -e "${RED}Some dependency issues still remain.${NC}"
            
            # Extract recommendations from error message
            RECOMMENDATIONS=$(grep -A 2 "use.*cargo update" /tmp/cargo_check_output.log | grep -v "^error")
            if [ -n "$RECOMMENDATIONS" ]; then
                echo -e "${YELLOW}Recommended commands from error message:${NC}"
                echo -e "$RECOMMENDATIONS"
                
                # Offer to execute the recommended command
                CMD=$(echo "$RECOMMENDATIONS" | grep "cargo update" | tr -d ' ')
                if [ -n "$CMD" ]; then
                    read -p "Execute the recommended command? (y/n): " exec_cmd
                    if [[ $exec_cmd =~ ^[Yy] ]]; then
                        eval "$CMD"
                        cargo clean
                        cargo check
                    fi
                fi
            fi
        fi
    fi
else
    echo -e "${GREEN}✓ All dependencies look good!${NC}"
fi

# Clean up temporary files
rm -f /tmp/cargo_check_output.log
else
    echo -e "${GREEN}✓ All dependencies look good!${NC}"
fi

# Offer to update specific dependencies that are causing issues
echo -e "\n${BLUE}Common dependency troubleshooting:${NC}"
echo -e "1. Update specific dependency"
echo -e "2. Downgrade a dependency to specific version"
echo -e "3. Check dependency tree"
echo -e "4. Run advanced dependency fix"
echo -e "5. Generate Cargo.lock"
echo -e "6. Exit"

read -p "Choose an option (1-6): " dep_option

case $dep_option in
    1)
        read -p "Enter dependency name to update: " dep_name
        if [ -n "$dep_name" ]; then
            cargo update $dep_name
            echo -e "${GREEN}✓ Updated $dep_name${NC}"
        fi
        ;;
    2)
        read -p "Enter dependency name to downgrade (e.g. icu_normalizer_data): " dep_name
        if [ -n "$dep_name" ]; then
            read -p "Enter version to use (e.g. 1.0.0): " dep_version
            if [ -n "$dep_version" ]; then
                echo -e "${BLUE}Downgrading $dep_name to version $dep_version...${NC}"
                cargo update -p $dep_name --precise $dep_version
                if [ $? -eq 0 ]; then
                    echo -e "${GREEN}✓ Successfully downgraded $dep_name to version $dep_version${NC}"
                else
                    echo -e "${RED}Failed to downgrade $dep_name${NC}"
                fi
            fi
        fi
        ;;
    3)
        if ! command -v cargo-tree &> /dev/null; then
            echo -e "${YELLOW}cargo-tree not found. Installing...${NC}"
            cargo install cargo-tree
        fi
        cargo tree
        ;;
    4)
        echo -e "${BLUE}Running advanced dependency fix...${NC}"
        echo -e "${YELLOW}This will attempt to fix common dependency issues by downgrading problematic packages${NC}"
        
        # List of known problematic dependencies with compatible versions for Rust 1.75.0
        echo -e "${BLUE}Downgrading known problematic dependencies...${NC}"
        
        cargo update -p icu_normalizer_data --precise 1.0.0
        cargo update -p regex --precise 1.9.5
        cargo update -p regex-syntax --precise 0.8.2
        cargo update -p icu_provider --precise 1.2.0
        cargo update -p icu_provider_adapters --precise 1.2.0
        cargo update -p icu_collections --precise 1.2.0
        
        echo -e "${BLUE}Cleaning and checking again...${NC}"
        cargo clean
        cargo check
        
        if [ $? -eq 0 ]; then
            echo -e "${GREEN}✓ Advanced dependency fix successful!${NC}"
        else
            echo -e "${RED}Some dependency issues still remain.${NC}"
            echo -e "${YELLOW}You may need to manually edit Cargo.toml to pin problematic dependencies.${NC}"
        fi
        ;;
    5)
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
echo -e "\n${YELLOW}TIP: If you still encounter issues, try the following:${NC}"
echo -e "1. Run this script again and choose option 4 (Run advanced dependency fix)"
echo -e "2. Manually edit Cargo.toml to add version constraints for problematic dependencies:"
echo -e "   regex = \"=1.9.5\""
echo -e "   icu_normalizer_data = \"=1.0.0\""
echo -e "   icu_provider = \"=1.2.0\""
echo -e "3. For Rust 1.75.0 compatibility, avoid dependencies requiring Rust 1.82.0 or newer"
exit 0