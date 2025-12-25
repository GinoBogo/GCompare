//! Status bar widget implementation.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

use gtk::prelude::*;
use gtk::{Box as GtkBox, Label, Orientation};

/// Status bar widget for displaying application status information.
#[derive(Clone)]
pub struct GStatusBar {
    container: GtkBox,
    status_bar_a: Label,
    status_bar_b: Label,
}

impl GStatusBar {
    /// Create a new status bar widget.
    pub fn new() -> Self {
        let container = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(0)
            .hexpand(true)
            .build();

        // Create containers - ONLY TWO expandable containers for columns 0 and 2
        let container_a = GtkBox::new(Orientation::Horizontal, 0);
        let container_b = GtkBox::new(Orientation::Horizontal, 0);

        // Make only col0 and col2 expand equally
        container_a.set_hexpand(true);
        container_b.set_hexpand(true);

        // Status bar A label
        let status_bar_a_label = Label::builder()
            .label("by Gino Bogo")
            .xalign(0.01)
            .width_chars(80)
            .hexpand(true)
            .build();

        // Status bar center spacer
        let center_spacer = Label::builder()
            .label("")
            .halign(gtk::Align::Center)
            .width_request(40)
            .hexpand(false)
            .build();

        // Status bar B label
        let status_bar_b_label = Label::builder()
            .label("")
            .xalign(0.02)
            .width_chars(80)
            .hexpand(true)
            .build();

        // Add labels to their containers
        container_a.append(&status_bar_a_label);
        container_b.append(&status_bar_b_label);

        // Align the containers themselves
        container_a.set_halign(gtk::Align::Start);
        container_b.set_halign(gtk::Align::Start);

        // Add all to main container
        container.append(&container_a);
        container.append(&center_spacer);
        container.append(&container_b);

        Self {
            container,
            status_bar_a: status_bar_a_label,
            status_bar_b: status_bar_b_label,
        }
    }

    /// Get the container widget.
    pub fn container(&self) -> &GtkBox {
        &self.container
    }

    /// Update the status bar A label.
    pub fn _set_status_a(&self, status: &str) {
        self.status_bar_a.set_text(status);
    }

    /// Update the status bar B label.
    pub fn _set_status_b(&self, status: &str) {
        self.status_bar_b.set_text(status);
    }

    /// Update the status bar A label with file information.
    pub fn set_status_a_file_info(&self, bytes: usize, lines: usize) {
        let status = format!("{} bytes, {} lines", bytes, lines);
        self.status_bar_a.set_text(&status);
    }

    /// Update the status bar B label with file information.
    pub fn set_status_b_file_info(&self, bytes: usize, lines: usize) {
        let status = format!("{} bytes, {} lines", bytes, lines);
        self.status_bar_b.set_text(&status);
    }
}
