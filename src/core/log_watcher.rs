use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use log::{debug, error, info, warn};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::task::JoinHandle;

use crate::app::AppEvent;
use crate::config::Config as AppConfig;
use crate::core::log_analyzer::LogAnalyzer;
use crate::data::{LogEntry, Store};

/// Start watching a log file for changes
pub async fn start_log_watcher(
    path: PathBuf,
    store: Arc<RwLock<Store>>,
    event_tx: broadcast::Sender<AppEvent>,
    config: &AppConfig,
) -> Result<JoinHandle<()>> {
    info!("Starting log watcher for {}", path.display());
    
    // Check if file exists
    if !path.exists() {
        return Err(anyhow::anyhow!("Log file does not exist: {}", path.display()));
    }
    
    // Create log analyzer
    let log_analyzer = LogAnalyzer::new();
    
    // Set up channel for file events
    let (file_event_tx, file_event_rx) = mpsc::channel(100);
    
    // Start file watcher
    let mut watcher = RecommendedWatcher::new(
        move |res| {
            match res {
                Ok(event) => {
                    if let Err(e) = file_event_tx.blocking_send(event) {
                        error!("Error sending file event: {}", e);
                    }
                }
                Err(e) => error!("Error watching file: {}", e),
            }
        },
        Config::default(),
    )?;
    
    // Watch file
    watcher.watch(path.parent().unwrap_or(&path), RecursiveMode::NonRecursive)?;
    
    // Determine starting point for log processing
    let start_time = determine_start_time(config)?;
    let file_path = path.clone();
    
    // Start processing task
    let task = tokio::spawn(async move {
        // Process existing log entries first
        if let Err(e) = process_existing_logs(
            &file_path,
            store.clone(),
            event_tx.clone(),
            &log_analyzer,
            start_time,
        ).await {
            error!("Error processing existing logs: {}", e);
        }
        
        // Process file change events
        process_file_events(
            file_event_rx,
            file_path,
            store,
            event_tx,
            log_analyzer,
        ).await;
    });
    
    Ok(task)
}

/// Process existing log entries in the file
async fn process_existing_logs(
    path: &Path,
    store: Arc<RwLock<Store>>,
    event_tx: broadcast::Sender<AppEvent>,
    log_analyzer: &LogAnalyzer,
    start_time: DateTime<Utc>,
) -> Result<()> {
    info!("Processing existing logs from {}", path.display());
    
    // Open the file
    let file = File::open(path).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    
    let mut count = 0;
    
    // Read lines and process
    while let Some(line) = lines.next_line().await? {
        // Parse and process the log entry
        match log_analyzer.parse_log_entry(&line) {
            Ok(entry) => {
                // Skip entries before start time
                if entry.timestamp < start_time {
                    continue;
                }
                
                // Add entry to store
                {
                    let mut store = store.write().await;
                    if let Err(e) = store.add_log_entry(entry.clone()) {
                        error!("Error adding log entry to store: {}", e);
                    }
                }
                
                // Send event
                let _ = event_tx.send(AppEvent::NewLogEntry(entry));
                
                count += 1;
            }
            Err(e) => {
                debug!("Error parsing log entry: {}", e);
            }
        }
    }
    
    info!("Processed {} existing log entries", count);
    Ok(())
}

/// Process file change events
async fn process_file_events(
    mut file_event_rx: mpsc::Receiver<Event>,
    path: PathBuf,
    store: Arc<RwLock<Store>>,
    event_tx: broadcast::Sender<AppEvent>,
    log_analyzer: LogAnalyzer,
) {
    let mut file_position = get_file_size(&path).unwrap_or(0);
    
    while let Some(event) = file_event_rx.recv().await {
        // Check if the event is relevant
        if !is_relevant_event(&event, &path) {
            continue;
        }
        
        // Handle file modification
        if matches!(event.kind, EventKind::Modify(_)) {
            if let Err(e) = process_file_changes(
                &path,
                &mut file_position,
                store.clone(),
                event_tx.clone(),
                &log_analyzer,
            ).await {
                error!("Error processing file changes: {}", e);
            }
        }
    }
}

/// Process changes to the log file
async fn process_file_changes(
    path: &Path,
    file_position: &mut u64,
    store: Arc<RwLock<Store>>,
    event_tx: broadcast::Sender<AppEvent>,
    log_analyzer: &LogAnalyzer,
) -> Result<()> {
    // Open the file
    let file = File::open(path).await?;
    
    // Get current file size
    let metadata = file.metadata().await?;
    let new_size = metadata.len();
    
    // Check if file was truncated
    if new_size < *file_position {
        debug!("File was truncated, resetting position");
        *file_position = 0;
    }
    
    // Check if file has grown
    if new_size > *file_position {
        // Seek to last position
        let take_bytes = new_size - *file_position;
        let file = tokio::fs::File::open(path).await?;
        let reader = BufReader::new(file.take(take_bytes));
        let mut lines = reader.lines();
        
        // Read and process new lines
        while let Some(line) = lines.next_line().await? {
            match log_analyzer.parse_log_entry(&line) {
                Ok(entry) => {
                    // Add entry to store
                    {
                        let mut store = store.write().await;
                        if let Err(e) = store.add_log_entry(entry.clone()) {
                            error!("Error adding log entry to store: {}", e);
                        }
                    }
                    
                    // Send event
                    let _ = event_tx.send(AppEvent::NewLogEntry(entry));
                }
                Err(e) => {
                    debug!("Error parsing log entry: {}", e);
                }
            }
        }
        
        // Update position
        *file_position = new_size;
    }
    
    Ok(())
}

/// Check if an event is relevant for the watched file
fn is_relevant_event(event: &Event, path: &Path) -> bool {
    for path_buf in &event.paths {
        if path_buf == path {
            return true;
        }
    }
    
    false
}

/// Get the current size of a file
fn get_file_size(path: &Path) -> Result<u64> {
    let metadata = std::fs::metadata(path)?;
    Ok(metadata.len())
}

/// Determine the start time for log processing based on configuration
fn determine_start_time(config: &AppConfig) -> Result<DateTime<Utc>> {
    let now = Utc::now();
    
    // Use history_hours from config
    let hours = config.honeypot.history_hours;
    
    if hours == 0 {
        // Process all logs
        Ok(DateTime::<Utc>::from(SystemTime::UNIX_EPOCH))
    } else {
        // Process logs from N hours ago
        Ok(now - Duration::hours(hours as i64))
    }
}