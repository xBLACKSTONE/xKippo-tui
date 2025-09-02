use anyhow::Result;
use chrono::Utc;
use log::{debug, info, warn};
use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

use crate::app::AppEvent;
use crate::config::AlertConfig;
use crate::data::{EventType, LogEntry, Session};

/// Alert types that can be triggered
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlertType {
    /// Successful login
    SuccessfulLogin {
        session_id: String,
        username: String,
        src_ip: String,
    },
    /// File upload
    FileUpload {
        session_id: String,
        filename: String,
        shasum: Option<String>,
    },
    /// Suspicious command
    SuspiciousCommand {
        session_id: String,
        command: String,
    },
    /// New source IP
    NewSourceIp {
        ip: String,
    },
    /// Blacklisted IP
    BlacklistedIp {
        ip: String,
    },
    /// High risk activity
    HighRiskActivity {
        session_id: String,
        risk_score: u8,
        reason: String,
    },
}

/// Alert notification
#[derive(Debug, Clone)]
pub struct Alert {
    /// Alert type
    pub alert_type: AlertType,
    /// Timestamp when the alert was generated
    pub timestamp: chrono::DateTime<Utc>,
    /// Is this alert acknowledged
    pub acknowledged: bool,
    /// Alert message
    pub message: String,
}

/// Alert engine that monitors events and generates alerts
pub struct AlertEngine {
    /// Alert configuration
    config: AlertConfig,
    /// Known source IPs
    known_ips: HashSet<IpAddr>,
    /// Blacklisted IPs
    blacklisted_ips: HashSet<IpAddr>,
    /// Whitelisted IPs
    whitelisted_ips: HashSet<IpAddr>,
    /// Event sender
    event_tx: broadcast::Sender<AppEvent>,
    /// Generated alerts
    alerts: Vec<Alert>,
}

impl AlertEngine {
    /// Create a new alert engine
    pub fn new(config: AlertConfig, event_tx: broadcast::Sender<AppEvent>) -> Self {
        let mut blacklisted_ips = HashSet::new();
        let mut whitelisted_ips = HashSet::new();
        
        // Parse IP blacklist
        for ip_str in &config.ip_blacklist {
            if let Ok(ip) = IpAddr::from_str(ip_str) {
                blacklisted_ips.insert(ip);
            }
        }
        
        // Parse IP whitelist
        for ip_str in &config.ip_whitelist {
            if let Ok(ip) = IpAddr::from_str(ip_str) {
                whitelisted_ips.insert(ip);
            }
        }
        
        Self {
            config,
            known_ips: HashSet::new(),
            blacklisted_ips,
            whitelisted_ips,
            event_tx,
            alerts: Vec::new(),
        }
    }
    
    /// Start the alert engine
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting alert engine");
        
        // Subscribe to events
        let mut rx = self.event_tx.subscribe();
        
        // Process events
        while let Ok(event) = rx.recv().await {
            match event {
                AppEvent::NewLogEntry(entry) => {
                    self.process_log_entry(&entry).await?;
                }
                AppEvent::SessionUpdate(session) => {
                    self.process_session_update(&session).await?;
                }
                AppEvent::Quit => break,
                _ => {}
            }
        }
        
