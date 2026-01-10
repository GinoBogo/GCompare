//! State management module for application state.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

use crate::libs::theme::{get_background_color_hex, get_color_hex};
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
            // Color settings - using CSS-based theme functions
            text_diff_remove_bg: get_background_color_hex("text-diff-remove"),
            text_diff_remove_fg: get_color_hex("text-diff-remove"),
            text_diff_add_bg: get_background_color_hex("text-diff-add"),
            text_diff_add_fg: get_color_hex("text-diff-add"),
            text_diff_empty_bg: get_background_color_hex("text-diff-empty"),
            text_diff_empty_fg: get_color_hex("text-diff-empty"),
            merge_conflict_bg: get_background_color_hex("merge-conflict"),
            merge_conflict_fg: get_color_hex("merge-conflict"),
            gutter_numbers_bg: get_background_color_hex("gutter-numbers"),
            gutter_numbers_fg: get_color_hex("gutter-numbers"),
            minimap_bg: get_background_color_hex("minimap"),
            minimap_fg: get_color_hex("minimap"),
            minimap_diff_remove: get_color_hex("minimap-diff-remove"),
            minimap_diff_add: get_color_hex("minimap-diff-add"),
            minimap_diff_empty: get_color_hex("minimap-diff-empty"),
            minimap_cursor_bg: get_background_color_hex("minimap-cursor"),
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
