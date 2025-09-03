#!/bin/bash

# xKippo-tui force dependency fix script
# This script forcefully fixes dependency issues for Rust 1.75.0 compatibility

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BOLD='\033[1m'
NC='\033[0m' # No Color

echo -e "${BLUE}${BOLD}xKippo-tui Force Dependency Fix Script${NC}"
echo -e "${YELLOW}This script will forcefully fix dependency issues for Rust 1.75.0 compatibility${NC}\n"

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: cargo is not installed.${NC}"
    echo -e "${YELLOW}Please install Rust and Cargo first:${NC}"
    echo -e "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check current rust version
CURRENT_VERSION=$(rustc --version | cut -d' ' -f2)
echo -e "${BLUE}Current Rust version: ${CURRENT_VERSION}${NC}"

# Create backup of Cargo.toml
echo -e "${BLUE}Creating backup of Cargo.toml...${NC}"
cp Cargo.toml Cargo.toml.bak
echo -e "${GREEN}✓ Backup created: Cargo.toml.bak${NC}"

# Create a temporary file for the modified Cargo.toml
TEMP_CARGO=$(mktemp)

echo -e "${BLUE}Modifying Cargo.toml to pin problematic dependencies...${NC}"

# Parse existing Cargo.toml and add version constraints
cat Cargo.toml | awk '
BEGIN { in_dependencies = 0; modified = 0; }
/^\[dependencies\]/ { in_dependencies = 1; print; next; }
/^\[/ && !/^\[dependencies\]/ { in_dependencies = 0; print; next; }
in_dependencies && /^icu_normalizer_data/ {
    print "icu_normalizer_data = \"=1.5.0\""; 
    modified = 1;
    next;
}
in_dependencies && /^regex\s*=/ {
    print "regex = \"=1.9.5\"";
    modified = 1;
    next;
}
in_dependencies && /^regex-syntax\s*=/ {
    print "regex-syntax = \"=0.8.2\"";
    modified = 1;
    next;
}
in_dependencies && /^icu_provider\s*=/ {
    print "icu_provider = \"=1.5.0\"";
    modified = 1;
    next;
}
in_dependencies && /^icu_provider_adapters\s*=/ {
    print "icu_provider_adapters = \"=1.5.0\"";
    modified = 1;
    next;
}
in_dependencies && /^icu_collections\s*=/ {
    print "icu_collections = \"=1.5.0\"";
    modified = 1;
    next;
}
{ print; }
' > "$TEMP_CARGO"

# Replace the original Cargo.toml with the modified version
mv "$TEMP_CARGO" Cargo.toml

echo -e "${GREEN}✓ Modified Cargo.toml to pin dependency versions${NC}"

# Clean build artifacts
echo -e "${BLUE}Cleaning build artifacts...${NC}"
cargo clean

# Update dependencies with pinned versions
echo -e "${BLUE}Updating dependencies with pinned versions...${NC}"
cargo update

# Additional targeted version fixes
echo -e "${BLUE}Applying targeted version fixes...${NC}"
cargo update -p icu_normalizer_data --precise 1.5.0
cargo update -p regex --precise 1.9.5
cargo update -p regex-syntax@0.8.2
cargo update -p icu_provider --precise 1.5.0
cargo update -p icu_provider_adapters --precise 1.5.0
cargo update -p icu_collections --precise 1.5.0

# Check if the fixes worked
echo -e "${BLUE}Verifying dependency fixes...${NC}"
cargo check

if [ $? -eq 0 ]; then
    echo -e "${GREEN}${BOLD}✓ All dependency issues fixed successfully!${NC}"
    echo -e "${GREEN}You can now build xKippo-tui with: ./scripts/install.sh${NC}"
else
    echo -e "${RED}Some dependency issues still remain.${NC}"
    echo -e "${YELLOW}Attempting additional aggressive fixes...${NC}"
    
    # Create a new Cargo.toml with aggressive version pinning
    cat > Cargo.toml << EOL
# Forcefully modified by fix_deps.sh for Rust 1.75.0 compatibility

$(cat Cargo.toml.bak | grep -A 3 '^\[package\]')

[dependencies]
# TUI Libraries
ratatui = "=0.23.0"  # TUI framework
crossterm = "=0.26.0"  # Terminal backend
tui-textarea = "=0.2.0"  # Text input widget

# Async Runtime
tokio = { version = "=1.28.0", features = ["full"] }

# Configuration
serde = { version = "=1.0.188", features = ["derive"] }
toml = "=0.7.0"
dirs = "=4.0.0"  # Find config directories

# Logging and errors
log = "=0.4.20"
env_logger = "=0.10.0"
thiserror = "=1.0.48"
anyhow = "=1.0.75"

# Time handling
chrono = { version = "=0.4.26", features = ["serde"] }

# Data processing
serde_json = "=1.0.99"
sqlx = { version = "=0.7.2", features = ["runtime-tokio-native-tls", "sqlite", "json", "chrono"], optional = true }
rusqlite = { version = "=0.29.0", features = ["bundled"], optional = true }
csv = "=1.2.2"

# File system
notify = "=6.1.1"  # File system notifications
glob = "=0.3.1"  # File pattern matching

# Networking and APIs
reqwest = { version = "=0.11.22", features = ["json"], optional = true }
maxminddb = { version = "=0.23.0", optional = true }
ipnetwork = "=0.20.0"  # IP address handling

# Utilities
lazy_static = "=1.4.0"
parking_lot = "=0.12.1"  # Improved mutex implementations
dashmap = "=5.5.3"  # Concurrent hashmap
rayon = "=1.7.0"  # Parallel iterators
rayon-core = "=1.11.0"  # Pinned to version compatible with rustc 1.75.0
regex = "=1.9.5"
regex-syntax = "=0.8.2"
clap = { version = "=4.4.6", features = ["derive"] }
indicatif = "=0.17.7"  # Progress bars
ctrlc = { version = "=3.4.1", features = ["termination"] }
indexmap = { version = "=2.0.2", features = ["serde"] }  # Ordered maps
uuid = { version = "=1.4.1", features = ["v4", "serde"] }  # UUID generation

# Session replay
termion = "=2.0.1"  # Terminal manipulation

# Explicitly downgraded for Rust 1.75.0 compatibility
icu_normalizer_data = "=1.5.0"
icu_normalizer = "=1.5.0"
icu_provider = "=1.5.0"
icu_provider_adapters = "=1.5.0" 
icu_collections = "=1.5.0"
icu_locale_core = "=1.5.0"
idna_adapter = "=1.2.1"
idna = "=1.1.0"

$(cat Cargo.toml.bak | grep -A 100 '^\[features\]')
EOL

    echo -e "${BLUE}Applied aggressive version pinning to all dependencies${NC}"
    echo -e "${BLUE}Updating dependencies with aggressively pinned versions...${NC}"
    cargo clean
    cargo update
    cargo check
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}${BOLD}✓ Aggressive version pinning succeeded!${NC}"
        echo -e "${GREEN}You can now build xKippo-tui with: ./scripts/install.sh${NC}"
    else
        echo -e "${RED}Unable to automatically fix all dependency issues.${NC}"
        echo -e "${YELLOW}Please restore your backup from Cargo.toml.bak if needed.${NC}"
        exit 1
    fi
fi

echo -e "\n${BLUE}${BOLD}What's next?${NC}"
echo -e "1. You can build xKippo-tui with: ./scripts/install.sh"
echo -e "2. If issues persist, try building with: cargo build --release --locked"
echo -e "3. If you encounter errors, restore Cargo.toml.bak: cp Cargo.toml.bak Cargo.toml"

exit 0