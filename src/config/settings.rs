use anyhow::{Context, Result};
use dirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use toml;

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Honeypot-specific configuration
    pub honeypot: HoneypotConfig,
    /// User interface configuration
    pub ui: UIConfig,
    /// Filtering configuration
    pub filter: FilterConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Alert configuration
    pub alert: AlertConfig,
    /// GeoIP configuration
    #[serde(default)]
    pub geoip: GeoIPConfig,
    /// Security analyst features
    #[serde(default)]
    pub security_analyst: SecurityAnalystConfig,
    /// Threat intelligence features
    #[serde(default)]
    pub threat_intel: ThreatIntelConfig,
    /// Malware analysis features
    #[serde(default)]
    pub malware_analysis: MalwareAnalysisConfig,
    /// SIEM integration
    #[serde(default)]
    pub siem_integration: SIEMIntegrationConfig,
    /// Export configuration
    #[serde(default)]
    pub export: ExportConfig,
    /// Dashboard configuration
    #[serde(default)]
    pub dashboard: DashboardConfig,
    /// Rules configuration
    #[serde(default)]
    pub rules: RulesConfig,
}

/// Honeypot-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoneypotConfig {
    /// Name of the honeypot
    #[serde(default = "default_honeypot_name")]
    pub name: String,
    /// Type of honeypot (cowrie, kippo, etc.)
    #[serde(default = "default_honeypot_type")]
    pub honeypot_type: String,
    /// Path to log files
    pub log_paths: Option<Vec<String>>,
    /// Automatically detect log files
    #[serde(default = "default_true")]
    pub auto_detect: bool,
    /// Path to download directory
    pub download_path: Option<String>,
    /// Path to TTY log directory
    pub tty_log_path: Option<String>,
    /// How far back to process logs on startup (in hours, 0 = from beginning)
    #[serde(default = "default_history_hours")]
    pub history_hours: u32,
    /// Check interval in milliseconds
    #[serde(default = "default_check_interval")]
    pub check_interval_ms: u64,
}

/// User interface configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIConfig {
    /// Theme name
    #[serde(default = "default_theme")]
    pub theme: String,
    /// Enable color
    #[serde(default = "default_true")]
    pub color_enabled: bool,
    /// Enable mouse support
    #[serde(default = "default_true")]
    pub mouse_enabled: bool,
    /// Default tab
    #[serde(default)]
    pub default_tab: usize,
    /// Refresh interval in milliseconds
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval_ms: u64,
    /// Enable animations
    #[serde(default = "default_true")]
    pub animations: bool,
    /// Terminal title
    #[serde(default = "default_terminal_title")]
    pub terminal_title: String,
    /// Show border
    #[serde(default = "default_true")]
    pub show_border: bool,
    /// Border type
    #[serde(default = "default_border_type")]
    pub border_type: String,
    /// Date format
    #[serde(default = "default_date_format")]
    pub date_format: String,
    /// Time format
    #[serde(default = "default_time_format")]
    pub time_format: String,
    /// Show help bar
    #[serde(default = "default_true")]
    pub show_help: bool,
    /// Show status bar
    #[serde(default = "default_true")]
    pub show_status: bool,
}

/// Filtering configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    /// Default filter for event types
    #[serde(default)]
    pub default_event_types: Vec<String>,
    /// Case-sensitive search
    #[serde(default)]
    pub case_sensitive: bool,
    /// Filter presets
    #[serde(default)]
    pub presets: Vec<FilterPreset>,
    /// Maximum sessions to display
    #[serde(default = "default_max_sessions")]
    pub max_sessions: usize,
    /// Maximum log entries to keep in memory
    #[serde(default = "default_max_logs")]
    pub max_logs: usize,
}

