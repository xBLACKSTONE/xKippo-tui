use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use log::{debug, error, trace};
use serde_json::{json, Value};
use std::collections::HashMap;
use uuid::Uuid;

use crate::data::{EventType, FileTransfer, FileTransferDirection, LogEntry};

/// Analyzes and parses log entries from Cowrie honeypot
pub struct LogAnalyzer {
    /// Mapping of Cowrie event types to our EventType enum
    event_type_mapping: HashMap<String, EventType>,
}

impl LogAnalyzer {
    /// Create a new log analyzer
    pub fn new() -> Self {
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
        
        Self {
            event_type_mapping,
        }
    }
    
    /// Parse a JSON log entry into our LogEntry struct
    pub fn parse_log_entry(&self, line: &str) -> Result<LogEntry> {
        trace!("Parsing log entry: {}", line);
        
        // Parse JSON
        let value: Value = serde_json::from_str(line)
            .context("Failed to parse log entry as JSON")?;
        
        // Extract required fields
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
                
                Some(FileTransfer {
                    filename,
                    local_path: outfile,
                    size: None, // Size is often not included in log entries
                    shasum,
                    timestamp,
                    direction,
                    mime_type: None,
                    is_executable: false,
                    is_malware: false,
                })
            }
            _ => None,
        }
    }
    
    /// Analyze a session for potential malicious activity
    pub fn analyze_session_risk(&self, session: &crate::data::Session) -> u8 {
        let mut score = 0;
        
        // Check for successful login
        if let Some(user) = &session.user {
            if user.login_success {
                score += 10;
            }
        }
        
        // Check for commands
        if !session.commands.is_empty() {
            score += 20;
            
            // Check for suspicious commands
            for cmd in &session.commands {
                let cmd_lower = cmd.command.to_lowercase();
                
                // Check for downloading tools
                if cmd_lower.contains("wget") || cmd_lower.contains("curl") || cmd_lower.contains("tftp") {
                    score += 10;
                }
                
                // Check for common malware paths
                if cmd_lower.contains("/tmp") || cmd_lower.contains("/var/tmp") || cmd_lower.contains("/dev/shm") {
                    score += 5;
                }
                
                // Check for chmod
                if cmd_lower.contains("chmod") && cmd_lower.contains("777") {
                    score += 15;
                }
                
                // Check for known malicious commands
                if cmd_lower.contains("busybox") || cmd_lower.contains("xmrig") || 
                   cmd_lower.contains("mirai") || cmd_lower.contains("ddos") {
                    score += 25;
                }
            }
        }
        
        // Check for file uploads
        if !session.files.is_empty() {
            score += 30;
        }
        
        // Cap score at 100
        score.min(100)
    }
}