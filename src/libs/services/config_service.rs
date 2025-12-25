//! Configuration service for managing application settings.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

use std::fs::File;
use std::io::BufReader;

use crate::libs::state::AppConfig;

const CONFIG_FILE: &str = "gcompare.json";

/// Service for managing application configuration.
#[derive(Clone)]
pub struct ConfigService;

impl ConfigService {
    /// Create a new configuration service.
    pub fn new() -> Self {
        Self
    }

    /// Load application configuration from file.
    pub fn load_config(&self) -> AppConfig {
        File::open(CONFIG_FILE)
            .ok()
            .and_then(|file| serde_json::from_reader(BufReader::new(file)).ok())
            .unwrap_or_default()
    }

    /// Save application configuration to file.
    pub fn save_config(&self, config: &AppConfig) {
        if let Ok(file) = File::create(CONFIG_FILE) {
            let _ = serde_json::to_writer_pretty(file, config);
        }
    }
}