/// Filter preset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterPreset {
    /// Preset name
    pub name: String,
    /// Source IP filter
    pub source_ip: Option<String>,
    /// Username filter
    pub username: Option<String>,
    /// Event type filter
    pub event_type: Option<String>,
    /// Search string
    pub search: Option<String>,
    /// Description
    pub description: Option<String>,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Enable application logging
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Log level
    #[serde(default = "default_log_level")]
    pub level: String,
    /// Log file path
    pub file_path: Option<String>,
    /// Log to console
    #[serde(default)]
    pub console: bool,
    /// Maximum log file size (in MB)
    #[serde(default = "default_log_size")]
    pub max_file_size: u64,
    /// Maximum number of log files to keep
    #[serde(default = "default_log_files")]
    pub max_files: u32,
}

/// Alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    /// Enable alerts
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Alert on successful login
    #[serde(default = "default_true")]
    pub on_successful_login: bool,
    /// Alert on file upload
    #[serde(default = "default_true")]
    pub on_file_upload: bool,
    /// Alert on command execution (specific commands)
    #[serde(default)]
    pub on_commands: Vec<String>,
    /// Alert on new source IP
    #[serde(default)]
    pub on_new_source_ip: bool,
    /// IP address blacklist
    #[serde(default)]
    pub ip_blacklist: Vec<String>,
    /// IP address whitelist
    #[serde(default)]
    pub ip_whitelist: Vec<String>,
    /// Sound alerts
    #[serde(default)]
    pub sound_enabled: bool,
    /// Visual alerts
    #[serde(default = "default_true")]
    pub visual_enabled: bool,
}

/// GeoIP configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GeoIPConfig {
    /// Enable GeoIP lookups
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Path to GeoIP database
    pub database_path: Option<String>,
    /// Download database if missing
    #[serde(default)]
    pub auto_download: bool,
    /// License key for MaxMind
    pub license_key: Option<String>,
}

impl Config {
    /// Load configuration from file or create default
    pub fn load(path: Option<&Path>) -> Result<Self> {
        // Try to load from provided path
        if let Some(path) = path {
            if path.exists() {
                return Self::from_file(path);
            }
        }

        // Try to load from default paths
        for path in get_config_paths() {
            if path.exists() {
                return Self::from_file(&path);
            }
        }

        // Create default config
        let config = Self::default();

        // Try to save default config to user config directory
        if let Some(user_config_dir) = dirs::config_dir() {
            let app_config_dir = user_config_dir.join("xkippo");
            let _ = fs::create_dir_all(&app_config_dir);
            
            let config_path = app_config_dir.join("config.toml");
            let _ = config.save(&config_path);
        }

        Ok(config)
    }

    /// Load configuration from file
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .context(format!("Failed to read config file: {}", path.display()))?;
        
        let config: Self = toml::from_str(&content)
            .context("Failed to parse config file")?;
        
        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self, path: &Path) -> Result<()> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .context(format!("Failed to create directory: {}", parent.display()))?;
        }

        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        
        fs::write(path, content)
            .context(format!("Failed to write config file: {}", path.display()))?;
        
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            honeypot: HoneypotConfig::default(),
            ui: UIConfig::default(),
            filter: FilterConfig::default(),
            logging: LoggingConfig::default(),
            alert: AlertConfig::default(),
            geoip: GeoIPConfig::default(),
            security_analyst: SecurityAnalystConfig::default(),
            threat_intel: ThreatIntelConfig::default(),
            malware_analysis: MalwareAnalysisConfig::default(),
            siem_integration: SIEMIntegrationConfig::default(),
            export: ExportConfig::default(),
            dashboard: DashboardConfig::default(),
            rules: RulesConfig::default(),
        }
    }
}

