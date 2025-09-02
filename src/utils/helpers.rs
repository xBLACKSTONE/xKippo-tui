use anyhow::Result;
use log::debug;
use std::path::Path;
use std::fs;

/// Helper functions for xKippo-TUI

/// Check if a path exists
pub fn path_exists(path: &str) -> bool {
    Path::new(path).exists()
}

/// Create directory if it doesn't exist
pub fn ensure_directory(path: &str) -> Result<()> {
    let path = Path::new(path);
    if !path.exists() {
        debug!("Creating directory: {:?}", path);
        fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Format bytes to human-readable string (KB, MB, GB)
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Format duration in seconds to human-readable string
pub fn format_duration(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;

    if days > 0 {
        format!("{}d {}h {}m {}s", days, hours, minutes, seconds)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}