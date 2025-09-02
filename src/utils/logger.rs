use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};
use chrono::Local;
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

/// Custom logger for the application
struct AppLogger {
    /// Log file handle
    file: Option<Arc<Mutex<File>>>,
    /// Log level
    level: Level,
}

impl AppLogger {
    /// Create a new logger instance
    pub fn new(log_file_path: Option<&Path>, level: Level) -> Result<Self, std::io::Error> {
        let file = match log_file_path {
            Some(path) => {
                // Create parent directories if they don't exist
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                
                // Open log file
                let file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)?;
                
                Some(Arc::new(Mutex::new(file)))
            }
            None => None,
        };
        
        Ok(Self {
            file,
            level,
        })
    }
}

impl log::Log for AppLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }
    
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            let message = format!("[{} {} {}] {}\n", 
                now, 
                record.level(),
                record.target(),
                record.args()
            );
            
            // Write to stderr
            eprint!("{}", message);
            
            // Write to file if available
            if let Some(ref file) = self.file {
                if let Ok(mut file) = file.lock() {
                    let _ = file.write_all(message.as_bytes());
                }
            }
        }
    }
    
    fn flush(&self) {
        if let Some(ref file) = self.file {
            if let Ok(mut file) = file.lock() {
                let _ = file.flush();
            }
        }
    }
}

/// Global logger instance
static mut LOGGER: Option<AppLogger> = None;

/// Initialize the logger
pub fn init_logger() -> Result<(), SetLoggerError> {
    // Default log level
    let level = LevelFilter::Info;
    
    unsafe {
        // Create logger instance
        LOGGER = match AppLogger::new(None, Level::Info) {
            Ok(logger) => Some(logger),
            Err(_) => {
                eprintln!("Failed to initialize logger, falling back to stderr only");
                Some(AppLogger::new(None, Level::Info).unwrap())
            }
        };
        
        // Set the global logger
        if let Some(ref logger) = LOGGER {
            log::set_max_level(level);
            log::set_logger(logger)?;
        }
    }
    
    Ok(())
}

/// Initialize the logger with a custom file and level
pub fn init_logger_with_file(log_file_path: &Path, level: Level) -> Result<(), Box<dyn std::error::Error>> {
    let level_filter = match level {
        Level::Error => LevelFilter::Error,
        Level::Warn => LevelFilter::Warn,
        Level::Info => LevelFilter::Info,
        Level::Debug => LevelFilter::Debug,
        Level::Trace => LevelFilter::Trace,
    };
    
    unsafe {
        // Create logger instance
        LOGGER = Some(AppLogger::new(Some(log_file_path), level)?);
        
        // Set the global logger
        if let Some(ref logger) = LOGGER {
            log::set_max_level(level_filter);
            log::set_logger(logger)?;
        }
    }
    
    Ok(())
}