impl Default for HoneypotConfig {
    fn default() -> Self {
        Self {
            name: default_honeypot_name(),
            honeypot_type: default_honeypot_type(),
            log_paths: None,
            auto_detect: default_true(),
            download_path: None,
            tty_log_path: None,
            history_hours: default_history_hours(),
            check_interval_ms: default_check_interval(),
        }
    }
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            color_enabled: default_true(),
            mouse_enabled: default_true(),
            default_tab: 0,
            refresh_interval_ms: default_refresh_interval(),
            animations: default_true(),
            terminal_title: default_terminal_title(),
            show_border: default_true(),
            border_type: default_border_type(),
            date_format: default_date_format(),
            time_format: default_time_format(),
            show_help: default_true(),
            show_status: default_true(),
        }
    }
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            default_event_types: Vec::new(),
            case_sensitive: false,
            presets: Vec::new(),
            max_sessions: default_max_sessions(),
            max_logs: default_max_logs(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            level: default_log_level(),
            file_path: None,
            console: false,
            max_file_size: default_log_size(),
            max_files: default_log_files(),
        }
    }
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            on_successful_login: default_true(),
            on_file_upload: default_true(),
            on_commands: Vec::new(),
            on_new_source_ip: false,
            ip_blacklist: Vec::new(),
            ip_whitelist: Vec::new(),
            sound_enabled: false,
            visual_enabled: default_true(),
        }
    }
}

/// Returns possible config file paths in priority order
fn get_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    
    // Current directory
    paths.push(PathBuf::from_str("./config.toml").unwrap());
    
    // XDG config directory
    if let Some(config_dir) = dirs::config_dir() {
        paths.push(config_dir.join("xkippo/config.toml"));
    }
    
    // Home directory
    if let Some(home_dir) = dirs::home_dir() {
        paths.push(home_dir.join(".xkippo.toml"));
    }
    
    // System config directories
    paths.push(PathBuf::from_str("/etc/xkippo/config.toml").unwrap());
    
    paths
}

// Default values

fn default_true() -> bool {
    true
}

fn default_honeypot_name() -> String {
    "Cowrie Honeypot".into()
}

fn default_honeypot_type() -> String {
    "cowrie".into()
}

fn default_theme() -> String {
    "default".into()
}

fn default_refresh_interval() -> u64 {
    250
}

fn default_check_interval() -> u64 {
    1000
}

fn default_history_hours() -> u32 {
    24
}

fn default_terminal_title() -> String {
    "xKippo - Honeypot Monitor".into()
}

fn default_border_type() -> String {
    "rounded".into()
}

fn default_date_format() -> String {
    "%Y-%m-%d".into()
}

fn default_time_format() -> String {
    "%H:%M:%S".into()
}

fn default_max_sessions() -> usize {
    1000
}

fn default_max_logs() -> usize {
    10000
}

fn default_log_level() -> String {
    "info".into()
}

fn default_log_size() -> u64 {
    10
}

fn default_log_files() -> u32 {
    5
}

/// Security analyst features configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAnalystConfig {
    /// Enable security analyst features
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Log retention in days (0 = unlimited)
    #[serde(default)]
    pub log_retention: u32,
}

impl Default for SecurityAnalystConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            log_retention: 0,
        }
    }
}

/// Threat intelligence configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatIntelConfig {
    /// Enable threat intelligence
    #[serde(default)]
    pub enabled: bool,
    /// Auto-update frequency in hours (0 = disable)
    #[serde(default = "default_ti_update_frequency")]
    pub update_frequency: u32,
    /// Directory to store threat intelligence data
    pub data_dir: Option<String>,
    /// Threat intelligence feeds
    #[serde(default)]
    pub feeds: Vec<String>,
}

impl Default for ThreatIntelConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            update_frequency: default_ti_update_frequency(),
            data_dir: None,
            feeds: Vec::new(),
        }
    }
}

/// Malware analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MalwareAnalysisConfig {
    /// Enable basic malware analysis for downloaded files
    #[serde(default)]
    pub enabled: bool,
    /// Maximum file size to analyze (in MB)
    #[serde(default = "default_malware_max_size")]
    pub max_file_size: u32,
    /// Save analysis reports
    #[serde(default = "default_true")]
    pub save_reports: bool,
    /// Analysis report directory
    pub report_dir: Option<String>,
    /// VirusTotal API integration
    #[serde(default)]
    pub virustotal_enabled: bool,
    /// VirusTotal API key
    pub virustotal_api_key: Option<String>,
}

