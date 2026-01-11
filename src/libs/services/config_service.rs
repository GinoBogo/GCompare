//! Configuration service for managing application settings.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::sync::{Arc, Mutex};

use crate::libs::state::AppConfig;

const CONFIG_FILE: &str = "gcompare.json";

/// Service for managing application configuration.
#[derive(Clone)]
pub struct ConfigService {
    config: Arc<Mutex<AppConfig>>,
}

impl ConfigService {
    /// Create a new configuration service and load config from file.
    pub fn new() -> Self {
        let config = Self::load_from_file();
        Self {
            config: Arc::new(Mutex::new(config)),
        }
    }

    /// Load configuration from file (called only once at startup).
    fn load_from_file() -> AppConfig {
        File::open(CONFIG_FILE)
            .ok()
            .and_then(|file| serde_json::from_reader(BufReader::new(file)).ok())
            .unwrap_or_default()
    }

    /// Get current configuration (in-memory, no file I/O).
    pub fn get_config(&self) -> AppConfig {
        self.config.lock().unwrap().clone()
    }

    /// Update configuration in memory (no file I/O).
    pub fn update_config(&self, new_config: AppConfig) {
        *self.config.lock().unwrap() = new_config;
    }

    /// Update specific fields in the configuration in memory.
    pub fn update_merge_window_geometry(&self, width: i32, height: i32, maximized: bool) {
        let mut config = self.config.lock().unwrap();
        config.merge_window_width = width;
        config.merge_window_height = height;
        config.merge_window_maximized = maximized;
    }

    /// Save configuration to file (called only once at shutdown).
    pub fn save_config(&self) {
        let config = self.config.lock().unwrap();
        match File::create(CONFIG_FILE) {
            Ok(file) => {
                let mut writer = BufWriter::new(file);
                match serde_json::to_writer_pretty(&mut writer, &*config) {
                    Ok(_) => {
                        match writer.flush() {
                            Ok(_) => {
                                // Config saved successfully
                            }
                            Err(e) => eprintln!("Failed to flush buffer to {}: {}", CONFIG_FILE, e),
                        }
                    }
                    Err(e) => eprintln!("Failed to write JSON to {}: {}", CONFIG_FILE, e),
                }
            }
            Err(e) => eprintln!("Failed to create config file {}: {}", CONFIG_FILE, e),
        }
    }
}
