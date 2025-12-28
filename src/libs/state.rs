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
    // Color settings
    pub diff_remove_bg: String,
    pub diff_add_bg: String,
    pub diff_empty_bg: String,
    pub diff_remove_text: String,
    pub diff_add_text: String,
    pub diff_empty_text: String,
    pub gutter_bg: String,
    pub gutter_text: String,
    pub minimap_bg: String,
    pub minimap_separator: String,
    pub minimap_remove: String,
    pub minimap_add: String,
    pub minimap_empty: String,
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
            // Color settings - using hex values from CSS
            diff_remove_bg: "#ffcccc".to_string(),    // Light red
            diff_add_bg: "#ccffcc".to_string(),        // Light green
            diff_empty_bg: "#fffacd".to_string(),      // Light yellow
            diff_remove_text: "#990000".to_string(),   // Dark red
            diff_add_text: "#009900".to_string(),      // Dark green
            diff_empty_text: "#ffcc00".to_string(),    // Dark yellow
            gutter_bg: "#f0f0f0".to_string(),          // Light gray
            gutter_text: "#888888".to_string(),        // Medium gray
            minimap_bg: "#ffffff".to_string(),          // White
            minimap_separator: "#cccccc".to_string(),  // Light gray
            minimap_remove: "#990000".to_string(),      // Dark red
            minimap_add: "#009900".to_string(),         // Dark green
            minimap_empty: "#ffcc00".to_string(),       // Dark yellow
        }
    }
}

/// Centralized application state management.
pub struct ApplicationState {
    config: std::cell::RefCell<AppConfig>,
}

impl ApplicationState {
    /// Create a new application state with the given configuration.
    pub fn new(config: AppConfig) -> Self {
        Self { config: std::cell::RefCell::new(config) }
    }

    /// Get current configuration.
    pub fn config(&self) -> std::cell::Ref<'_, AppConfig> {
        self.config.borrow()
    }

    /// Update the configuration.
    pub fn update_config(&self, config: AppConfig) {
        *self.config.borrow_mut() = config;
    }

    /// Update configuration from UI state.
    pub fn update_config_from_ui(
        &self,
        window: &ApplicationWindow,
        path_combo_a: &ComboBoxText,
        path_combo_b: &ComboBoxText,
    ) -> AppConfig {
        let current_config = self.config.borrow();
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
            font_family: current_config.font_family.clone(),
            font_size: current_config.font_size,
            file_a_history: update_history(
                current_config.file_a_history.clone(),
                get_current_path(path_combo_a),
            ),
            file_b_history: update_history(
                current_config.file_b_history.clone(),
                get_current_path(path_combo_b),
            ),
            sync_scroll: current_config.sync_scroll,
            // Preserve color settings
            diff_remove_bg: current_config.diff_remove_bg.clone(),
            diff_add_bg: current_config.diff_add_bg.clone(),
            diff_empty_bg: current_config.diff_empty_bg.clone(),
            diff_remove_text: current_config.diff_remove_text.clone(),
            diff_add_text: current_config.diff_add_text.clone(),
            diff_empty_text: current_config.diff_empty_text.clone(),
            gutter_bg: current_config.gutter_bg.clone(),
            gutter_text: current_config.gutter_text.clone(),
            minimap_bg: current_config.minimap_bg.clone(),
            minimap_separator: current_config.minimap_separator.clone(),
            minimap_remove: current_config.minimap_remove.clone(),
            minimap_add: current_config.minimap_add.clone(),
            minimap_empty: current_config.minimap_empty.clone(),
        }
    }
}
