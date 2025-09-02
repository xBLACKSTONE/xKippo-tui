#!/bin/bash

# xKippo-tui setup script
# Advanced setup script for Cowrie honeypot TUI monitor with security analyst features

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Config paths
CONFIG_DIR="$HOME/.config/xkippo"
CONFIG_FILE="$CONFIG_DIR/config.toml"
DATA_DIR="$HOME/.local/share/xkippo"
GEOIP_DIR="$DATA_DIR/geoip"
THREAT_INTEL_DIR="$DATA_DIR/threat_intel"
RULES_DIR="$DATA_DIR/rules"
BACKUPS_DIR="$DATA_DIR/backups"

# Default paths to check for Cowrie installation
COWRIE_PATHS=(
  "/opt/cowrie"
  "/usr/local/cowrie"
  "/home/$USER/cowrie"
  "/var/lib/cowrie"
  "/data/cowrie"
  "/srv/cowrie"
  "/usr/share/cowrie"
)

# Enhanced log paths to check for Cowrie logs
LOG_PATHS=(
  # Standard JSON logs
  "/var/log/cowrie/cowrie.json"
  "/opt/cowrie/var/log/cowrie/cowrie.json"
  "/home/$USER/cowrie/var/log/cowrie/cowrie.json"
  "/usr/local/cowrie/var/log/cowrie/cowrie.json"
  "/data/cowrie/var/log/cowrie/cowrie.json"
  "/srv/cowrie/var/log/cowrie/cowrie.json"
  
  # Rotated logs
  "/var/log/cowrie/cowrie.json.*"
  "/opt/cowrie/var/log/cowrie/cowrie.json.*"
  
  # Text logs
  "/var/log/cowrie/audit.log"
  "/opt/cowrie/var/log/cowrie/audit.log"
  "/home/$USER/cowrie/var/log/cowrie/audit.log"
  "/usr/local/cowrie/var/log/cowrie/audit.log"
)

# TTY log paths to check
TTY_LOG_PATHS=(
  "/var/log/cowrie/tty"
  "/opt/cowrie/var/log/cowrie/tty"
  "/home/$USER/cowrie/var/log/cowrie/tty"
  "/usr/local/cowrie/var/log/cowrie/tty"
)

# Download paths to check
DOWNLOAD_PATHS=(
  "/var/lib/cowrie/downloads"
  "/opt/cowrie/var/lib/cowrie/downloads"
  "/home/$USER/cowrie/var/lib/cowrie/downloads"
  "/usr/local/cowrie/var/lib/cowrie/downloads"
)

# Cross-platform sed function
# Usage: safe_sed "search" "replace" "file"
safe_sed() {
  local search=$1
  local replace=$2
  local file=$3
  
  if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS version
    sed -i '' "s|$search|$replace|" "$file"
  else
    # Linux version
    sed -i "s|$search|$replace|" "$file"
  fi
}

# Banner
echo -e "${BLUE}"
echo "╔═╗╦╔═╦╔═╦╔═╗╔═╗╔═╗   ╔╦╗╦ ╦╦"
echo "╚═╗╠╩╗╠╩╗║╠═╝╠═╝║ ║    ║ ║ ║║"
echo "╚═╝╩ ╩╩ ╩╩╩  ╩  ╚═╝    ╩ ╚═╝╩"
echo -e "${NC}"
echo -e "${BOLD}Cowrie Honeypot TUI Monitor - Advanced Security Analyst Setup${NC}"
echo -e "Version: 0.1.0\n"

# Create required directories
create_directories() {
  echo -e "${BLUE}Creating required directories...${NC}"
  
  for dir in "$CONFIG_DIR" "$DATA_DIR" "$GEOIP_DIR" "$THREAT_INTEL_DIR" "$RULES_DIR" "$BACKUPS_DIR"; do
    if [ ! -d "$dir" ]; then
      echo -e "${YELLOW}Creating directory: $dir${NC}"
      mkdir -p "$dir"
    fi
  done
}

