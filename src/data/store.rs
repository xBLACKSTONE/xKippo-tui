use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use crate::config::Config;
use crate::data::models::{EventType, LogEntry, Session, User};

/// In-memory data store for honeypot data
pub struct Store {
    /// Log entries, indexed by ID
    log_entries: HashMap<String, LogEntry>,
    /// Sessions, indexed by session ID
    sessions: HashMap<String, Session>,
    /// Maximum number of log entries to keep
    max_logs: usize,
    /// Maximum number of sessions to track
    max_sessions: usize,
    /// Log entry IDs in chronological order
    log_entry_ids: Vec<String>,
    /// Session IDs in chronological order
    session_ids: Vec<String>,
    /// Unique source IPs
    unique_ips: HashSet<String>,
    /// Unique usernames
    unique_usernames: HashSet<String>,
    /// Unique passwords
    unique_passwords: HashSet<String>,
    /// Database path (if using persistent storage)
    db_path: Option<PathBuf>,
}

impl Store {
    /// Create a new data store
    pub fn new(config: &Config) -> Result<Self> {
        let max_logs = config.filter.max_logs;
        let max_sessions = config.filter.max_sessions;
        
        info!("Initializing data store with max_logs={}, max_sessions={}", max_logs, max_sessions);
        
        let store = Self {
            log_entries: HashMap::new(),
            sessions: HashMap::new(),
            max_logs,
            max_sessions,
            log_entry_ids: Vec::new(),
            session_ids: Vec::new(),
            unique_ips: HashSet::new(),
            unique_usernames: HashSet::new(),
            unique_passwords: HashSet::new(),
            db_path: None,
        };
        
        Ok(store)
    }
    
    /// Add a new log entry
    pub fn add_log_entry(&mut self, entry: LogEntry) -> Result<()> {
        // Track unique values
        if let Some(src_ip) = &entry.src_ip {
            self.unique_ips.insert(src_ip.clone());
        }
        
        if let Some(username) = &entry.username {
            self.unique_usernames.insert(username.clone());
        }
        
        if let Some(password) = &entry.password {
            self.unique_passwords.insert(password.clone());
        }
        
        // Add to chronological index
        self.log_entry_ids.push(entry.id.clone());
        
        // Add to map
        self.log_entries.insert(entry.id.clone(), entry);
        
        // Prune old entries if needed
        self.prune_log_entries();
        
        Ok(())
    }
    
    /// Get a log entry by ID
    pub fn get_log_entry(&self, id: &str) -> Option<&LogEntry> {
        self.log_entries.get(id)
    }
    
    /// Get all log entries
    pub fn get_log_entries(&self) -> Vec<&LogEntry> {
        // Return log entries in chronological order
        self.log_entry_ids.iter()
            .filter_map(|id| self.log_entries.get(id))
            .collect()
    }
    
    /// Get log entries by session ID
    pub fn get_log_entries_by_session(&self, session_id: &str) -> Vec<&LogEntry> {
        self.log_entry_ids.iter()
            .filter_map(|id| self.log_entries.get(id))
            .filter(|entry| entry.session.as_deref() == Some(session_id))
            .collect()
    }
    
    /// Get log entries by event type
    pub fn get_log_entries_by_event_type(&self, event_type: &EventType) -> Vec<&LogEntry> {
        self.log_entry_ids.iter()
            .filter_map(|id| self.log_entries.get(id))
            .filter(|entry| &entry.event_type == event_type)
            .collect()
    }
    
    /// Get log entries by time range
    pub fn get_log_entries_by_time_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<&LogEntry> {
        self.log_entry_ids.iter()
            .filter_map(|id| self.log_entries.get(id))
            .filter(|entry| entry.timestamp >= start && entry.timestamp <= end)
            .collect()
    }
    
    /// Get log entries by source IP
    pub fn get_log_entries_by_source_ip(&self, src_ip: &str) -> Vec<&LogEntry> {
        self.log_entry_ids.iter()
            .filter_map(|id| self.log_entries.get(id))
            .filter(|entry| entry.src_ip.as_deref() == Some(src_ip))
            .collect()
    }
    
    /// Get log entries by username
    pub fn get_log_entries_by_username(&self, username: &str) -> Vec<&LogEntry> {
        self.log_entry_ids.iter()
            .filter_map(|id| self.log_entries.get(id))
            .filter(|entry| entry.username.as_deref() == Some(username))
            .collect()
    }
    
    /// Add a new session
    pub fn add_session(&mut self, session: Session) -> Result<()> {
        // Add to chronological index
        self.session_ids.push(session.id.clone());
        
        // Add to map
        self.sessions.insert(session.id.clone(), session);
        
        // Prune old sessions if needed
        self.prune_sessions();
        
        Ok(())
    }
    
