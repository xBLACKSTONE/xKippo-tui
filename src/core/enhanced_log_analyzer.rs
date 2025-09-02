use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use log::{debug, error, trace};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use uuid::Uuid;
use regex::Regex;

use crate::data::{EventType, FileTransfer, FileTransferDirection, LogEntry, Session, Command};
use crate::config::Config;

/// Enhanced analyzer for Cowrie honeypot logs with advanced security analyst features
pub struct EnhancedLogAnalyzer {
    /// Mapping of Cowrie event types to our EventType enum
    event_type_mapping: HashMap<String, EventType>,
    /// Regex for detecting malicious command patterns
    malicious_cmd_patterns: Vec<Regex>,
    /// Known IoC (Indicators of Compromise) IPs
    known_ioc_ips: Vec<String>,
    /// Configuration reference
    config: Config,
    /// Threat intelligence data
    threat_intel: HashMap<String, ThreatIntelData>,
    /// ASN and geographic data cache
    geo_data_cache: HashMap<String, GeoData>,
}

/// Threat intelligence data for an IP address
#[derive(Debug, Clone)]
pub struct ThreatIntelData {
    /// IP address
    pub ip: String,
    /// Threat score (0-100)
    pub score: u8,
    /// Threat classification labels
    pub labels: Vec<String>,
    /// Last seen timestamp
    pub last_seen: Option<DateTime<Utc>>,
    /// First seen timestamp
    pub first_seen: Option<DateTime<Utc>>,
    /// Source of the threat intel
    pub source: String,
}

/// Geographic and ASN data for an IP
#[derive(Debug, Clone)]
pub struct GeoData {
    /// IP address
    pub ip: String,
    /// Country code
    pub country_code: String,
    /// Country name
    pub country_name: String,
    /// City name
    pub city: Option<String>,
    /// Latitude
    pub latitude: Option<f64>,
    /// Longitude
    pub longitude: Option<f64>,
    /// ASN number
    pub asn: Option<String>,
    /// ISP/Organization name
    pub org: Option<String>,
}

impl EnhancedLogAnalyzer {
    /// Create a new enhanced log analyzer
    pub fn new(config: &Config) -> Self {
        let mut event_type_mapping = HashMap::new();
        
        // Map Cowrie event types to our EventType enum
        event_type_mapping.insert("cowrie.session.connect".to_string(), EventType::Connect);
        event_type_mapping.insert("cowrie.session.closed".to_string(), EventType::Disconnect);
        event_type_mapping.insert("cowrie.login.success".to_string(), EventType::LoginSuccess);
        event_type_mapping.insert("cowrie.login.failed".to_string(), EventType::LoginFailed);
        event_type_mapping.insert("cowrie.client.kex".to_string(), EventType::Connect);
        event_type_mapping.insert("cowrie.client.version".to_string(), EventType::Connect);
        event_type_mapping.insert("cowrie.command.input".to_string(), EventType::Command);
        event_type_mapping.insert("cowrie.command.success".to_string(), EventType::Command);
        event_type_mapping.insert("cowrie.session.file_download".to_string(), EventType::FileDownload);
        event_type_mapping.insert("cowrie.session.file_upload".to_string(), EventType::FileUpload);
        event_type_mapping.insert("cowrie.client.fingerprint".to_string(), EventType::KeyAuth);
        event_type_mapping.insert("cowrie.direct-tcpip.request".to_string(), EventType::TCPForward);
        event_type_mapping.insert("cowrie.direct-tcpip.data".to_string(), EventType::TCPForward);
        
        // Set up malicious command pattern detection
        let malicious_cmd_patterns = vec![
            Regex::new(r"wget\s+.+\s+\|\s*sh").unwrap(),          // wget pipe to shell
            Regex::new(r"curl\s+.+\s+\|\s*sh").unwrap(),          // curl pipe to shell
            Regex::new(r"/dev/tcp/\d+\.\d+\.\d+\.\d+/\d+").unwrap(), // bash reverse shell
            Regex::new(r"python\s+-c\s+'(.*socket|.*connect)'").unwrap(), // python reverse shell
            Regex::new(r"nc\s+(-e|-c)\s+").unwrap(),              // netcat reverse shell
            Regex::new(r"busybox\s+tftp").unwrap(),               // busybox tftp download
            Regex::new(r"chmod\s+[+]x").unwrap(),                 // make file executable
            Regex::new(r"dd\s+bs=\d+\s+count=\d+\s+if=/dev/zero").unwrap(), // DoS attack
            Regex::new(r"ping\s+(-f|-t|\-s\s+\d{4,})").unwrap(),  // Ping flood
        ];
        
        // Set up known IoC IPs (empty for now, would be populated from threat intel feeds)
        let known_ioc_ips = Vec::new();
        
        Self {
            event_type_mapping,
            malicious_cmd_patterns,
            known_ioc_ips,
            config: config.clone(),
            threat_intel: HashMap::new(),
            geo_data_cache: HashMap::new(),
        }
    }
    