# Function to detect Cowrie installation with enhanced diagnostics
detect_cowrie() {
  echo -e "\n${BLUE}${BOLD}[1/7] Detecting Cowrie installation...${NC}"
  
  COWRIE_PATH=""
  COWRIE_VERSION=""
  
  for path in "${COWRIE_PATHS[@]}"; do
    if [ -d "$path" ]; then
      if [ -f "$path/bin/cowrie" ]; then
        COWRIE_PATH="$path"
        echo -e "${GREEN}✓ Found Cowrie installation at $path${NC}"
        
        # Try to detect version
        if [ -f "$path/cowrie.cfg" ]; then
          # Check if we can find version in config or elsewhere
          if grep -q "version" "$path/cowrie.cfg"; then
            COWRIE_VERSION=$(grep "version" "$path/cowrie.cfg" | head -n 1 | cut -d "=" -f 2 | tr -d '[:space:]')
            echo -e "${GREEN}✓ Detected Cowrie version: $COWRIE_VERSION${NC}"
          fi
        elif [ -d "$path/.git" ] && command -v git >/dev/null 2>&1; then
          # Try to get version from git
          pushd "$path" >/dev/null
          COWRIE_VERSION=$(git describe --tags 2>/dev/null || echo "Unknown")
          popd >/dev/null
          echo -e "${GREEN}✓ Detected Cowrie version (git): $COWRIE_VERSION${NC}"
        fi
        
        # Check if Cowrie is running
        if pgrep -f "cowrie" >/dev/null; then
          echo -e "${GREEN}✓ Cowrie appears to be running${NC}"
        else
          echo -e "${YELLOW}⚠ Cowrie does not appear to be running${NC}"
        fi
        
        return 0
      elif [ -d "$path/cowrie" ] || [ -f "$path/cowrie.cfg" ]; then
        echo -e "${YELLOW}⚠ Possible Cowrie installation found at $path (but no bin/cowrie script)${NC}"
        read -p "Use this path anyway? (y/n): " use_anyway
        if [[ $use_anyway =~ ^[Yy] ]]; then
          COWRIE_PATH="$path"
          echo -e "${GREEN}✓ Using Cowrie installation at $path${NC}"
          return 0
        fi
      fi
    fi
  done
  
  if [ -z "$COWRIE_PATH" ]; then
    echo -e "${YELLOW}⚠ No Cowrie installation found automatically.${NC}"
    read -p "Do you want to specify a custom Cowrie installation path? (y/n): " specify_path
    
    if [[ $specify_path =~ ^[Yy] ]]; then
      read -p "Enter the path to your Cowrie installation: " custom_path
      if [ -d "$custom_path" ]; then
        COWRIE_PATH="$custom_path"
        echo -e "${GREEN}✓ Using custom Cowrie path: $COWRIE_PATH${NC}"
        return 0
      else
        echo -e "${RED}✗ Directory not found: $custom_path${NC}"
      fi
    fi
    
    echo -e "${YELLOW}⚠ No Cowrie installation path set. Some features may be limited.${NC}"
  fi
  
  return 1
}

