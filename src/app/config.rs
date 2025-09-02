//! Configuration handling for the application
//! 
//! This module provides functionality to load and validate configuration
//! from config files and environment variables.

use anyhow::{Context, Result};
use config::{Config as ConfigSource, ConfigBuilder, File, FileFormat};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    /// General application settings
    pub general: GeneralConfig,
    
    /// Honeypot connection settings
    pub connection: ConnectionConfig,
    
    /// User interface settings
    pub ui: UiConfig,
    
    /// Logging settings
    pub logging: LoggingConfig,
}

/// General application settings
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeneralConfig {
    /// Application name
    pub name: String,
    
    /// Auto-refresh interval in seconds
    pub refresh_interval: u64,
    
    /// Maximum number of sessions to keep in memory
    pub max_sessions: usize,
}

/// Honeypot connection settings
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConnectionConfig {
    /// Whether to enable connection
    pub enabled: bool,
    
    /// Connection type (ssh, direct, log, etc.)
    pub connection_type: String,
    
    /// Hostname or IP address
    pub host: String,
    
    /// Port number
    pub port: u16,
    
    /// Username for authentication
    pub username: String,
    
    /// Password or key file path
    pub auth_method: String,
    
    /// Connection timeout in seconds
    pub timeout: u64,
}

/// User interface settings
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UiConfig {
    /// Color theme
    pub theme: String,
    
    /// Enable or disable animations
    pub enable_animations: bool,
    
    /// Default tab to show on startup
    pub default_tab: String,
    
    /// Date and time format
    pub datetime_format: String,
}

/// Logging settings
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoggingConfig {
    /// Log level (error, warn, info, debug, trace)
    pub level: String,
    
    /// Log file path
    pub file: Option<String>,
    
    /// Whether to log to stdout
    pub stdout: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                name: "xKippo-tui".to_string(),
                refresh_interval: 5,
                max_sessions: 1000,
            },
            connection: ConnectionConfig {
                enabled: false,
                connection_type: "ssh".to_string(),
                host: "localhost".to_string(),
                port: 2222,
                username: "admin".to_string(),
                auth_method: "password".to_string(),
                timeout: 10,
            },
            ui: UiConfig {
                theme: "dark".to_string(),
                enable_animations: true,
                default_tab: "dashboard".to_string(),
                datetime_format: "%Y-%m-%d %H:%M:%S".to_string(),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file: Some("xkippo.log".to_string()),
                stdout: true,
            },
        }
    }
}

/// Get the configuration file path
pub fn get_config_path() -> PathBuf {
    if let Ok(path) = std::env::var("XKIPPO_CONFIG_PATH") {
        return PathBuf::from(path);
    }
    
    // Check in current directory first
    let current_dir_config = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("config.toml");
    
    if current_dir_config.exists() {
        return current_dir_config;
    }

    // Then check in user config directory
    if let Some(config_dir) = dirs::config_dir() {
        let user_config = config_dir.join("xKippo-tui").join("config.toml");
        if user_config.exists() {
            return user_config;
        }
    }
    
    // Default to current directory
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("config.toml")
}

/// Load configuration from file and environment variables
pub fn load_config() -> Result<Config> {
    let config_path = get_config_path();
    
    let config_builder = ConfigBuilder::<ConfigSource>::default()
        .add_source(File::new("config.toml", FileFormat::Toml).required(false))
        .add_source(File::from(config_path).required(false))
        // Override with environment variables prefixed with XKIPPO_
        .add_source(config::Environment::with_prefix("XKIPPO").separator("_"))
        .build()
        .context("Failed to build configuration")?;
        
    // Try to deserialize the config into our Config struct
    let config: Config = match config_builder.try_deserialize() {
        Ok(cfg) => cfg,
        Err(e) => {
            // If deserialization fails, log the error and use default config
            eprintln!("Error loading config: {}. Using default configuration.", e);
            Config::default()
        }
    };
    
    Ok(config)
}

/// Save configuration to file
pub fn save_config(config: &Config, path: Option<PathBuf>) -> Result<()> {
    let config_path = path.unwrap_or_else(get_config_path);
    
    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .context("Failed to create parent directories for config file")?;
    }
    
    // Serialize and write config
    let toml = toml::to_string_pretty(config)
        .context("Failed to serialize configuration")?;
    
    std::fs::write(&config_path, toml)
        .context("Failed to write configuration to file")?;
    
    Ok(())
}