    /// Update an existing session
    pub fn update_session(&mut self, session: Session) -> Result<()> {
        // Check if session exists
        if !self.sessions.contains_key(&session.id) {
            return Err(anyhow::anyhow!("Session not found: {}", session.id));
        }
        
        // Update session
        self.sessions.insert(session.id.clone(), session);
        
        Ok(())
    }
    
    /// Get a session by ID
    pub fn get_session(&self, id: &str) -> Option<&Session> {
        self.sessions.get(id)
    }
    
    /// Get all sessions
    pub fn get_sessions(&self) -> Vec<&Session> {
        // Return sessions in chronological order
        self.session_ids.iter()
            .filter_map(|id| self.sessions.get(id))
            .collect()
    }
    
    /// Get active sessions (not ended)
    pub fn get_active_sessions(&self) -> Vec<&Session> {
        self.session_ids.iter()
            .filter_map(|id| self.sessions.get(id))
            .filter(|session| session.end_time.is_none())
            .collect()
    }
    
    /// Get sessions by source IP
    pub fn get_sessions_by_source_ip(&self, src_ip: &str) -> Vec<&Session> {
        self.session_ids.iter()
            .filter_map(|id| self.sessions.get(id))
            .filter(|session| session.src_ip == src_ip)
            .collect()
    }
    
    /// Get sessions by username
    pub fn get_sessions_by_username(&self, username: &str) -> Vec<&Session> {
        self.session_ids.iter()
            .filter_map(|id| self.sessions.get(id))
            .filter(|session| {
                session.user.as_ref().map_or(false, |user| user.username == username)
            })
            .collect()
    }
    
    /// Get sessions by time range
    pub fn get_sessions_by_time_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<&Session> {
        self.session_ids.iter()
            .filter_map(|id| self.sessions.get(id))
            .filter(|session| {
                (session.start_time >= start && session.start_time <= end) ||
                (session.end_time.map_or(false, |end_time| end_time >= start && end_time <= end))
            })
            .collect()
    }
    
    /// Get unique source IPs
    pub fn get_unique_source_ips(&self) -> &HashSet<String> {
        &self.unique_ips
    }
    
    /// Get unique usernames
    pub fn get_unique_usernames(&self) -> &HashSet<String> {
        &self.unique_usernames
    }
    
    /// Get unique passwords
    pub fn get_unique_passwords(&self) -> &HashSet<String> {
        &self.unique_passwords
    }
    
    /// Get total number of log entries
    pub fn get_log_entry_count(&self) -> usize {
        self.log_entries.len()
    }
    
    /// Get total number of sessions
    pub fn get_session_count(&self) -> usize {
        self.sessions.len()
    }
    
    /// Search log entries by keyword
    pub fn search_log_entries(&self, keyword: &str, case_sensitive: bool) -> Vec<&LogEntry> {
        let keyword = if case_sensitive {
            keyword.to_string()
        } else {
            keyword.to_lowercase()
        };
        
        self.log_entry_ids.iter()
            .filter_map(|id| self.log_entries.get(id))
            .filter(|entry| {
                // Search in various fields
                let command_match = entry.command.as_ref().map_or(false, |cmd| {
                    if case_sensitive {
                        cmd.contains(&keyword)
                    } else {
                        cmd.to_lowercase().contains(&keyword)
                    }
                });
                
                let username_match = entry.username.as_ref().map_or(false, |username| {
                    if case_sensitive {
                        username.contains(&keyword)
                    } else {
                        username.to_lowercase().contains(&keyword)
                    }
                });
                
                let password_match = entry.password.as_ref().map_or(false, |password| {
                    if case_sensitive {
                        password.contains(&keyword)
                    } else {
                        password.to_lowercase().contains(&keyword)
                    }
                });
                
                command_match || username_match || password_match
            })
            .collect()
    }
    
    /// Clear all data
    pub fn clear(&mut self) {
        self.log_entries.clear();
        self.sessions.clear();
        self.log_entry_ids.clear();
        self.session_ids.clear();
        self.unique_ips.clear();
        self.unique_usernames.clear();
        self.unique_passwords.clear();
        
        debug!("Cleared all data from store");
    }
    
    /// Prune old log entries if needed
    fn prune_log_entries(&mut self) {
        while self.log_entries.len() > self.max_logs {
            if let Some(oldest_id) = self.log_entry_ids.first().cloned() {
                self.log_entries.remove(&oldest_id);
                self.log_entry_ids.remove(0);
                debug!("Pruned oldest log entry: {}", oldest_id);
            } else {
                break;
            }
        }
    }
    
    /// Prune old sessions if needed
    fn prune_sessions(&mut self) {
        while self.sessions.len() > self.max_sessions {
            if let Some(oldest_id) = self.session_ids.first().cloned() {
                self.sessions.remove(&oldest_id);
                self.session_ids.remove(0);
                debug!("Pruned oldest session: {}", oldest_id);
            } else {
                break;
            }
        }
    }
}