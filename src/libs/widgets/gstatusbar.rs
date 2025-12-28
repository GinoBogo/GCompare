//! Status bar widget implementation.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

#![allow(dead_code)]

use gtk::prelude::*;
use gtk::subclass::prelude::ObjectSubclassIsExt;
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

    /// Update the status bar A label with file and diff information.
    pub fn set_status_a_complete(
        &self,
        bytes: usize,
        lines: usize,
        regular_diffs: usize,
        empty_diffs: usize,
    ) {
        let status = match (regular_diffs, empty_diffs) {
            (0, 0) => format!("{} bytes, {} lines", bytes, lines),
            (0, empty) => format!(
                "{} bytes, {} lines, {} empty line{}",
                bytes,
                lines,
                empty,
                if empty == 1 { "" } else { "s" }
            ),
            (regular, 0) => format!(
                "{} bytes, {} lines, {} differing line{}",
                bytes,
                lines,
                regular,
                if regular == 1 { "" } else { "s" }
            ),
            (regular, empty) => format!(
                "{} bytes, {} lines, {} differing, {} empty",
                bytes, lines, regular, empty
            ),
        };
        self.status_bar_a.set_text(&status);
    }

    /// Update the status bar B label with file and diff information.
    pub fn set_status_b_complete(
        &self,
        bytes: usize,
        lines: usize,
        regular_diffs: usize,
        empty_diffs: usize,
    ) {
        let status = match (regular_diffs, empty_diffs) {
            (0, 0) => format!("{} bytes, {} lines", bytes, lines),
            (0, empty) => format!(
                "{} bytes, {} lines, {} empty line{}",
                bytes,
                lines,
                empty,
                if empty == 1 { "" } else { "s" }
            ),
            (regular, 0) => format!(
                "{} bytes, {} lines, {} differing line{}",
                bytes,
                lines,
                regular,
                if regular == 1 { "" } else { "s" }
            ),
            (regular, empty) => format!(
                "{} bytes, {} lines, {} differing, {} empty",
                bytes, lines, regular, empty
            ),
        };
        self.status_bar_b.set_text(&status);
    }

    /// Update status with current buffer and diff information
    pub fn update_status_from_buffers(
        &self,
        panel_a_buffer: &gtk::TextBuffer,
        panel_b_buffer: &gtk::TextBuffer,
        diff_map: &crate::libs::widgets::gdiffmap::GDiffMap,
    ) {
        // Update file info for panel A
        let (start_a, end_a) = panel_a_buffer.bounds();
        let text_a = panel_a_buffer.text(&start_a, &end_a, true);
        let bytes_a = text_a.len();
        let lines_a = panel_a_buffer.line_count() as usize;

        // Update file info for panel B
        let (start_b, end_b) = panel_b_buffer.bounds();
        let text_b = panel_b_buffer.text(&start_b, &end_b, true);
        let bytes_b = text_b.len();
        let lines_b = panel_b_buffer.line_count() as usize;

        // Get diff line counts (separate regular and empty)
        let imp = diff_map.imp();
        let regular_diffs_a = imp.diff_lines_a.borrow().len();
        let empty_diffs_a = imp.empty_lines_a.borrow().len();
        let regular_diffs_b = imp.diff_lines_b.borrow().len();
        let empty_diffs_b = imp.empty_lines_b.borrow().len();

        // Update status bars
        self.set_status_a_complete(bytes_a, lines_a, regular_diffs_a, empty_diffs_a);
        self.set_status_b_complete(bytes_b, lines_b, regular_diffs_b, empty_diffs_b);
    }
}
