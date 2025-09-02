use anyhow::{Context, Result};
use chrono::Utc;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::task::JoinHandle;

use crate::app::AppEvent;
use crate::config::Config;
use crate::data::{EventType, LogEntry, Session, User};
use crate::data::Store;

/// Manages honeypot sessions
pub struct SessionManager {
    /// Data store
    store: Arc<RwLock<Store>>,
    /// Event sender
    event_tx: broadcast::Sender<AppEvent>,
    /// Active tasks
    tasks: Vec<JoinHandle<()>>,
    /// Configuration
    config: Config,
    /// Session timeout in seconds
    session_timeout: u64,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(
        store: Arc<RwLock<Store>>,
        event_tx: broadcast::Sender<AppEvent>,
        config: &Config,
    ) -> Result<Self> {
        // Default session timeout is 30 minutes
        let session_timeout = 30 * 60;
        
        Ok(Self {
            store,
            event_tx,
            tasks: Vec::new(),
            config: config.clone(),
            session_timeout,
        })
    }
    
    /// Start the session manager
    pub async fn start(&self) -> Result<()> {
        info!("Starting session manager");
        
        // Start session timeout checker
        let store = self.store.clone();
        let event_tx = self.event_tx.clone();
        let session_timeout = self.session_timeout;
        
        let task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                
                if let Err(e) = Self::check_session_timeouts(
                    store.clone(),
                    event_tx.clone(),
                    session_timeout,
                ).await {
                    error!("Error checking session timeouts: {}", e);
                }
            }
        });
        
        // Start event listener
        let store = self.store.clone();
        let event_tx = self.event_tx.clone();
        
        let task = tokio::spawn(async move {
            let mut rx = event_tx.subscribe();
            
            while let Ok(event) = rx.recv().await {
                match event {
                    AppEvent::NewLogEntry(entry) => {
                        if let Err(e) = Self::process_log_entry(
                            store.clone(),
                            event_tx.clone(),
                            entry,
                        ).await {
                            error!("Error processing log entry: {}", e);
                        }
                    }
                    AppEvent::Quit => break,
                    _ => {}
                }
            }
        });
        
        Ok(())
    }
    
    /// Stop the session manager
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping session manager");
        
        // Stop all tasks
        for task in &self.tasks {
            task.abort();
        }
        
        Ok(())
    }
    
    /// Process a new log entry
    async fn process_log_entry(
        store: Arc<RwLock<Store>>,
        event_tx: broadcast::Sender<AppEvent>,
        entry: LogEntry,
    ) -> Result<()> {
        // Get session ID from entry
        let session_id = match &entry.session {
            Some(id) => id,
            None => return Ok(()),
        };
        
        // Get current session or create a new one
        let mut session = {
            let store = store.read().await;
            store.get_session(session_id).cloned()
        };
        
        let session = match session {
            Some(mut session) => {
                // Update existing session
                Self::update_session_from_log_entry(&mut session, &entry);
                session
            }
            None => {
                // Create new session
                Self::create_session_from_log_entry(session_id, &entry)?
            }
        };
        
        // Update session in store
        {
            let mut store = store.write().await;
            if store.get_session(session_id).is_some() {
                store.update_session(session.clone())?;
            } else {
                store.add_session(session.clone())?;
            }
        }
        
        // Notify subscribers
        let _ = event_tx.send(AppEvent::SessionUpdate(session));
        
        Ok(())
    }
    
    /// Update a session with data from a log entry
    fn update_session_from_log_entry(session: &mut Session, entry: &LogEntry) {
        match entry.event_type {
            EventType::Connect => {
                // Update connection information
                if let Some(src_ip) = &entry.src_ip {
                    session.src_ip = src_ip.clone();
                }
                
                if let Some(src_port) = entry.src_port {
                    session.src_port = src_port;
                }
                
                if let Some(dst_ip) = &entry.dst_ip {
                    session.dst_ip = dst_ip.clone();
                }
                
                if let Some(dst_port) = entry.dst_port {
                    session.dst_port = dst_port;
                }
                
                // Check for client version in fields
                if let Some(version) = entry.fields.get("version") {
                    if let Some(version_str) = version.as_str() {
                        session.client_version = Some(version_str.to_string());
                    }
                }
            }
            
            EventType::Disconnect => {
                // Update end time
                session.end_time = Some(entry.timestamp);
                
                // Calculate duration
                if let Some(end_time) = session.end_time {
                    let duration = end_time.signed_duration_since(session.start_time);
                    session.duration = Some(duration.num_seconds() as u64);
                }
            }
            
            EventType::LoginAttempt | EventType::LoginSuccess | EventType::LoginFailed => {
                // Handle login attempts
                let username = entry.username.clone().unwrap_or_default();
                let password = entry.password.clone();
                let success = entry.event_type == EventType::LoginSuccess;
                
                // Update or create user information
                if session.user.is_none() || (success && !session.user.as_ref().unwrap().login_success) {
                    session.user = Some(User {
                        username,
                        password,
                        key_fingerprint: None,
                        login_success: success,
                        login_time: entry.timestamp,
                    });
                }
            }
            
            EventType::KeyAuth => {
                // Handle SSH key authentication
                let username = entry.username.clone().unwrap_or_default();
                let key_fingerprint = entry.fields.get("fingerprint")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                
                let success = entry.fields.get("success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                
                // Update or create user information
                if session.user.is_none() || (success && !session.user.as_ref().unwrap().login_success) {
                    session.user = Some(User {
                        username,
                        password: None,
                        key_fingerprint,
                        login_success: success,
                        login_time: entry.timestamp,
                    });
                }
            }
            
            EventType::Command => {
                // Handle command execution
                if let Some(cmd) = &entry.command {
                    let success = entry.fields.get("success")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true);
                    
                    let output = entry.fields.get("output")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    
                    session.commands.push(crate::data::Command {
                        command: cmd.clone(),
                        timestamp: entry.timestamp,
                        success,
                        output,
                    });
                }
            }
            
            EventType::FileUpload | EventType::FileDownload => {
                // Handle file transfers
                if let Some(file) = &entry.file {
                    session.files.push(file.clone());
                }
            }
            
            _ => {}
        }
        
        // Update TTY log information
        if let Some(tty_log) = entry.fields.get("ttylog") {
            if let Some(tty_log_str) = tty_log.as_str() {
                session.tty_log = Some(tty_log_str.to_string());
            }
        }
        
        // Update shasum
        if let Some(shasum) = entry.fields.get("shasum") {
            if let Some(shasum_str) = shasum.as_str() {
                session.shasum = Some(shasum_str.to_string());
            }
        }
    }
    
    /// Create a new session from a log entry
    fn create_session_from_log_entry(session_id: &str, entry: &LogEntry) -> Result<Session> {
        let src_ip = entry.src_ip.clone().unwrap_or_else(|| "0.0.0.0".to_string());
        let src_port = entry.src_port.unwrap_or(0);
        let dst_ip = entry.dst_ip.clone().unwrap_or_else(|| "0.0.0.0".to_string());
        let dst_port = entry.dst_port.unwrap_or(0);
        
        let protocol = if dst_port == 22 || dst_port == 2222 {
            "SSH"
        } else if dst_port == 23 || dst_port == 2223 {
            "Telnet"
        } else {
            "Unknown"
        }.to_string();
        
        let session = Session {
            id: session_id.to_string(),
            start_time: entry.timestamp,
            end_time: None,
            src_ip,
            src_port,
            dst_ip,
            dst_port,
            protocol,
            client_version: None,
            user: None,
            duration: None,
            commands: Vec::new(),
            files: Vec::new(),
            geo_location: None,
            tty_log: None,
            shasum: None,
            is_malicious: false,
            malicious_score: 0,
        };
        
        Ok(session)
    }
    
    /// Check for timed-out sessions
    async fn check_session_timeouts(
        store: Arc<RwLock<Store>>,
        event_tx: broadcast::Sender<AppEvent>,
        timeout: u64,
    ) -> Result<()> {
        debug!("Checking for timed-out sessions");
        
        let now = Utc::now();
        let mut sessions_to_update = Vec::new();
        
        // Find active sessions that have timed out
        {
            let store = store.read().await;
            let active_sessions = store.get_active_sessions();
            
            for session in active_sessions {
                // Check if session has timed out
                let elapsed = now.signed_duration_since(session.start_time);
                
                if elapsed.num_seconds() as u64 > timeout {
                    // Session has timed out
                    debug!("Session {} has timed out", session.id);
                    
                    let mut session = session.clone();
                    session.end_time = Some(now);
                    session.duration = Some(elapsed.num_seconds() as u64);
                    
                    sessions_to_update.push(session);
                }
            }
        }
        
        // Update timed-out sessions
        {
            let mut store = store.write().await;
            
            for session in &sessions_to_update {
                if let Err(e) = store.update_session(session.clone()) {
                    error!("Error updating timed-out session {}: {}", session.id, e);
                } else {
                    // Notify subscribers
                    let _ = event_tx.send(AppEvent::SessionUpdate(session.clone()));
                }
            }
        }
        
        Ok(())
    }
}