//! Custom control panel widget implementation.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

use gtk::Box as GtkBox;
use gtk::prelude::*;

use crate::libs::widgets::gbutton::{ButtonTheme, GButton};

/// Control panel widget containing action buttons.
pub struct ControlPanelWidget {
    container: GtkBox,
    pub compare_button: GButton,
    pub reload_button: GButton,
    pub previous_button: GButton,
    pub next_button: GButton,
    pub options_button: GButton,
}

impl ControlPanelWidget {
    /// Create a new control panel widget.
    pub fn new() -> Self {
        let container = GtkBox::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(6)
            .halign(gtk::Align::Center)
            .build();

        // Create buttons
        let compare_button = GButton::new("Compare");
        compare_button.set_theme(ButtonTheme::Primary);
        container.append(&compare_button);

        let reload_button = GButton::new("Reload");
        reload_button.set_theme(ButtonTheme::Primary);
        container.append(&reload_button);

        let previous_button = GButton::new(" Prev ▲");
        previous_button.set_theme(ButtonTheme::Highlight);
        container.append(&previous_button);

        let next_button = GButton::new(" Next ▼");
        next_button.set_theme(ButtonTheme::Highlight);
        container.append(&next_button);

        let options_button = GButton::new("Options");
        container.append(&options_button);

        Self {
            container,
            compare_button,
            reload_button,
            previous_button,
            next_button,
            options_button,
        }
    }

    /// Get the container widget.
    pub fn container(&self) -> &GtkBox {
        &self.container
    }
}