    /// Load threat intelligence data from feeds
    pub fn load_threat_intel(&mut self) -> Result<()> {
        if !self.config.threat_intel.enabled {
            debug!("Threat intelligence disabled, skipping load");
            return Ok(());
        }
        
        for feed_url in &self.config.threat_intel.feeds {
            debug!("Loading threat intel from feed: {}", feed_url);
            // In a real implementation, this would download and parse the feed
            // For now, just log that we would do it
        }
        
        // Add some example threat intel data for testing
        self.add_example_threat_intel();
        
        debug!("Loaded {} threat intel entries", self.threat_intel.len());
        Ok(())
    }
    
    /// Add example threat intel data for testing
    fn add_example_threat_intel(&mut self) {
        // Add some example threat intel data
        let examples = vec![
            ("185.156.73.54", 85, vec!["scanner", "bruteforce", "malware"], "AbuseIPDB"),
            ("112.85.42.2", 90, vec!["botnet", "c2", "scanner"], "Feodo Tracker"),
            ("45.227.255.206", 75, vec!["ransomware", "malware"], "Blocklist.de"),
            ("193.142.146.78", 60, vec!["scanner", "bruteforce"], "AlienVault"),
        ];
        
        for (ip, score, labels, source) in examples {
            self.threat_intel.insert(ip.to_string(), ThreatIntelData {
                ip: ip.to_string(),
                score: score,
                labels: labels.iter().map(|s| s.to_string()).collect(),
                last_seen: Some(Utc::now()),
                first_seen: Some(Utc::now()),
                source: source.to_string(),
            });
        }
    }
    
