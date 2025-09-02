use std::sync::Arc;
use serde_json::Value;

use crate::data::store::DataStore;
use crate::data::models::{LogEntry, Alert};
use crate::config::settings::Settings;

/// Plugin metadata
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin author
    pub author: String,
    /// Plugin description
    pub description: String,
    /// Plugin dependencies
    pub dependencies: Vec<String>,
}

/// Plugin trait defining the interface for all plugins
pub trait Plugin: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> PluginMetadata;
    
    /// Initialize the plugin
    fn initialize(&mut self, data_store: Arc<DataStore>, settings: &Settings) -> Result<(), String>;
    
    /// Update the plugin state
    fn update(&mut self) -> Result<(), String>;
    
    /// Process a log entry
    fn process_log(&mut self, log_entry: &LogEntry) -> Result<(), String>;
    
    /// Clean up resources
    fn shutdown(&mut self) -> Result<(), String>;
    
    /// Get configuration schema
    fn config_schema(&self) -> Value;
    
    /// Configure the plugin with the provided settings
    fn configure(&mut self, settings: Value) -> Result<(), String>;
}

/// Data source plugin trait
pub trait DataSourcePlugin: Plugin {
    /// Start collecting data
    fn start_collection(&mut self) -> Result<(), String>;
    
    /// Stop collecting data
    fn stop_collection(&mut self) -> Result<(), String>;
    
    /// Check if the data source is available
    fn is_available(&self) -> bool;
}

/// Analysis plugin trait
pub trait AnalysisPlugin: Plugin {
    /// Analyze data and return results
    fn analyze(&mut self) -> Result<Value, String>;
    
    /// Get supported analysis types
    fn supported_analysis(&self) -> Vec<String>;
}

/// Visualization plugin trait
pub trait VisualizationPlugin: Plugin {
    /// Render visualization to a buffer
    fn render(&self, width: u16, height: u16) -> Result<Vec<Vec<char>>, String>;
    
    /// Get supported visualization types
    fn supported_types(&self) -> Vec<String>;
}

/// Alert plugin trait
pub trait AlertPlugin: Plugin {
    /// Process an alert
    fn process_alert(&mut self, alert: &Alert) -> Result<(), String>;
    
    /// Check if an alert should be triggered
    fn check_alert_condition(&self, log_entry: &LogEntry) -> Option<Alert>;
}