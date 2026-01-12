//! Custom control panel widget implementation.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

use crate::libs::widgets::gbutton::{ButtonTheme, GButton};
use gtk::Box;
use gtk::prelude::*;

/// Control panel widget containing action buttons.
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
    pub fn new() -> Self {
        let container = Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(6)
            .halign(gtk::Align::Center)
            .build();

        // Create buttons
        let load_button = GButton::new("Load");
        load_button.set_tooltip_text(Some("Load files from disk"));
        load_button.set_theme(ButtonTheme::Primary);
        container.append(&load_button);

        let compare_button = GButton::new("Compare");
        compare_button.set_tooltip_text(Some("Compare the two files and show differences"));
        compare_button.set_theme(ButtonTheme::Primary);
        container.append(&compare_button);

        let merge_button = GButton::new("Merge");
        merge_button.set_tooltip_text(Some("Open merge view to combine files"));
        merge_button.set_theme(ButtonTheme::Primary);
        container.append(&merge_button);

        let previous_button = GButton::new(" Prev ▲");
        previous_button.set_tooltip_text(Some("Navigate to previous difference"));
        previous_button.set_theme(ButtonTheme::Highlight);
        container.append(&previous_button);

        let next_button = GButton::new(" Next ▼");
        next_button.set_tooltip_text(Some("Navigate to next difference"));
        next_button.set_theme(ButtonTheme::Highlight);
        container.append(&next_button);

        // Add Auto-Compare toggle button
        let auto_compare_button = GButton::new("Auto: ON");
        auto_compare_button.set_tooltip_text(Some("Toggle automatic comparison on file changes"));
        auto_compare_button.set_theme(ButtonTheme::Primary);
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

    /// Get the container widget.
    pub fn container(&self) -> &Box {
        &self.container
    }
}