# Function to detect log files with enhanced diagnostics
detect_logs() {
  echo -e "\n${BLUE}${BOLD}[2/7] Detecting Cowrie log files...${NC}"
  
  FOUND_LOGS=()
  FOUND_TTY_LOGS=""
  FOUND_DOWNLOADS=""
  
  # First check standard paths
  for path in "${LOG_PATHS[@]}"; do
    # Handle glob patterns
    if [[ $path == *"*"* ]]; then
      base_path=$(echo "$path" | sed 's/\*.*$//')
      if [ -d "$(dirname "$base_path")" ]; then
        # Use find to look for matching files
        matching_files=$(find "$(dirname "$base_path")" -name "$(basename "$path")" -type f 2>/dev/null)
        if [ -n "$matching_files" ]; then
          echo -e "${GREEN}✓ Found log files matching pattern $path${NC}"
          while IFS= read -r file; do
            if [ -f "$file" ]; then
              echo -e "  - $file"
              FOUND_LOGS+=("$file")
            fi
          done <<< "$matching_files"
        fi
      fi
    elif [ -f "$path" ]; then
      # Check if file is a proper Cowrie log (contains eventid field for JSON logs)
      if head -n 20 "$path" | grep -q '"eventid":' || head -n 20 "$path" | grep -q "cowrie"; then
        echo -e "${GREEN}✓ Found Cowrie log file: $path${NC}"
        
        # Get file size and modification time
        file_size=$(du -h "$path" | cut -f1)
        file_mod=$(stat -c %y "$path" 2>/dev/null || stat -f "%Sm" "$path" 2>/dev/null)
        echo -e "  - Size: $file_size, Last modified: $file_mod"
        
        # Check if logs are being actively written
        last_line_count=$(wc -l < "$path")
        sleep 1
        new_line_count=$(wc -l < "$path")
        if [ "$last_line_count" -lt "$new_line_count" ]; then
          echo -e "  - ${GREEN}✓ Log is actively being written to${NC}"
        else
          echo -e "  - ${YELLOW}⚠ Log does not appear to be actively updated${NC}"
        fi
        
        FOUND_LOGS+=("$path")
      else
        echo -e "${YELLOW}⚠ Found file at $path but it doesn't appear to be a Cowrie log${NC}"
      fi
    fi
  done
  
  # Look for TTY logs directory
  for path in "${TTY_LOG_PATHS[@]}"; do
    if [ -d "$path" ] && [ "$(ls -A "$path" 2>/dev/null)" ]; then
      echo -e "${GREEN}✓ Found TTY logs directory: $path${NC}"
      FOUND_TTY_LOGS="$path"
      tty_count=$(find "$path" -type f | wc -l)
      echo -e "  - Contains approximately $tty_count TTY log files"
      break
    fi
  done
  
  # Look for downloads directory
  for path in "${DOWNLOAD_PATHS[@]}"; do
    if [ -d "$path" ] && [ "$(ls -A "$path" 2>/dev/null)" ]; then
      echo -e "${GREEN}✓ Found downloads directory: $path${NC}"
      FOUND_DOWNLOADS="$path"
      download_count=$(find "$path" -type f | wc -l)
      echo -e "  - Contains approximately $download_count downloaded files"
      break
    fi
  done
  
  if [ ${#FOUND_LOGS[@]} -eq 0 ]; then
    echo -e "${YELLOW}⚠ No Cowrie log files found automatically.${NC}"
    return 1
  fi
  
  return 0
}

# Function to ask for custom log path with validation
ask_log_path() {
  echo -e "\n${BLUE}Specify custom log locations:${NC}"
  echo -e "${YELLOW}Enter path to log file (leave blank to finish):${NC}"
  
  while true; do
    read -p "> " custom_path
    
    if [ -z "$custom_path" ]; then
      break
    fi
    
    if [ -f "$custom_path" ]; then
      # Validate log file format
      if head -n 20 "$custom_path" | grep -q '"eventid":' || head -n 20 "$custom_path" | grep -q "cowrie"; then
        FOUND_LOGS+=("$custom_path")
        echo -e "${GREEN}✓ Log file added: $custom_path${NC}"
      else
        echo -e "${YELLOW}⚠ File doesn't appear to be a Cowrie log. Add anyway? (y/n):${NC}"
        read -p "> " add_anyway
        if [[ $add_anyway =~ ^[Yy] ]]; then
          FOUND_LOGS+=("$custom_path")
          echo -e "${GREEN}✓ Log file added: $custom_path${NC}"
        fi
      fi
    else
      echo -e "${RED}✗ File not found: $custom_path${NC}"
    fi
  done
  
  if [ ${#FOUND_LOGS[@]} -eq 0 ]; then
    return 1
  fi
  
  return 0
}

# Function to set up security analyst specific configurations
setup_security_features() {
  echo -e "\n${BLUE}${BOLD}[3/7] Configuring security analyst features...${NC}"
  
  # Set up threat intelligence feeds
  echo -e "\n${CYAN}Would you like to set up threat intelligence feeds? (y/n):${NC}"
  read -p "> " setup_ti
  
  TI_ENABLED="false"
  TI_FEEDS=()
  
  if [[ $setup_ti =~ ^[Yy] ]]; then
    TI_ENABLED="true"
    echo -e "${GREEN}✓ Threat intelligence enabled${NC}"
    
    # Default threat intelligence feeds
    default_feeds=(
      "https://reputation.alienvault.com/reputation.data"
      "https://www.binarydefense.com/banlist.txt"
      "https://raw.githubusercontent.com/firehol/blocklist-ipsets/master/firehol_level1.netset"
      "https://raw.githubusercontent.com/stamparm/ipsum/master/ipsum.txt"
    )
    
    echo -e "${CYAN}Default threat intelligence feeds:${NC}"
    for i in "${!default_feeds[@]}"; do
      echo -e "$((i+1)). ${default_feeds[$i]}"
    done
    
    echo -e "${CYAN}Include default feeds? (y/n):${NC}"
    read -p "> " include_default
    
    if [[ $include_default =~ ^[Yy] ]]; then
      TI_FEEDS=("${default_feeds[@]}")
    fi
    
    echo -e "${CYAN}Add custom threat intelligence feeds (leave blank to finish):${NC}"
    while true; do
      read -p "Feed URL> " custom_feed
      
      if [ -z "$custom_feed" ]; then
        break
      fi
      
      TI_FEEDS+=("$custom_feed")
      echo -e "${GREEN}✓ Custom feed added${NC}"
    done
  else
    echo -e "${YELLOW}⚠ Threat intelligence disabled${NC}"
  fi
  
  # Configure alert rules
  echo -e "\n${CYAN}Configure alert rules? (y/n):${NC}"
  read -p "> " setup_alerts
  
  ALERTS_ENABLED="true"
  ALERT_COMMANDS=()
  ALERT_IPS=()
  
  if [[ $setup_alerts =~ ^[Yy] ]]; then
    echo -e "${CYAN}Alert on specific commands? (y/n):${NC}"
    read -p "> " alert_commands
    
    if [[ $alert_commands =~ ^[Yy] ]]; then
      echo -e "${GREEN}Default suspicious commands to alert on:${NC}"
      default_commands=("wget" "curl" "tftp" "chmod +x" "dd" "busybox" "python -c" "perl -e" "./" "bash -i" "sh -i" "nc -e" "rm -rf")
      
      for cmd in "${default_commands[@]}"; do
        echo -e "- $cmd"
      done
      
      echo -e "${CYAN}Include these default commands? (y/n):${NC}"
      read -p "> " include_default_commands
      
      if [[ $include_default_commands =~ ^[Yy] ]]; then
        ALERT_COMMANDS=("${default_commands[@]}")
      fi
      
      echo -e "${CYAN}Add additional commands to alert on (leave blank to finish):${NC}"
      while true; do
        read -p "Command> " custom_command
        
        if [ -z "$custom_command" ]; then
          break
        fi
        
        ALERT_COMMANDS+=("$custom_command")
        echo -e "${GREEN}✓ Alert command added: $custom_command${NC}"
      done
    fi
    
    echo -e "${CYAN}Add IP addresses to blacklist or whitelist? (b/w/n):${NC}"
    read -p "> " ip_list_type
    
    if [[ $ip_list_type =~ ^[Bb] ]]; then
      echo -e "${CYAN}Enter IP addresses to blacklist (leave blank to finish):${NC}"
      while true; do
        read -p "IP> " ip_address
        
        if [ -z "$ip_address" ]; then
          break
        fi
        
        # Simple IP validation
        if [[ $ip_address =~ ^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
          ALERT_IPS+=("$ip_address")
          echo -e "${GREEN}✓ IP added to blacklist: $ip_address${NC}"
        else
          echo -e "${RED}✗ Invalid IP address format${NC}"
        fi
      done
    fi
  fi
  
  # Configure malware analysis
  echo -e "\n${CYAN}Enable basic malware analysis for downloaded files? (y/n):${NC}"
  read -p "> " malware_analysis
  
  MALWARE_ANALYSIS="false"
  
  if [[ $malware_analysis =~ ^[Yy] ]]; then
    MALWARE_ANALYSIS="true"
    echo -e "${GREEN}✓ Malware analysis enabled${NC}"
    
    # Check for analysis tools
    if command -v file >/dev/null && command -v strings >/dev/null; then
      echo -e "${GREEN}✓ Basic analysis tools detected (file, strings)${NC}"
    else
      echo -e "${YELLOW}⚠ Some analysis tools may be missing${NC}"
    fi
    
    # Check for VirusTotal API integration
    echo -e "${CYAN}Do you have a VirusTotal API key for malware scanning? (y/n):${NC}"
    read -p "> " vt_enabled
    
    VT_API_KEY=""
    
    if [[ $vt_enabled =~ ^[Yy] ]]; then
      echo -e "${CYAN}Enter your VirusTotal API key:${NC}"
      read -p "> " VT_API_KEY
      
      if [ -n "$VT_API_KEY" ]; then
        echo -e "${GREEN}✓ VirusTotal API key added${NC}"
      fi
    fi
  else
    echo -e "${YELLOW}⚠ Malware analysis disabled${NC}"
  fi
}

# Function to configure TUI interface
configure_ui() {
  echo -e "\n${BLUE}${BOLD}[4/7] Configuring TUI interface...${NC}"
  
  # Theme selection
  echo -e "${CYAN}Select theme for TUI:${NC}"
  echo "1. Default"
  echo "2. Dark"
  echo "3. Light"
  echo "4. Security-focused (Red/Green)"
  read -p "> " theme_choice
  
  UI_THEME="default"
  case $theme_choice in
    2) UI_THEME="dark";;
    3) UI_THEME="light";;
    4) UI_THEME="security";;
  esac
  
  echo -e "${GREEN}✓ Selected theme: $UI_THEME${NC}"
  
  # Dashboard layout
  echo -e "\n${CYAN}Configure default dashboard layout:${NC}"
  echo "1. Standard (Overview, Logs, Sessions)"
  echo "2. Security focus (Attack map, Sessions, Alerts)"
  echo "3. Analytics focus (Statistics, Charts, Patterns)"
  read -p "> " layout_choice
  
  LAYOUT="standard"
  case $layout_choice in
    2) LAYOUT="security";;
    3) LAYOUT="analytics";;
  esac
  
  echo -e "${GREEN}✓ Selected layout: $LAYOUT${NC}"
}

# Function to configure GeoIP database
configure_geoip() {
  echo -e "\n${BLUE}${BOLD}[5/7] Configuring GeoIP database...${NC}"
  
  echo -e "${CYAN}xKippo-tui can use MaxMind's GeoLite2 database for IP geolocation.${NC}"
  read -p "Would you like to download the GeoLite2 database now? (y/n): " download_geoip
  
  GEOIP_ENABLED="false"
  LICENSE_KEY=""
  
  if [[ $download_geoip =~ ^[Yy] ]]; then
    GEOIP_ENABLED="true"
    echo -e "${YELLOW}Note: You need a MaxMind license key to download the database.${NC}"
    echo -e "${YELLOW}If you don't have one, you can sign up for free at:${NC}"
    echo -e "${CYAN}https://dev.maxmind.com/geoip/geolite2-free-geolocation-data${NC}"
    
    read -p "Enter your MaxMind license key: " LICENSE_KEY
    
    if [ -n "$LICENSE_KEY" ]; then
      echo -e "${BLUE}Downloading GeoLite2 City database...${NC}"
      curl -s "https://download.maxmind.com/app/geoip_download?edition_id=GeoLite2-City&license_key=$LICENSE_KEY&suffix=tar.gz" -o "$GEOIP_DIR/GeoLite2-City.tar.gz"
      
      if [ $? -eq 0 ] && [ -f "$GEOIP_DIR/GeoLite2-City.tar.gz" ]; then
        tar -xzf "$GEOIP_DIR/GeoLite2-City.tar.gz" -C "$GEOIP_DIR"
        DB_FILE=$(find "$GEOIP_DIR" -name "*.mmdb" | head -n 1)
        
        if [ -n "$DB_FILE" ]; then
          cp "$DB_FILE" "$GEOIP_DIR/GeoLite2-City.mmdb"
          echo -e "${GREEN}✓ GeoIP database downloaded and installed.${NC}"
          
          # Clean up
          rm -f "$GEOIP_DIR/GeoLite2-City.tar.gz"
          rm -rf "$GEOIP_DIR/GeoLite2-City_"*
        else
          echo -e "${RED}✗ Failed to extract GeoIP database.${NC}"
          GEOIP_ENABLED="false"
        fi
      else
        echo -e "${RED}✗ Failed to download GeoIP database.${NC}"
        GEOIP_ENABLED="false"
      fi
    else
      echo -e "${YELLOW}⚠ No license key provided, skipping GeoIP database download.${NC}"
      GEOIP_ENABLED="false"
    fi
  else
    echo -e "${YELLOW}⚠ Skipping GeoIP database configuration.${NC}"
  fi
}

# Function to configure advanced logging and export options
configure_advanced_options() {
  echo -e "\n${BLUE}${BOLD}[6/7] Configuring advanced options...${NC}"
  
  # Log retention
  echo -e "${CYAN}Configure log retention policy:${NC}"
  echo "1. Keep all logs (no limit)"
  echo "2. Keep logs for 30 days"
  echo "3. Keep logs for 90 days"
  echo "4. Custom retention period"
  read -p "> " retention_choice
  
  LOG_RETENTION="0"
  case $retention_choice in
    2) LOG_RETENTION="30";;
    3) LOG_RETENTION="90";;
    4) 
      read -p "Enter number of days to retain logs: " custom_days
      if [[ "$custom_days" =~ ^[0-9]+$ ]]; then
        LOG_RETENTION="$custom_days"
      else
        echo -e "${YELLOW}⚠ Invalid input, defaulting to no limit${NC}"
      fi
      ;;
  esac
  
  echo -e "${GREEN}✓ Log retention set to $LOG_RETENTION days (0 = unlimited)${NC}"
  
  # Export formats
  echo -e "\n${CYAN}Enable data export capabilities? (y/n):${NC}"
  read -p "> " enable_export
  
  EXPORT_FORMATS=()
  
  if [[ $enable_export =~ ^[Yy] ]]; then
    echo -e "${CYAN}Select export formats (space-separated numbers):${NC}"
    echo "1. CSV"
    echo "2. JSON"
    echo "3. HTML"
    echo "4. PDF"
    echo "5. STIX (security threat info)"
    read -p "> " export_choices
    
    if [[ $export_choices =~ 1 ]]; then EXPORT_FORMATS+=("csv"); fi
    if [[ $export_choices =~ 2 ]]; then EXPORT_FORMATS+=("json"); fi
    if [[ $export_choices =~ 3 ]]; then EXPORT_FORMATS+=("html"); fi
    if [[ $export_choices =~ 4 ]]; then EXPORT_FORMATS+=("pdf"); fi
    if [[ $export_choices =~ 5 ]]; then EXPORT_FORMATS+=("stix"); fi
    
    if [ ${#EXPORT_FORMATS[@]} -gt 0 ]; then
      echo -e "${GREEN}✓ Enabled export formats: ${EXPORT_FORMATS[*]}${NC}"
    else
      echo -e "${YELLOW}⚠ No export formats selected${NC}"
    fi
  fi
  
  # Configure SIEM integration
  echo -e "\n${CYAN}Configure SIEM integration? (y/n):${NC}"
  read -p "> " siem_integration
  
  SIEM_ENABLED="false"
  SIEM_URL=""
  
  if [[ $siem_integration =~ ^[Yy] ]]; then
    SIEM_ENABLED="true"
    echo -e "${CYAN}Select SIEM platform:${NC}"
    echo "1. ELK Stack"
    echo "2. Splunk"
    echo "3. Graylog"
    echo "4. Custom endpoint"
    read -p "> " siem_choice
    
    case $siem_choice in
      1) SIEM_TYPE="elk";;
      2) SIEM_TYPE="splunk";;
      3) SIEM_TYPE="graylog";;
      4) SIEM_TYPE="custom";;
      *) SIEM_TYPE="custom";;
    esac
    
    read -p "Enter SIEM endpoint URL: " SIEM_URL
    
    if [ -n "$SIEM_URL" ]; then
      echo -e "${GREEN}✓ SIEM integration configured: $SIEM_TYPE at $SIEM_URL${NC}"
    else
      echo -e "${YELLOW}⚠ SIEM URL not provided, integration will be disabled${NC}"
      SIEM_ENABLED="false"
    fi
  fi
}

