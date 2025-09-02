use std::fmt;
use std::error::Error;
use std::io;

/// Application error types
#[derive(Debug)]
pub enum AppError {
    /// IO errors
    IoError(io::Error),
    /// Configuration errors
    ConfigError(String),
    /// Plugin loading errors
    PluginLoadError(String),
    /// Plugin initialization errors
    PluginInitError(String, String),
    /// Plugin configuration errors
    PluginConfigError(String, String),
    /// Log parsing errors
    LogParseError(String),
    /// Database errors
    DatabaseError(String),
    /// Network errors
    NetworkError(String),
    /// UI errors
    UiError(String),
    /// Geolocation errors
    GeolocationError(String),
    /// Data validation errors
    ValidationError(String),
    /// External API errors
    ApiError(String),
    /// Other errors
    Other(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(err) => write!(f, "IO Error: {}", err),
            Self::ConfigError(err) => write!(f, "Configuration Error: {}", err),
            Self::PluginLoadError(err) => write!(f, "Plugin Loading Error: {}", err),
            Self::PluginInitError(plugin, err) => write!(f, "Plugin '{}' Initialization Error: {}", plugin, err),
            Self::PluginConfigError(plugin, err) => write!(f, "Plugin '{}' Configuration Error: {}", plugin, err),
            Self::LogParseError(err) => write!(f, "Log Parsing Error: {}", err),
            Self::DatabaseError(err) => write!(f, "Database Error: {}", err),
            Self::NetworkError(err) => write!(f, "Network Error: {}", err),
            Self::UiError(err) => write!(f, "UI Error: {}", err),
            Self::GeolocationError(err) => write!(f, "Geolocation Error: {}", err),
            Self::ValidationError(err) => write!(f, "Validation Error: {}", err),
            Self::ApiError(err) => write!(f, "API Error: {}", err),
            Self::Other(err) => write!(f, "Error: {}", err),
        }
    }
}

impl Error for AppError {}

impl From<io::Error> for AppError {
    fn from(error: io::Error) -> Self {
        Self::IoError(error)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        Self::LogParseError(error.to_string())
    }
}

impl From<toml::de::Error> for AppError {
    fn from(error: toml::de::Error) -> Self {
        Self::ConfigError(error.to_string())
    }
}

impl From<toml::ser::Error> for AppError {
    fn from(error: toml::ser::Error) -> Self {
        Self::ConfigError(error.to_string())
    }
}

impl From<std::string::FromUtf8Error> for AppError {
    fn from(error: std::string::FromUtf8Error) -> Self {
        Self::Other(error.to_string())
    }
}