        Ok(())
    }
    
    /// Process a new log entry
    async fn process_log_entry(&mut self, entry: &LogEntry) -> Result<()> {
        // Skip if alerts are disabled
        if !self.config.enabled {
            return Ok(());
        }
        
        // Check for successful login
        if self.config.on_successful_login && entry.event_type == EventType::LoginSuccess {
            if let (Some(session_id), Some(username), Some(src_ip)) = 
                (&entry.session, &entry.username, &entry.src_ip) {
                
                self.trigger_alert(AlertType::SuccessfulLogin {
                    session_id: session_id.clone(),
                    username: username.clone(),
                    src_ip: src_ip.clone(),
                });
            }
        }
        
        // Check for file upload
        if self.config.on_file_upload && entry.event_type == EventType::FileUpload {
            if let (Some(session_id), Some(file)) = (&entry.session, &entry.file) {
                self.trigger_alert(AlertType::FileUpload {
                    session_id: session_id.clone(),
                    filename: file.filename.clone(),
                    shasum: file.shasum.clone(),
                });
            }
        }
        
        // Check for specific commands
        if entry.event_type == EventType::Command {
            if let (Some(session_id), Some(command)) = (&entry.session, &entry.command) {
                for cmd_pattern in &self.config.on_commands {
                    if command.contains(cmd_pattern) {
                        self.trigger_alert(AlertType::SuspiciousCommand {
                            session_id: session_id.clone(),
                            command: command.clone(),
                        });
                        break;
                    }
                }
            }
        }
        
        // Check for new source IP
        if self.config.on_new_source_ip {
            if let Some(src_ip) = &entry.src_ip {
                if let Ok(ip) = IpAddr::from_str(src_ip) {
                    if !self.known_ips.contains(&ip) {
                        self.known_ips.insert(ip);
                        self.trigger_alert(AlertType::NewSourceIp {
                            ip: src_ip.clone(),
                        });
                    }
                }
            }
        }
        
        // Check for blacklisted IP
        if let Some(src_ip) = &entry.src_ip {
            if let Ok(ip) = IpAddr::from_str(src_ip) {
                if self.blacklisted_ips.contains(&ip) && !self.whitelisted_ips.contains(&ip) {
                    self.trigger_alert(AlertType::BlacklistedIp {
                        ip: src_ip.clone(),
                    });
                }
            }
        }
        
        Ok(())
    }
    
    /// Process a session update
    async fn process_session_update(&mut self, session: &Session) -> Result<()> {
        // Skip if alerts are disabled
        if !self.config.enabled {
            return Ok(());
        }
        
        // Check risk score
        if session.malicious_score >= 80 {
            self.trigger_alert(AlertType::HighRiskActivity {
                session_id: session.id.clone(),
                risk_score: session.malicious_score,
                reason: self.determine_risk_reason(session),
            });
        }
        
        Ok(())
    }
    
    /// Determine the reason for high risk score
    fn determine_risk_reason(&self, session: &Session) -> String {
        let mut reasons = Vec::new();
        
        // Check for successful login
        if let Some(user) = &session.user {
            if user.login_success {
                reasons.push("Successful login");
            }
        }
        
        // Check for commands
        if !session.commands.is_empty() {
            let suspicious_commands = session.commands.iter()
                .filter(|cmd| {
                    let cmd_lower = cmd.command.to_lowercase();
                    cmd_lower.contains("wget") || cmd_lower.contains("curl") || 
                    cmd_lower.contains("tftp") || cmd_lower.contains("chmod") || 
                    cmd_lower.contains("busybox") || cmd_lower.contains("xmrig") || 
                    cmd_lower.contains("mirai") || cmd_lower.contains("ddos")
                })
                .count();
            
            if suspicious_commands > 0 {
                reasons.push(&format!("{} suspicious commands", suspicious_commands));
            }
        }
        
        // Check for file uploads
        if !session.files.is_empty() {
            reasons.push(&format!("{} files transferred", session.files.len()));
        }
        
        if reasons.is_empty() {
            "Unknown high risk activity".to_string()
        } else {
            reasons.join(", ")
        }
    }
    
    /// Trigger an alert
    fn trigger_alert(&mut self, alert_type: AlertType) {
        let message = match &alert_type {
            AlertType::SuccessfulLogin { username, src_ip, .. } => {
                format!("Successful login for user '{}' from {}", username, src_ip)
            }
            AlertType::FileUpload { filename, shasum, .. } => {
                if let Some(hash) = shasum {
                    format!("File uploaded: {} (SHA256: {})", filename, hash)
                } else {
                    format!("File uploaded: {}", filename)
                }
            }
            AlertType::SuspiciousCommand { command, .. } => {
                format!("Suspicious command: {}", command)
            }
            AlertType::NewSourceIp { ip } => {
                format!("New source IP detected: {}", ip)
            }
            AlertType::BlacklistedIp { ip } => {
                format!("Connection from blacklisted IP: {}", ip)
            }
            AlertType::HighRiskActivity { risk_score, reason, .. } => {
                format!("High risk activity detected (Score: {}): {}", risk_score, reason)
            }
        };
        
        let alert = Alert {
            alert_type,
            timestamp: Utc::now(),
            acknowledged: false,
            message,
        };
        
        // Log the alert
        warn!("ALERT: {}", alert.message);
        
        // Add to alerts list
        self.alerts.push(alert.clone());
        
        // Send visual alert if enabled
        if self.config.visual_enabled {
            // Visual alerts will be handled by the UI
        }
        
        // Send sound alert if enabled
        if self.config.sound_enabled {
            // Sound alerts would be implemented here
            // This is platform-specific and would require additional dependencies
        }
    }
    
    /// Get all alerts
    pub fn get_alerts(&self) -> &[Alert] {
        &self.alerts
    }
    
    /// Acknowledge an alert
    pub fn acknowledge_alert(&mut self, index: usize) -> Result<()> {
        if index < self.alerts.len() {
            self.alerts[index].acknowledged = true;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Alert index out of bounds"))
        }
    }
    
    /// Clear all acknowledged alerts
    pub fn clear_acknowledged_alerts(&mut self) {
        self.alerts.retain(|alert| !alert.acknowledged);
    }
    
    /// Clear all alerts
    pub fn clear_all_alerts(&mut self) {
        self.alerts.clear();
    }
}