# Function to insert content into file after pattern
# Usage: insert_after_pattern "pattern" "content_file" "target_file"
insert_after_pattern() {
  local pattern=$1
  local content_file=$2
  local target_file=$3
  
  if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS version
    sed -i '' -e "/$pattern/r $content_file" "$target_file"
  else
    # Linux version
    sed -i -e "/$pattern/r $content_file" "$target_file"
  fi
}

# Function to generate an enhanced configuration file
generate_config() {
  echo -e "\n${BLUE}${BOLD}[7/7] Generating configuration file...${NC}"
  
  # Start with the base config template
  cp "$(dirname "$0")/../config.toml" "$CONFIG_FILE.new"
  
  # Update honeypot section
  safe_sed 'name = "Cowrie Honeypot"' 'name = "Cowrie Security Monitor"' "$CONFIG_FILE.new"
  safe_sed 'auto_detect = true' 'auto_detect = false' "$CONFIG_FILE.new"
  
  # Add detected log paths
  if [ ${#FOUND_LOGS[@]} -gt 0 ]; then
    echo "log_paths = [" >> "$CONFIG_FILE.tmp"
    for log_path in "${FOUND_LOGS[@]}"; do
      echo "  \"$log_path\"," >> "$CONFIG_FILE.tmp"
    done
    echo "]" >> "$CONFIG_FILE.tmp"
    
    # Replace placeholder in config file
    insert_after_pattern '# log_paths = \[' "$CONFIG_FILE.tmp" "$CONFIG_FILE.new"
    rm "$CONFIG_FILE.tmp"
  fi
  
  # Add TTY logs path if found
  if [ -n "$FOUND_TTY_LOGS" ]; then
    safe_sed '# tty_log_path = .*' "tty_log_path = \"$FOUND_TTY_LOGS\"" "$CONFIG_FILE.new"
  fi
  
  # Add downloads path if found
  if [ -n "$FOUND_DOWNLOADS" ]; then
    safe_sed '# download_path = .*' "download_path = \"$FOUND_DOWNLOADS\"" "$CONFIG_FILE.new"
  fi
  
  # Update UI section
  safe_sed 'theme = "default"' "theme = \"$UI_THEME\"" "$CONFIG_FILE.new"
  safe_sed 'terminal_title = "xKippo - Honeypot Monitor"' 'terminal_title = "xKippo - Honeypot Security Analyst Platform"' "$CONFIG_FILE.new"
  
  # Update alert section with security analyst settings
  if [ ${#ALERT_COMMANDS[@]} -gt 0 ]; then
    echo "on_commands = [" >> "$CONFIG_FILE.cmd.tmp"
    for cmd in "${ALERT_COMMANDS[@]}"; do
      echo "  \"$cmd\"," >> "$CONFIG_FILE.cmd.tmp"
    done
    echo "]" >> "$CONFIG_FILE.cmd.tmp"
    
    # Replace placeholder in config file
    insert_after_pattern '# on_commands = \[' "$CONFIG_FILE.cmd.tmp" "$CONFIG_FILE.new"
    rm "$CONFIG_FILE.cmd.tmp"
  fi
  
  if [ ${#ALERT_IPS[@]} -gt 0 ]; then
    echo "ip_blacklist = [" >> "$CONFIG_FILE.ip.tmp"
    for ip in "${ALERT_IPS[@]}"; do
      echo "  \"$ip\"," >> "$CONFIG_FILE.ip.tmp"
    done
    echo "]" >> "$CONFIG_FILE.ip.tmp"
    
    # Replace placeholder in config file
    insert_after_pattern '# ip_blacklist = \[' "$CONFIG_FILE.ip.tmp" "$CONFIG_FILE.new"
    rm "$CONFIG_FILE.ip.tmp"
  fi
  
  # Update GeoIP section
  safe_sed 'enabled = true' "enabled = $GEOIP_ENABLED" "$CONFIG_FILE.new"
  if [ -n "$LICENSE_KEY" ]; then
    safe_sed '# license_key = .*' "license_key = \"$LICENSE_KEY\"" "$CONFIG_FILE.new"
    safe_sed '# database_path = .*' "database_path = \"$GEOIP_DIR/GeoLite2-City.mmdb\"" "$CONFIG_FILE.new"
  fi
  
  # Add security analyst specific sections
  cat >> "$CONFIG_FILE.new" << EOL

# Security Analyst Features
[security_analyst]
# Enable security analyst features
enabled = true
# Log retention in days (0 = unlimited)
log_retention = $LOG_RETENTION

[threat_intel]
# Enable threat intelligence
enabled = $TI_ENABLED
# Auto-update frequency in hours (0 = disable)
update_frequency = 24
# Directory to store threat intelligence data
data_dir = "$THREAT_INTEL_DIR"
EOL

  if [ ${#TI_FEEDS[@]} -gt 0 ]; then
    cat >> "$CONFIG_FILE.new" << EOL
# Threat intelligence feeds
feeds = [
EOL
    for feed in "${TI_FEEDS[@]}"; do
      echo "  \"$feed\"," >> "$CONFIG_FILE.new"
    done
    echo "]" >> "$CONFIG_FILE.new"
  fi
  
  cat >> "$CONFIG_FILE.new" << EOL

[malware_analysis]
# Enable basic malware analysis for downloaded files
enabled = $MALWARE_ANALYSIS
# Maximum file size to analyze (in MB)
max_file_size = 5
# Save analysis reports
save_reports = true
# Analysis report directory
report_dir = "$DATA_DIR/analysis"
EOL

  if [ -n "$VT_API_KEY" ]; then
    cat >> "$CONFIG_FILE.new" << EOL
# VirusTotal API integration
virustotal_enabled = true
virustotal_api_key = "$VT_API_KEY"
EOL
  else
    cat >> "$CONFIG_FILE.new" << EOL
# VirusTotal API integration
virustotal_enabled = false
virustotal_api_key = ""
EOL
  fi
  
  # Add SIEM integration if enabled
  if [ "$SIEM_ENABLED" = "true" ]; then
    cat >> "$CONFIG_FILE.new" << EOL

[siem_integration]
# Enable SIEM integration
enabled = true
# SIEM type (elk, splunk, graylog, custom)
siem_type = "$SIEM_TYPE"
# SIEM endpoint URL
siem_url = "$SIEM_URL"
# Authentication token (if needed)
auth_token = ""
# Batch size for sending events
batch_size = 100
# Send interval in seconds
send_interval = 30
EOL
  fi
  
  # Add export configuration
  if [ ${#EXPORT_FORMATS[@]} -gt 0 ]; then
    cat >> "$CONFIG_FILE.new" << EOL

[export]
# Enable data export
enabled = true
# Available export formats
formats = [
EOL
    for format in "${EXPORT_FORMATS[@]}"; do
      echo "  \"$format\"," >> "$CONFIG_FILE.new"
    done
    echo "]" >> "$CONFIG_FILE.new"
    
    cat >> "$CONFIG_FILE.new" << EOL
# Default export directory
export_dir = "$DATA_DIR/exports"
EOL
  fi
  
  # Add dashboard layout configuration
  cat >> "$CONFIG_FILE.new" << EOL

[dashboard]
# Dashboard layout type
layout = "$LAYOUT"
# Refresh interval in seconds
refresh_interval = 10
# Show attack map
show_map = true
# Show statistics panel
show_stats = true
# Show alerts panel
show_alerts = true
# Show top attackers panel
show_top_attackers = true
# Show command cloud
show_command_cloud = true
EOL
  
  # Add rules configuration
  cat >> "$CONFIG_FILE.new" << EOL

[rules]
# Directory for custom detection rules
rules_dir = "$RULES_DIR"
# Auto-reload rules on change
auto_reload = true
# Enable correlation engine
enable_correlation = true
# Minimum risk score for alerts (0-100)
min_risk_score = 50
# Alert on new attacker IPs
alert_new_ips = true
EOL

  # Backup existing config if present
  if [ -f "$CONFIG_FILE" ]; then
    cp "$CONFIG_FILE" "$BACKUPS_DIR/config.toml.bak.$(date +%Y%m%d%H%M%S)"
    echo -e "${GREEN}✓ Backed up existing configuration${NC}"
  fi
  
  # Move new config into place
  mv "$CONFIG_FILE.new" "$CONFIG_FILE"
  
  echo -e "${GREEN}✓ Enhanced configuration file created at $CONFIG_FILE${NC}"
}

# Main setup flow
echo -e "${MAGENTA}${BOLD}Welcome to the xKippo-tui advanced security analyst setup.${NC}"
echo -e "${MAGENTA}This script will configure xKippo-tui for optimal security monitoring of your Cowrie honeypot.${NC}\n"

# Check for required system dependencies
check_dependencies() {
  echo -e "${BLUE}${BOLD}[0/7] Checking system dependencies...${NC}"
  
  missing_deps=()
  
  # Check for pkg-config
  if ! command -v pkg-config &> /dev/null; then
    missing_deps+=("pkg-config")
  fi
  
  # Check for other essential build tools
  if ! command -v cc &> /dev/null && ! command -v gcc &> /dev/null && ! command -v clang &> /dev/null; then
    missing_deps+=("build-essential or gcc")
  fi
  
  # Check for OpenSSL development packages (often needed for Rust crypto)
  if ! pkg-config --exists openssl 2>/dev/null; then
    missing_deps+=("openssl-dev or libssl-dev")
  fi
  
  if [ ${#missing_deps[@]} -gt 0 ]; then
    echo -e "${YELLOW}⚠ Missing required dependencies: ${missing_deps[*]}${NC}"
    echo -e "Please install them using your distribution's package manager:"
    
    if command -v apt-get &> /dev/null; then
      echo -e "${CYAN}sudo apt-get update${NC}"
      echo -e "${CYAN}sudo apt-get install -y pkg-config build-essential libssl-dev${NC}"
    elif command -v dnf &> /dev/null; then
      echo -e "${CYAN}sudo dnf install -y pkgconfig gcc openssl-devel${NC}"
    elif command -v yum &> /dev/null; then
      echo -e "${CYAN}sudo yum install -y pkgconfig gcc openssl-devel${NC}"
    elif command -v pacman &> /dev/null; then
      echo -e "${CYAN}sudo pacman -S pkg-config base-devel openssl${NC}"
    elif command -v zypper &> /dev/null; then
      echo -e "${CYAN}sudo zypper install pkg-config gcc libopenssl-devel${NC}"
    elif command -v brew &> /dev/null; then
      echo -e "${CYAN}brew install pkg-config openssl${NC}"
    else
      echo -e "${YELLOW}Please install these packages with your package manager:${NC}"
      echo -e "- pkg-config"
      echo -e "- A C compiler (gcc or clang)"
      echo -e "- OpenSSL development headers"
    fi
    
    read -p "Do you want to attempt to install these dependencies automatically? (y/n): " install_deps
    if [[ $install_deps =~ ^[Yy] ]]; then
      if command -v apt-get &> /dev/null; then
        echo -e "${CYAN}Installing dependencies with apt-get...${NC}"
        sudo apt-get update
        sudo apt-get install -y pkg-config build-essential libssl-dev
      elif command -v dnf &> /dev/null; then
        echo -e "${CYAN}Installing dependencies with dnf...${NC}"
        sudo dnf install -y pkgconfig gcc openssl-devel
      elif command -v yum &> /dev/null; then
        echo -e "${CYAN}Installing dependencies with yum...${NC}"
        sudo yum install -y pkgconfig gcc openssl-devel
      elif command -v pacman &> /dev/null; then
        echo -e "${CYAN}Installing dependencies with pacman...${NC}"
        sudo pacman -S --noconfirm pkg-config base-devel openssl
      elif command -v zypper &> /dev/null; then
        echo -e "${CYAN}Installing dependencies with zypper...${NC}"
        sudo zypper install -y pkg-config gcc libopenssl-devel
      elif command -v brew &> /dev/null; then
        echo -e "${CYAN}Installing dependencies with brew...${NC}"
        brew install pkg-config openssl
      else
        echo -e "${RED}✗ Couldn't determine package manager. Please install dependencies manually.${NC}"
        exit 1
      fi
    else
      echo -e "${RED}✗ Required dependencies missing. Please install them and run the setup again.${NC}"
      exit 1
    fi
  else
    echo -e "${GREEN}✓ All required system dependencies are installed.${NC}"
  fi
}

# Check dependencies before proceeding
check_dependencies

# Create required directories
create_directories

# Try to detect Cowrie installation
detect_cowrie

# Try to detect log files
detect_logs

# If no logs found, ask for manual path
if [ ${#FOUND_LOGS[@]} -eq 0 ]; then
  echo -e "${YELLOW}${BOLD}⚠ No Cowrie log files were detected automatically.${NC}"
  echo "Please enter the path to your Cowrie log file manually."
  
  while true; do
    if ask_log_path; then
      break
    fi
    
    read -p "Try again? (y/n): " try_again
    if [[ ! $try_again =~ ^[Yy] ]]; then
      echo -e "${RED}✗ Setup cannot continue without log files.${NC}"
      exit 1
    fi
  done
fi

# Setup security analyst features
setup_security_features

# Configure TUI interface
configure_ui

# Configure GeoIP
configure_geoip

# Configure advanced options
configure_advanced_options

# Generate config
generate_config

echo
echo -e "${GREEN}${BOLD}✅ Setup completed successfully!${NC}"
echo -e "${CYAN}You can now run xKippo-tui with:${NC}"
echo "  xkippo-tui"
echo
echo -e "${CYAN}Or specify a custom config file:${NC}"
echo "  xkippo-tui -c $CONFIG_FILE"
echo
echo -e "${YELLOW}For optimal security analysis capabilities:${NC}"
echo -e "1. Make sure your Cowrie honeypot is properly configured and running"
echo -e "2. Consider setting up regular backups of your logs"
echo -e "3. Review the configuration file manually to fine-tune settings"
echo -e "4. Check for updates to both xKippo-tui and Cowrie regularly"
echo

exit 0