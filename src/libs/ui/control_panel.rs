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
    pub btn_reload: GButton,
    pub btn_compare: GButton,
    pub btn_merge: GButton,
    pub btn_previous: GButton,
    pub btn_next: GButton,
    pub btn_auto_compare: GButton,
    pub btn_options: GButton,
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
        let btn_reload = GButton::new("Reload");
        btn_reload.set_theme(ButtonTheme::Primary);
        container.append(&btn_reload);

        let btn_compare = GButton::new("Compare");
        btn_compare.set_theme(ButtonTheme::Primary);
        container.append(&btn_compare);

        let btn_merge = GButton::new("Merge");
        btn_merge.set_theme(ButtonTheme::Primary);
        container.append(&btn_merge);

        let btn_previous = GButton::new(" Prev ▲");
        btn_previous.set_theme(ButtonTheme::Highlight);
        container.append(&btn_previous);

        let btn_next = GButton::new(" Next ▼");
        btn_next.set_theme(ButtonTheme::Highlight);
        container.append(&btn_next);

        // Add Auto-Compare toggle button
        let btn_auto_compare = GButton::new("Auto: ON");
        btn_auto_compare.set_theme(ButtonTheme::Primary);
        container.append(&btn_auto_compare);

        let btn_options = GButton::new("Options");
        container.append(&btn_options);

        Self {
            container,
            btn_reload,
            btn_compare,
            btn_merge,
            btn_previous,
            btn_next,
            btn_auto_compare,
            btn_options,
        }
    }

    /// Get the container widget.
    pub fn container(&self) -> &Box {
        &self.container
    }
}
