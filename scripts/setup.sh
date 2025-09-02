#!/bin/bash

# xKippo-tui setup script
# This script helps you set up xKippo-tui to monitor your Cowrie honeypot.

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Config paths
CONFIG_DIR="$HOME/.config/xkippo"
CONFIG_FILE="$CONFIG_DIR/config.toml"
DATA_DIR="$HOME/.local/share/xkippo"
GEOIP_DIR="$DATA_DIR/geoip"

# Default paths to check for Cowrie installation
COWRIE_PATHS=(
  "/opt/cowrie"
  "/usr/local/cowrie"
  "/home/$USER/cowrie"
  "/var/lib/cowrie"
)

LOG_PATHS=(
  "/var/log/cowrie/cowrie.json"
  "/opt/cowrie/var/log/cowrie/cowrie.json"
  "/home/$USER/cowrie/var/log/cowrie/cowrie.json"
  "/usr/local/cowrie/var/log/cowrie/cowrie.json"
)

# Banner
echo -e "${BLUE}"
echo "╔═╗╦╔═╦╔═╦╔═╗╔═╗╔═╗   ╔╦╗╦ ╦╦"
echo "╚═╗╠╩╗╠╩╗║╠═╝╠═╝║ ║    ║ ║ ║║"
echo "╚═╝╩ ╩╩ ╩╩╩  ╩  ╚═╝    ╩ ╚═╝╩"
echo -e "${NC}"
echo "Cowrie Honeypot TUI Monitor - Setup Script"
echo

# Check if config directory exists, create if needed
if [ ! -d "$CONFIG_DIR" ]; then
  echo -e "${YELLOW}Creating configuration directory at $CONFIG_DIR${NC}"
  mkdir -p "$CONFIG_DIR"
fi

# Check if data directory exists, create if needed
if [ ! -d "$DATA_DIR" ]; then
  echo -e "${YELLOW}Creating data directory at $DATA_DIR${NC}"
  mkdir -p "$DATA_DIR"
fi

# Check if GeoIP directory exists, create if needed
if [ ! -d "$GEOIP_DIR" ]; then
  echo -e "${YELLOW}Creating GeoIP directory at $GEOIP_DIR${NC}"
  mkdir -p "$GEOIP_DIR"
fi

# Function to detect Cowrie installation
detect_cowrie() {
  echo -e "${BLUE}Detecting Cowrie installation...${NC}"
  
  for path in "${COWRIE_PATHS[@]}"; do
    if [ -d "$path" ] && [ -f "$path/bin/cowrie" ]; then
      echo -e "${GREEN}Found Cowrie installation at $path${NC}"
      COWRIE_PATH="$path"
      return 0
    fi
  done
  
  echo -e "${YELLOW}No Cowrie installation found automatically.${NC}"
  return 1
}