    /// Parse a log file and extract log entries
    pub fn parse_log_file(&self, path: &Path) -> Result<Vec<LogEntry>> {
        debug!("Parsing log file: {}", path.display());
        
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();
        
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    if line.trim().is_empty() {
                        continue;
                    }
                    
                    match self.parse_log_entry(&line) {
                        Ok(entry) => entries.push(entry),
                        Err(e) => error!("Failed to parse log entry: {}", e),
                    }
                }
                Err(e) => error!("Error reading line from log file: {}", e),
            }
        }
        
        debug!("Parsed {} log entries from {}", entries.len(), path.display());
        Ok(entries)
    }
    
    /// Parse a JSON log entry into our LogEntry struct with enhanced analysis
    pub fn parse_log_entry(&self, line: &str) -> Result<LogEntry> {
        trace!("Parsing log entry: {}", line);
        
        // Parse JSON
        let value: Value = serde_json::from_str(line)
            .context("Failed to parse log entry as JSON")?;
        
        // Extract basic fields
        let event_type = self.extract_event_type(&value)?;
        let timestamp = self.extract_timestamp(&value)?;
        let session = self.extract_string_field(&value, "session");
        let src_ip = self.extract_string_field(&value, "src_ip");
        let src_port = self.extract_number_field(&value, "src_port").map(|n| n as u16);
        let dst_ip = self.extract_string_field(&value, "dst_ip");
        let dst_port = self.extract_number_field(&value, "dst_port").map(|n| n as u16);
        let username = self.extract_string_field(&value, "username");
        let password = self.extract_string_field(&value, "password");
        let command = self.extract_string_field(&value, "input");
        
        // Generate a unique ID for this log entry
        let id = Uuid::new_v4().to_string();
        
        // Extract additional fields into a HashMap
        let fields = self.extract_additional_fields(&value);
        
        // Extract file information if present
        let file = self.extract_file_info(&value, &event_type);
        
        // Create the log entry
        let entry = LogEntry {
            id,
            timestamp,
            event_type,
            session,
            src_ip,
            src_port,
            dst_ip,
            dst_port,
            username,
            password,
            command,
            file,
            fields,
            raw: value.clone(),
        };
        
        Ok(entry)
    }
    
    /// Extract the event type from a log entry
    fn extract_event_type(&self, value: &Value) -> Result<EventType> {
        let event_name = value["eventid"]
            .as_str()
            .context("Missing or invalid eventid field")?;
        
        let event_type = self.event_type_mapping
            .get(event_name)
            .cloned()
            .unwrap_or(EventType::Unknown);
        
        Ok(event_type)
    }
    
    /// Extract the timestamp from a log entry
    fn extract_timestamp(&self, value: &Value) -> Result<DateTime<Utc>> {
        let timestamp_str = value["timestamp"]
            .as_str()
            .context("Missing or invalid timestamp field")?;
        
        let timestamp = DateTime::parse_from_rfc3339(timestamp_str)
            .context("Failed to parse timestamp")?
            .with_timezone(&Utc);
        
        Ok(timestamp)
    }
    
    /// Extract a string field from a log entry
    fn extract_string_field(&self, value: &Value, field_name: &str) -> Option<String> {
        value[field_name].as_str().map(String::from)
    }
    
    /// Extract a number field from a log entry
    fn extract_number_field(&self, value: &Value, field_name: &str) -> Option<u64> {
        if let Some(num) = value[field_name].as_u64() {
            Some(num)
        } else if let Some(num_str) = value[field_name].as_str() {
            num_str.parse::<u64>().ok()
        } else {
            None
        }
    }
    
    /// Extract additional fields from a log entry
    fn extract_additional_fields(&self, value: &Value) -> HashMap<String, Value> {
        let mut fields = HashMap::new();
        
        if let Some(obj) = value.as_object() {
            for (key, val) in obj {
                // Skip fields we've already extracted
                if !["id", "timestamp", "eventid", "session", "src_ip", "src_port",
                     "dst_ip", "dst_port", "username", "password", "input"].contains(&key.as_str()) {
                    fields.insert(key.clone(), val.clone());
                }
            }
        }
        
        fields
    }
    
    /// Extract file information from a log entry
    fn extract_file_info(&self, value: &Value, event_type: &EventType) -> Option<FileTransfer> {
        match event_type {
            EventType::FileUpload | EventType::FileDownload => {
                let filename = self.extract_string_field(value, "filename")?;
                let outfile = self.extract_string_field(value, "outfile");
                let shasum = self.extract_string_field(value, "shasum");
                
                let direction = match event_type {
                    EventType::FileUpload => FileTransferDirection::Upload,
                    EventType::FileDownload => FileTransferDirection::Download,
                    _ => unreachable!(),
                };
                
                let timestamp = self.extract_timestamp(value).ok()?;
                
                // Determine if file is executable based on name
                let is_executable = filename.ends_with(".sh") || 
                                   filename.ends_with(".bin") ||
                                   filename.ends_with(".elf") ||
                                   filename.ends_with(".exe");
                
                // In a real implementation, we would check if the file is malware
                // by checking the shasum against known malware databases
                let is_malware = shasum.as_ref().map_or(false, |s| {
                    s.starts_with("e") || s.starts_with("a")  // Just for testing
                });
                
                Some(FileTransfer {
                    filename,
                    local_path: outfile,
                    size: None, // Size is often not included in log entries
                    shasum,
                    timestamp,
                    direction,
                    mime_type: None,
                    is_executable,
                    is_malware,
                })
            }
            _ => None,
        }
    }
    
    /// Check if an IP is in the threat intelligence database
    pub fn get_threat_intel(&self, ip: &str) -> Option<&ThreatIntelData> {
        self.threat_intel.get(ip)
    }
    
    /// Check if a command is potentially malicious
    pub fn is_command_malicious(&self, cmd: &str) -> bool {
        // Check if command matches any of our malicious patterns
        self.malicious_cmd_patterns.iter().any(|re| re.is_match(cmd))
    }
    
    /// Analyze a session for potential malicious activity with enhanced detection
    pub fn analyze_session_risk(&self, session: &Session) -> u8 {
        let mut score = 0;
        
        // Check for successful login
        if let Some(user) = &session.user {
            if user.login_success {
                score += 10;
            }
        }
        
        // Check if IP is in threat intel
        if let Some(threat_data) = session.src_ip.as_ref().and_then(|ip| self.get_threat_intel(ip)) {
            // Add a portion of the threat intel score
            score += threat_data.score / 5;
            
            // Add points for certain threat categories
            if threat_data.labels.iter().any(|l| l == "malware" || l == "c2" || l == "botnet") {
                score += 10;
            }
        }
        
        // Check for commands
        if !session.commands.is_empty() {
            score += 5;
            
            // Score for number of commands (more commands = more interaction = higher risk)
            if session.commands.len() > 20 {
                score += 10;
            } else if session.commands.len() > 10 {
                score += 5;
            }
            
            // Check for malicious commands
            for cmd in &session.commands {
                let cmd_lower = cmd.command.to_lowercase();
                
                // Check directly for malicious commands
                if self.is_command_malicious(&cmd.command) {
                    score += 20;
                }
                
                // Check for downloading tools
                if cmd_lower.contains("wget") || cmd_lower.contains("curl") || cmd_lower.contains("tftp") {
                    score += 10;
                }
                
                // Check for common malware paths
                if cmd_lower.contains("/tmp") || cmd_lower.contains("/var/tmp") || cmd_lower.contains("/dev/shm") {
                    score += 5;
                }
                
                // Check for chmod
                if cmd_lower.contains("chmod") && (cmd_lower.contains("+x") || cmd_lower.contains("777")) {
                    score += 15;
                }
                
                // Check for known malicious commands
                if cmd_lower.contains("busybox") || cmd_lower.contains("xmrig") || 
                   cmd_lower.contains("mirai") || cmd_lower.contains("ddos") {
                    score += 25;
                }
                
                // Check for reverse shell attempts
                if (cmd_lower.contains("bash") && cmd_lower.contains("dev/tcp")) ||
                   (cmd_lower.contains("nc") && cmd_lower.contains("-e")) {
                    score += 30;
                }
            }
        }
        
        // Check for file uploads
        if !session.files.is_empty() {
            score += 10;
            
            // Check for malicious or executable files
            for file in &session.files {
                if file.is_executable {
                    score += 10;
                }
                
                if file.is_malware {
                    score += 30;
                }
            }
        }
        
        // Cap score at 100
        score.min(100)
    }
    
    /// Group sessions by source IP to identify potential campaigns
    pub fn identify_campaigns(&self, sessions: &[Session]) -> Vec<(String, Vec<&Session>)> {
        // Group sessions by source IP
        let mut ip_sessions: HashMap<String, Vec<&Session>> = HashMap::new();
        
        for session in sessions {
            if let Some(ip) = &session.src_ip {
                ip_sessions.entry(ip.clone()).or_insert_with(Vec::new).push(session);
            }
        }
        
        // Filter to only IPs with multiple sessions
        let campaigns: Vec<(String, Vec<&Session>)> = ip_sessions
            .into_iter()
            .filter(|(_, sessions)| sessions.len() > 1)
            .collect();
        
        campaigns
    }
    
    /// Correlate sessions based on command patterns
    pub fn correlate_command_patterns(&self, sessions: &[Session]) -> HashMap<String, Vec<String>> {
        let mut pattern_to_sessions: HashMap<String, Vec<String>> = HashMap::new();
        
        // Simple pattern extraction: First command run after login
        for session in sessions {
            if let Some(first_cmd) = session.commands.first() {
                let cmd_pattern = first_cmd.command.split_whitespace().next()
                    .unwrap_or("unknown")
                    .to_string();
                
                pattern_to_sessions
                    .entry(cmd_pattern)
                    .or_insert_with(Vec::new)
                    .push(session.id.clone());
            }
        }
        
        // Only return patterns with multiple sessions
        pattern_to_sessions
            .into_iter()
            .filter(|(_, sessions)| sessions.len() > 1)
            .collect()
    }
    
    /// Detect if a session belongs to a specific malware family
    pub fn detect_malware_family(&self, session: &Session) -> Option<String> {
        let cmd_str = session.commands.iter()
            .map(|cmd| cmd.command.as_str())
            .collect::<Vec<&str>>()
            .join("; ");
        
        // Check for common malware patterns
        if cmd_str.contains("busybox") && cmd_str.contains("wget") && cmd_str.contains("chmod +x") {
            return Some("Mirai-like".to_string());
        }
        
        if cmd_str.contains("xmrig") || cmd_str.contains("monero") || cmd_str.contains("cryptonight") {
            return Some("Crypto Miner".to_string());
        }
        
        if cmd_str.contains("/dev/tcp") && cmd_str.contains("sh -i") {
            return Some("Reverse Shell".to_string());
        }
        
        None
    }
}