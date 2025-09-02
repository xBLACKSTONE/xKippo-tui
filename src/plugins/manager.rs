use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::config::settings::Settings;
use crate::data::models::LogEntry;
use crate::data::store::DataStore;
use crate::plugins::api::{Plugin, PluginMetadata};
use crate::utils::errors::AppError;

/// Manages plugin loading, initialization, and execution
pub struct PluginManager {
    /// Reference to the data store
    data_store: Arc<DataStore>,
    /// Loaded plugins
    plugins: Mutex<HashMap<String, Box<dyn Plugin>>>,
    /// Plugin settings
    settings: Settings,
    /// Plugin directory
    plugin_dir: String,
    /// Whether plugins are enabled
    enabled: bool,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new(data_store: Arc<DataStore>, settings: &Settings) -> Self {
        Self {
            data_store,
            plugins: Mutex::new(HashMap::new()),
            settings: settings.clone(),
            plugin_dir: settings.plugins.directory.to_string_lossy().to_string(),
            enabled: settings.plugins.enabled,
        }
    }
    
    /// Initialize the plugin manager and load plugins
    pub fn initialize(&self) -> Result<(), AppError> {
        if !self.enabled {
            return Ok(());
        }
        
        // Load built-in plugins
        self.load_built_in_plugins()?;
        
        // Load external plugins
        self.load_external_plugins()?;
        
        Ok(())
    }
    
    /// Load built-in plugins
    fn load_built_in_plugins(&self) -> Result<(), AppError> {
        // In a real implementation, we would instantiate and register built-in plugins here
        // For example:
        //
        // let geo_plugin = GeoIpPlugin::new();
        // self.register_plugin(Box::new(geo_plugin))?;
        
        Ok(())
    }
    
    /// Load external plugins from the plugin directory
    fn load_external_plugins(&self) -> Result<(), AppError> {
        let plugin_dir = Path::new(&self.plugin_dir);
        
        if !plugin_dir.exists() || !plugin_dir.is_dir() {
            return Ok(); // Plugin directory doesn't exist, nothing to load
        }
        
        // In a real implementation, we would load dynamic libraries or scripts from the plugin directory
        // For now, we'll just log the found plugins
        
        for entry in fs::read_dir(plugin_dir).map_err(|e| AppError::PluginLoadError(e.to_string()))? {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("so") {
                    // Here we would load the dynamic library and register the plugin
                    // For example:
                    //
                    // let lib = unsafe { Library::new(path) }.map_err(|e| AppError::PluginLoadError(e.to_string()))?;
                    // let constructor: Symbol<extern fn() -> Box<dyn Plugin>> = unsafe {
                    //     lib.get(b"create_plugin")
                    // }.map_err(|e| AppError::PluginLoadError(e.to_string()))?;
                    // let plugin = constructor();
                    // self.register_plugin(plugin)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Register a plugin
    fn register_plugin(&self, mut plugin: Box<dyn Plugin>) -> Result<(), AppError> {
        let metadata = plugin.metadata();
        let name = metadata.name.clone();
        
        // Check if plugin is enabled in settings
        if !self.settings.plugins.enabled_plugins.contains(&name) {
            return Ok();
        }
        
        // Get plugin-specific settings
        let plugin_settings = self.settings.plugins.plugin_config.get(&name).cloned().unwrap_or_default();
        
        // Initialize the plugin
        plugin.initialize(Arc::clone(&self.data_store), &self.settings)
            .map_err(|e| AppError::PluginInitError(name.clone(), e))?;
        
        // Configure the plugin
        plugin.configure(plugin_settings)
            .map_err(|e| AppError::PluginConfigError(name.clone(), e))?;
        
        // Store the plugin
        let mut plugins = self.plugins.lock().unwrap();
        plugins.insert(name, plugin);
        
        Ok(())
    }
    
    /// Process a log entry with all plugins
    pub fn process_log(&self, log_entry: &LogEntry) {
        if !self.enabled {
            return;
        }
        
        let plugins = self.plugins.lock().unwrap();
        for (name, plugin) in plugins.iter() {
            if let Err(e) = plugin.process_log(log_entry) {
                eprintln!("Error processing log with plugin {}: {}", name, e);
            }
        }
    }
    
    /// Update all plugins
    pub fn update(&self) {
        if !self.enabled {
            return;
        }
        
        let mut plugins = self.plugins.lock().unwrap();
        for (name, plugin) in plugins.iter_mut() {
            if let Err(e) = plugin.update() {
                eprintln!("Error updating plugin {}: {}", name, e);
            }
        }
    }
    
    /// Get metadata for all plugins
    pub fn get_plugin_metadata(&self) -> Vec<PluginMetadata> {
        let plugins = self.plugins.lock().unwrap();
        plugins.values().map(|p| p.metadata()).collect()
    }
    
    /// Check if a plugin is loaded
    pub fn has_plugin(&self, name: &str) -> bool {
        let plugins = self.plugins.lock().unwrap();
        plugins.contains_key(name)
    }
    
    /// Shutdown all plugins
    pub fn shutdown(&self) {
        let mut plugins = self.plugins.lock().unwrap();
        for (name, plugin) in plugins.iter_mut() {
            if let Err(e) = plugin.shutdown() {
                eprintln!("Error shutting down plugin {}: {}", name, e);
            }
        }
    }
}