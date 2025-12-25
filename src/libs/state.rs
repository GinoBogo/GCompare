//! State management module for application state.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

use gtk::prelude::*;
use gtk::{ApplicationWindow, ComboBoxText, Entry};
use serde::{Deserialize, Serialize};

/// Holds the application's configuration settings.
#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct AppConfig {
    pub window_width: i32,
    pub window_height: i32,
    pub window_maximized: bool,
    pub font_family: String,
    pub font_size: i32,
    pub file_a_history: Vec<String>,
    pub file_b_history: Vec<String>,
    pub sync_scroll: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            window_width: 800,
            window_height: 600,
            window_maximized: false,
            font_family: "Monospace".to_string(),
            font_size: 12,
            file_a_history: Vec::new(),
            file_b_history: Vec::new(),
            sync_scroll: true,
        }
    }
}

/// Centralized application state management.
pub struct ApplicationState {
    config: AppConfig,
}

impl ApplicationState {
    /// Create a new application state with the given configuration.
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    /// Get current configuration.
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// Update configuration from UI state.
    pub fn update_config_from_ui(
        &self,
        window: &ApplicationWindow,
        path_combo_a: &ComboBoxText,
        path_combo_b: &ComboBoxText,
    ) -> AppConfig {
        // Helper function to get current path from combo box
        let get_current_path = |combo: &ComboBoxText| {
            combo
                .child()
                .and_then(|child| child.downcast::<Entry>().ok())
                .map(|entry| entry.text().to_string())
        };

        // Helper function to update history
        let update_history = |mut history: Vec<String>, path: Option<String>| {
            if let Some(path_str) = path {
                if !path_str.trim().is_empty() {
                    // Remove existing entry if present
                    if let Some(position) = history.iter().position(|x| x == &path_str) {
                        history.remove(position);
                    }
                    // Add to beginning of history
                    history.insert(0, path_str);
                    // Keep history to reasonable size
                    if history.len() > 10 {
                        history.truncate(10);
                    }
                }
            }
            history
        };

        AppConfig {
            window_width: window.default_width(),
            window_height: window.default_height(),
            window_maximized: window.is_maximized(),
            font_family: self.config.font_family.clone(),
            font_size: self.config.font_size,
            file_a_history: update_history(
                self.config.file_a_history.clone(),
                get_current_path(path_combo_a),
            ),
            file_b_history: update_history(
                self.config.file_b_history.clone(),
                get_current_path(path_combo_b),
            ),
            sync_scroll: self.config.sync_scroll,
        }
    }
}
