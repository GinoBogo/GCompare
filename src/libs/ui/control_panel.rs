//! Custom control panel widget implementation.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

use crate::libs::widgets::gbutton::GButton;
use gtk::Box;
use gtk::prelude::*;

/// Control panel widget containing action buttons.
#[derive(Clone)]
pub struct ControlPanelWidget {
    container: Box,
    pub load_button: GButton,
    pub compare_button: GButton,
    pub merge_button: GButton,
    pub previous_button: GButton,
    pub next_button: GButton,
    pub auto_compare_button: GButton,
    pub options_button: GButton,
}

impl ControlPanelWidget {
    /// Create a new control panel widget.
    pub fn new(primary_bg: &str, primary_fg: &str, highlight_bg: &str, highlight_fg: &str) -> Self {
        let container = Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(6)
            .halign(gtk::Align::Center)
            .build();

        // Create buttons
        let load_button = GButton::new("Load");
        load_button.set_tooltip_text(Some("Load files from disk"));
        load_button.set_custom_colors(primary_bg, primary_fg);
        container.append(&load_button);

        let compare_button = GButton::new("Compare");
        compare_button.set_tooltip_text(Some("Compare the two files and show differences"));
        compare_button.set_custom_colors(primary_bg, primary_fg);
        container.append(&compare_button);

        let merge_button = GButton::new("Merge");
        merge_button.set_tooltip_text(Some("Open merge view to combine files"));
        merge_button.set_custom_colors(primary_bg, primary_fg);
        container.append(&merge_button);

        let previous_button = GButton::new(" Prev ▲");
        previous_button.set_tooltip_text(Some("Navigate to previous difference"));
        previous_button.set_custom_colors(highlight_bg, highlight_fg);
        container.append(&previous_button);

        let next_button = GButton::new(" Next ▼");
        next_button.set_tooltip_text(Some("Navigate to next difference"));
        next_button.set_custom_colors(highlight_bg, highlight_fg);
        container.append(&next_button);

        // Add Auto-Compare toggle button
        let auto_compare_button = GButton::new("Auto: ON");
        auto_compare_button.set_tooltip_text(Some("Toggle automatic comparison on file changes"));
        auto_compare_button.set_custom_colors(primary_bg, primary_fg);
        container.append(&auto_compare_button);

        let options_button = GButton::new("Options");
        options_button.set_tooltip_text(Some("Open application settings and preferences"));
        container.append(&options_button);

        Self {
            container,
            load_button,
            compare_button,
            merge_button,
            previous_button,
            next_button,
            auto_compare_button,
            options_button,
        }
    }

    /// Update all button colors from config.
    pub fn update_button_colors(&self, config: &crate::libs::state::AppConfig) {
        self.load_button
            .set_custom_colors(&config.button_primary_bg, &config.button_primary_fg);
        self.compare_button
            .set_custom_colors(&config.button_primary_bg, &config.button_primary_fg);
        self.merge_button
            .set_custom_colors(&config.button_primary_bg, &config.button_primary_fg);
        self.previous_button
            .set_custom_colors(&config.button_highlight_bg, &config.button_highlight_fg);
        self.next_button
            .set_custom_colors(&config.button_highlight_bg, &config.button_highlight_fg);
        self.auto_compare_button
            .set_custom_colors(&config.button_primary_bg, &config.button_primary_fg);

        // Don't update Options button - keep its original appearance
    }

    /// Get the container widget.
    pub fn container(&self) -> &Box {
        &self.container
    }
}
