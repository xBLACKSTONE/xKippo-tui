use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex, RwLock};

use crate::config::Config;
use crate::core::{self, SessionManager};
use crate::data::{LogEntry, Session, Store};

/// Current application state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    /// Application is starting up
    Starting,
    /// Normal operation mode
    Running,
    /// Configuration mode
    ConfigMode,
    /// Viewing session details
    SessionView,
    /// Shutting down
    ShuttingDown,
}

/// Connection status to the honeypot
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    /// Not connected
    Disconnected,
    /// Currently connecting
    Connecting,
    /// Connected and monitoring
    Connected,
    /// Connection failed
    Failed(u8), // Retry count
}

/// Main application struct
pub struct App {
    /// Current application state
    pub state: AppState,
    /// Application configuration
    pub config: Config,
    /// Status of connection to honeypot
    pub connection_status: ConnectionStatus,
    /// Data store
    pub store: Arc<RwLock<Store>>,
    /// Session manager
    pub session_manager: Arc<SessionManager>,
    /// Event sender
    pub event_tx: broadcast::Sender<AppEvent>,
    /// Currently selected tab index
    pub selected_tab: usize,
    /// Currently selected session ID (if in session view)
    pub selected_session_id: Option<String>,
    /// Currently active filters
    pub filters: AppFilters,
    /// Application statistics
    pub stats: Arc<Mutex<AppStats>>,
    /// Path to honeypot logs
    pub log_paths: Vec<PathBuf>,
}

/// Application event types
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// New log entry detected
    NewLogEntry(LogEntry),
    /// New session detected
    NewSession(Session),
    /// Session updated
    SessionUpdate(Session),
    /// Connection status changed
    ConnectionStatusChange(ConnectionStatus),
    /// Quit application
    Quit,
}

/// Application filters
#[derive(Debug, Clone, Default)]
pub struct AppFilters {
    /// Filter by source IP
    pub source_ip: Option<String>,
    /// Filter by username
    pub username: Option<String>,
    /// Filter by timestamp (from)
    pub from_time: Option<DateTime<Utc>>,
    /// Filter by timestamp (to)
    pub to_time: Option<DateTime<Utc>>,
    /// Filter by event type
    pub event_type: Option<String>,
    /// Search string
    pub search: Option<String>,
}

/// Application statistics
#[derive(Debug, Default)]
pub struct AppStats {
    /// Total log entries processed
    pub total_log_entries: u64,
    /// Total sessions detected
    pub total_sessions: u64,
    /// Total login attempts
    pub login_attempts: u64,
    /// Successful logins
    pub successful_logins: u64,
    /// Commands executed
    pub commands_executed: u64,
    /// Files uploaded
    pub files_uploaded: u64,
    /// Unique source IPs
    pub unique_ips: std::collections::HashSet<String>,
    /// Unique usernames tried
    pub unique_usernames: std::collections::HashSet<String>,
    /// Unique passwords tried
    pub unique_passwords: std::collections::HashSet<String>,
}

impl App {
    /// Create a new application instance
    pub async fn new(config: Config) -> Result<Self> {
        info!("Initializing application");

        // Set up event channel
        let (event_tx, _) = broadcast::channel(100);

        // Create data store
        let store = Arc::new(RwLock::new(Store::new(&config)?));

        // Create session manager
        let session_manager = Arc::new(SessionManager::new(
            store.clone(),
            event_tx.clone(),
            &config,
        )?);

        // Determine log paths
        let log_paths = find_log_paths(&config)
            .context("Failed to locate honeypot log paths")?;

        if log_paths.is_empty() {
            warn!("No log paths found. Use setup script or configure manually.");
        } else {
            info!("Found {} log paths", log_paths.len());
            for path in &log_paths {
                debug!("Log path: {}", path.display());
            }
        }

        let app = Self {
            state: AppState::Starting,
            config,
            connection_status: ConnectionStatus::Disconnected,
            store,
            session_manager,
            event_tx,
            selected_tab: 0,
            selected_session_id: None,
            filters: AppFilters::default(),
            stats: Arc::new(Mutex::new(AppStats::default())),
            log_paths,
        };

        Ok(app)
    }

    /// Connect to the honeypot logs
    pub async fn connect(&mut self) -> Result<()> {
        info!("Connecting to honeypot logs");
        self.connection_status = ConnectionStatus::Connecting;

        // Initialize file watchers for log paths
        for path in &self.log_paths {
            match core::start_log_watcher(
                path.clone(),
                self.store.clone(),
                self.event_tx.clone(),
                &self.config,
            )
            .await
            {
                Ok(_) => {
                    info!("Started watching log file: {}", path.display());
                }
                Err(e) => {
                    error!("Failed to watch log file {}: {}", path.display(), e);
                    self.connection_status = ConnectionStatus::Failed(1);
                    return Err(e);
                }
            }
        }

        // Start session manager
        self.session_manager.start().await?;

        self.connection_status = ConnectionStatus::Connected;
        self.state = AppState::Running;

        info!("Connected successfully");
        Ok(())
    }

    /// Update application state
    pub fn update(&mut self) -> Result<()> {
        // Process any pending events
        // Update statistics
        // Check connection status
        Ok(())
    }

    /// Handle quit request
    pub async fn quit(&mut self) -> Result<()> {
        info!("Shutting down");
        self.state = AppState::ShuttingDown;
        
        // Graceful shutdown of components
        self.session_manager.stop().await?;
        
        // Notify subscribers
        let _ = self.event_tx.send(AppEvent::Quit);
        
        Ok(())
    }
}

/// Find log paths based on configuration and common locations
fn find_log_paths(config: &Config) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();

    // Use configured paths first
    if let Some(configured_paths) = &config.honeypot.log_paths {
        for path in configured_paths {
            paths.push(PathBuf::from(path));
        }
    }

    // If no configured paths or auto-detect is enabled, try common locations
    if paths.is_empty() || config.honeypot.auto_detect {
        // Common Cowrie log locations
        let common_paths = [
            "/var/log/cowrie/cowrie.json",
            "/opt/cowrie/var/log/cowrie/cowrie.json",
            "/home/cowrie/cowrie/var/log/cowrie/cowrie.json",
            "/usr/local/cowrie/var/log/cowrie/cowrie.json",
        ];

        for path in &common_paths {
            let path = PathBuf::from(path);
            if path.exists() {
                paths.push(path);
            }
        }
    }

    Ok(paths)
}