impl Default for MalwareAnalysisConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_file_size: default_malware_max_size(),
            save_reports: default_true(),
            report_dir: None,
            virustotal_enabled: false,
            virustotal_api_key: None,
        }
    }
}

/// SIEM integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SIEMIntegrationConfig {
    /// Enable SIEM integration
    #[serde(default)]
    pub enabled: bool,
    /// SIEM type (elk, splunk, graylog, custom)
    #[serde(default = "default_siem_type")]
    pub siem_type: String,
    /// SIEM endpoint URL
    pub siem_url: Option<String>,
    /// Authentication token (if needed)
    pub auth_token: Option<String>,
    /// Batch size for sending events
    #[serde(default = "default_batch_size")]
    pub batch_size: u32,
    /// Send interval in seconds
    #[serde(default = "default_send_interval")]
    pub send_interval: u32,
}

impl Default for SIEMIntegrationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            siem_type: default_siem_type(),
            siem_url: None,
            auth_token: None,
            batch_size: default_batch_size(),
            send_interval: default_send_interval(),
        }
    }
}

/// Export configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    /// Enable data export
    #[serde(default)]
    pub enabled: bool,
    /// Available export formats
    #[serde(default)]
    pub formats: Vec<String>,
    /// Default export directory
    pub export_dir: Option<String>,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            formats: Vec::new(),
            export_dir: None,
        }
    }
}

/// Dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    /// Dashboard layout type
    #[serde(default = "default_layout")]
    pub layout: String,
    /// Refresh interval in seconds
    #[serde(default = "default_dashboard_refresh")]
    pub refresh_interval: u32,
    /// Show attack map
    #[serde(default = "default_true")]
    pub show_map: bool,
    /// Show statistics panel
    #[serde(default = "default_true")]
    pub show_stats: bool,
    /// Show alerts panel
    #[serde(default = "default_true")]
    pub show_alerts: bool,
    /// Show top attackers panel
    #[serde(default = "default_true")]
    pub show_top_attackers: bool,
    /// Show command cloud
    #[serde(default = "default_true")]
    pub show_command_cloud: bool,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            layout: default_layout(),
            refresh_interval: default_dashboard_refresh(),
            show_map: default_true(),
            show_stats: default_true(),
            show_alerts: default_true(),
            show_top_attackers: default_true(),
            show_command_cloud: default_true(),
        }
    }
}

/// Rules configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulesConfig {
    /// Directory for custom detection rules
    pub rules_dir: Option<String>,
    /// Auto-reload rules on change
    #[serde(default = "default_true")]
    pub auto_reload: bool,
    /// Enable correlation engine
    #[serde(default)]
    pub enable_correlation: bool,
    /// Minimum risk score for alerts (0-100)
    #[serde(default = "default_min_risk_score")]
    pub min_risk_score: u8,
    /// Alert on new attacker IPs
    #[serde(default)]
    pub alert_new_ips: bool,
}

impl Default for RulesConfig {
    fn default() -> Self {
        Self {
            rules_dir: None,
            auto_reload: default_true(),
            enable_correlation: false,
            min_risk_score: default_min_risk_score(),
            alert_new_ips: false,
        }
    }
}

// Additional default values for security analyst features

fn default_ti_update_frequency() -> u32 {
    24
}

fn default_malware_max_size() -> u32 {
    5
}

fn default_siem_type() -> String {
    "elk".into()
}

fn default_batch_size() -> u32 {
    100
}

fn default_send_interval() -> u32 {
    30
}

fn default_layout() -> String {
    "standard".into()
}

fn default_dashboard_refresh() -> u32 {
    10
}

fn default_min_risk_score() -> u8 {
    50
}