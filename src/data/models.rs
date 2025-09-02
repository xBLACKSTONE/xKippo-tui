use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Log entry from honeypot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Unique identifier for this log entry
    pub id: String,
    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
    /// Type of event
    pub event_type: EventType,
    /// Session identifier
    pub session: Option<String>,
    /// Source IP address
    pub src_ip: Option<String>,
    /// Source port
    pub src_port: Option<u16>,
    /// Destination IP address
    pub dst_ip: Option<String>,
    /// Destination port
    pub dst_port: Option<u16>,
    /// Username (for login attempts)
    pub username: Option<String>,
    /// Password (for login attempts)
    pub password: Option<String>,
    /// Command (for command execution)
    pub command: Option<String>,
    /// File information (for uploads/downloads)
    pub file: Option<FileTransfer>,
    /// Additional fields
    pub fields: HashMap<String, serde_json::Value>,
    /// Raw log entry as JSON
    pub raw: serde_json::Value,
}

/// Type of event
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    /// Connection established
    Connect,
    /// Connection closed
    Disconnect,
    /// Login attempt
    LoginAttempt,
    /// Successful login
    LoginSuccess,
    /// Failed login
    LoginFailed,
    /// Command execution
    Command,
    /// File upload
    FileUpload,
    /// File download
    FileDownload,
    /// SSH key authentication attempt
    KeyAuth,
    /// TCP forwarding request
    TCPForward,
    /// Unknown event type
    Unknown,
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::Connect => write!(f, "Connect"),
            EventType::Disconnect => write!(f, "Disconnect"),
            EventType::LoginAttempt => write!(f, "Login Attempt"),
            EventType::LoginSuccess => write!(f, "Login Success"),
            EventType::LoginFailed => write!(f, "Login Failed"),
            EventType::Command => write!(f, "Command"),
            EventType::FileUpload => write!(f, "File Upload"),
            EventType::FileDownload => write!(f, "File Download"),
            EventType::KeyAuth => write!(f, "Key Auth"),
            EventType::TCPForward => write!(f, "TCP Forward"),
            EventType::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Session identifier
    pub id: String,
    /// Start time
    pub start_time: DateTime<Utc>,
    /// End time (if session has ended)
    pub end_time: Option<DateTime<Utc>>,
    /// Source IP address
    pub src_ip: String,
    /// Source port
    pub src_port: u16,
    /// Destination IP address
    pub dst_ip: String,
    /// Destination port
    pub dst_port: u16,
    /// Protocol (SSH, Telnet)
    pub protocol: String,
    /// Client version
    pub client_version: Option<String>,
    /// User information
    pub user: Option<User>,
    /// Session duration in seconds
    pub duration: Option<u64>,
    /// Commands executed in this session
    pub commands: Vec<Command>,
    /// Files transferred in this session
    pub files: Vec<FileTransfer>,
    /// Geographic location information
    pub geo_location: Option<GeoLocation>,
    /// Path to TTY log file
    pub tty_log: Option<String>,
    /// Session SHA256 hash
    pub shasum: Option<String>,
    /// Session is marked as malicious
    pub is_malicious: bool,
    /// Malicious score (0-100)
    pub malicious_score: u8,
}

/// User information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Username
    pub username: String,
    /// Password
    pub password: Option<String>,
    /// SSH key fingerprint
    pub key_fingerprint: Option<String>,
    /// Login success
    pub login_success: bool,
    /// Login time
    pub login_time: DateTime<Utc>,
}

/// Command execution information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    /// Command entered
    pub command: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Success status
    pub success: bool,
    /// Command output
    pub output: Option<String>,
}

/// File transfer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTransfer {
    /// File name
    pub filename: String,
    /// Local storage path
    pub local_path: Option<String>,
    /// File size in bytes
    pub size: Option<u64>,
    /// SHA256 hash of file
    pub shasum: Option<String>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Transfer direction (upload/download)
    pub direction: FileTransferDirection,
    /// MIME type
    pub mime_type: Option<String>,
    /// Is file executable
    pub is_executable: bool,
    /// Is file detected as malware
    pub is_malware: bool,
}

/// File transfer direction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileTransferDirection {
    /// Upload (attacker to honeypot)
    Upload,
    /// Download (honeypot to attacker)
    Download,
}

/// Geographic location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocation {
    /// Country code
    pub country_code: String,
    /// Country name
    pub country_name: String,
    /// City
    pub city: Option<String>,
    /// Latitude
    pub latitude: Option<f64>,
    /// Longitude
    pub longitude: Option<f64>,
    /// Autonomous system number
    pub asn: Option<String>,
    /// ISP name
    pub isp: Option<String>,
}