# Function to detect log files
detect_logs() {
  echo -e "${BLUE}Detecting Cowrie log files...${NC}"
  
  FOUND_LOGS=()
  
  for path in "${LOG_PATHS[@]}"; do
    if [ -f "$path" ]; then
      echo -e "${GREEN}Found log file at $path${NC}"
      FOUND_LOGS+=("$path")
    fi
  done
  
  if [ ${#FOUND_LOGS[@]} -eq 0 ]; then
    echo -e "${YELLOW}No Cowrie log files found automatically.${NC}"
    return 1
  fi
  
  return 0
}

# Function to ask for custom log path
ask_log_path() {
  read -p "Enter the path to your Cowrie log file: " custom_path
  
  if [ -f "$custom_path" ]; then
    FOUND_LOGS+=("$custom_path")
    echo -e "${GREEN}Log file added.${NC}"
    return 0
  else
    echo -e "${RED}File not found at $custom_path${NC}"
    return 1
  fi
}

# Function to generate config
generate_config() {
  echo -e "${BLUE}Generating configuration...${NC}"
  
  # Start with the base config template
  cp "$(dirname "$0")/../config.toml" "$CONFIG_FILE"
  
  # Add detected log paths to config
  if [ ${#FOUND_LOGS[@]} -gt 0 ]; then
    echo "log_paths = [" >> "$CONFIG_FILE.tmp"
    for log_path in "${FOUND_LOGS[@]}"; do
      echo "  \"$log_path\"," >> "$CONFIG_FILE.tmp"
    done
    echo "]" >> "$CONFIG_FILE.tmp"
    
    # Replace placeholder in config file
    sed -i.bak -e '/# log_paths = \[/r '"$CONFIG_FILE.tmp" "$CONFIG_FILE"
    rm "$CONFIG_FILE.tmp" "$CONFIG_FILE.bak"
  fi
  
  echo -e "${GREEN}Configuration file created at $CONFIG_FILE${NC}"
}

# Main setup flow
echo "Welcome to xKippo-tui setup."
echo "This script will help you configure xKippo-tui to monitor your Cowrie honeypot."
echo

# Try to detect Cowrie installation
detect_cowrie

# Try to detect log files
detect_logs

# If no logs found, ask for manual path
if [ ${#FOUND_LOGS[@]} -eq 0 ]; then
  echo "No Cowrie log files were detected automatically."
  echo "Please enter the path to your Cowrie log file manually."
  
  while true; do
    if ask_log_path; then
      break
    fi
    
    read -p "Try again? (y/n): " try_again
    if [[ ! $try_again =~ ^[Yy] ]]; then
      echo -e "${RED}Setup cannot continue without log files.${NC}"
      exit 1
    fi
  done
fi

# Ask about downloading GeoIP database
echo
echo "xKippo-tui can use MaxMind's GeoLite2 database for IP geolocation."
read -p "Would you like to download the GeoLite2 database now? (y/n): " download_geoip

if [[ $download_geoip =~ ^[Yy] ]]; then
  echo -e "${YELLOW}Note: You need a MaxMind license key to download the database.${NC}"
  echo "If you don't have one, you can sign up for free at https://dev.maxmind.com/geoip/geolite2-free-geolocation-data"
  
  read -p "Enter your MaxMind license key: " license_key
  
  if [ -n "$license_key" ]; then
    echo -e "${BLUE}Downloading GeoLite2 City database...${NC}"
    curl -s "https://download.maxmind.com/app/geoip_download?edition_id=GeoLite2-City&license_key=$license_key&suffix=tar.gz" -o "$GEOIP_DIR/GeoLite2-City.tar.gz"
    
    if [ $? -eq 0 ] && [ -f "$GEOIP_DIR/GeoLite2-City.tar.gz" ]; then
      tar -xzf "$GEOIP_DIR/GeoLite2-City.tar.gz" -C "$GEOIP_DIR"
      DB_FILE=$(find "$GEOIP_DIR" -name "*.mmdb" | head -n 1)
      
      if [ -n "$DB_FILE" ]; then
        cp "$DB_FILE" "$GEOIP_DIR/GeoLite2-City.mmdb"
        echo -e "${GREEN}GeoIP database downloaded and installed.${NC}"
        
        # Add to config
        sed -i.bak -e 's/# database_path = .*/database_path = "'"$GEOIP_DIR\/GeoLite2-City.mmdb"'"/' "$CONFIG_FILE"
        sed -i.bak -e 's/# license_key = .*/license_key = "'"$license_key"'"/' "$CONFIG_FILE"
        rm -f "$CONFIG_FILE.bak"
      else
        echo -e "${RED}Failed to extract GeoIP database.${NC}"
      fi
      
      # Clean up
      rm -f "$GEOIP_DIR/GeoLite2-City.tar.gz"
      rm -rf "$GEOIP_DIR/GeoLite2-City_"*
    else
      echo -e "${RED}Failed to download GeoIP database.${NC}"
    fi
  else
    echo -e "${YELLOW}No license key provided, skipping GeoIP database download.${NC}"
  fi
fi

# Generate config
generate_config

echo
echo -e "${GREEN}Setup completed successfully!${NC}"
echo "You can now run xKippo-tui with:"
echo "  xkippo-tui"
echo
echo "Or specify a custom config file:"
echo "  xkippo-tui -c $CONFIG_FILE"
echo

exit 0