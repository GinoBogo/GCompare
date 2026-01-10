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
    pub auto_compare: bool,
    pub sync_scroll: bool,
    pub ignore_whitespace: bool,
    // Color settings
    pub text_diff_remove_bg: String,
    pub text_diff_remove_fg: String,
    pub text_diff_add_bg: String,
    pub text_diff_add_fg: String,
    pub text_diff_empty_bg: String,
    pub text_diff_empty_fg: String,
    pub merge_conflict_bg: String,
    pub merge_conflict_fg: String,
    pub gutter_numbers_bg: String,
    pub gutter_numbers_fg: String,
    pub minimap_bg: String,
    pub minimap_fg: String,
    pub minimap_diff_remove: String,
    pub minimap_diff_add: String,
    pub minimap_diff_empty: String,
    pub minimap_cursor_bg: String,
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
            auto_compare: true,
            sync_scroll: true,
            ignore_whitespace: false,
            // Color settings - using hex values from CSS
            text_diff_remove_bg: "#ffcccc".to_string(), // Light red
            text_diff_remove_fg: "#990000".to_string(), // Dark red
            text_diff_add_bg: "#ccffcc".to_string(),    // Light green
            text_diff_add_fg: "#009900".to_string(),    // Dark green
            text_diff_empty_bg: "#fffacd".to_string(),  // Light yellow
            text_diff_empty_fg: "#ffcc00".to_string(),  // Dark yellow
            merge_conflict_bg: "#00ccff".to_string(),   // Light blue
            merge_conflict_fg: "#000000".to_string(),   // Black
            gutter_numbers_bg: "#f0f0f0".to_string(),   // Light gray
            gutter_numbers_fg: "#888888".to_string(),   // Medium gray
            minimap_bg: "#ffffff".to_string(),          // White
            minimap_fg: "#cccccc".to_string(),          // Light gray
            minimap_diff_remove: "#990000".to_string(), // Dark red
            minimap_diff_add: "#009900".to_string(),    // Dark green
            minimap_diff_empty: "#ffcc00".to_string(),  // Dark yellow
            minimap_cursor_bg: "#00000008".to_string(), // Black with 5% alpha (0x08 ≈ 5%)
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
        Self {
            config: std::cell::RefCell::new(config),
        }
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
            if let Some(path_str) = path
                && !path_str.trim().is_empty()
            {
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
            history
        };

        AppConfig {
            // Save current size only if not maximized, otherwise keep previous
            // size
            window_width: if window.is_maximized() {
                current_config.window_width
            } else {
                window.width()
            },
            window_height: if window.is_maximized() {
                current_config.window_height
            } else {
                window.height()
            },
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
            auto_compare: current_config.auto_compare,
            sync_scroll: current_config.sync_scroll,
            ignore_whitespace: current_config.ignore_whitespace,
            // Preserve color settings
            text_diff_remove_bg: current_config.text_diff_remove_bg.clone(),
            text_diff_remove_fg: current_config.text_diff_remove_fg.clone(),
            text_diff_add_bg: current_config.text_diff_add_bg.clone(),
            text_diff_add_fg: current_config.text_diff_add_fg.clone(),
            text_diff_empty_bg: current_config.text_diff_empty_bg.clone(),
            text_diff_empty_fg: current_config.text_diff_empty_fg.clone(),
            merge_conflict_bg: current_config.merge_conflict_bg.clone(),
            merge_conflict_fg: current_config.merge_conflict_fg.clone(),
            gutter_numbers_bg: current_config.gutter_numbers_bg.clone(),
            gutter_numbers_fg: current_config.gutter_numbers_fg.clone(),
            minimap_bg: current_config.minimap_bg.clone(),
            minimap_fg: current_config.minimap_fg.clone(),
            minimap_diff_remove: current_config.minimap_diff_remove.clone(),
            minimap_diff_add: current_config.minimap_diff_add.clone(),
            minimap_diff_empty: current_config.minimap_diff_empty.clone(),
            minimap_cursor_bg: current_config.minimap_cursor_bg.clone(),
        }
    }
}
