//! Custom color picker widget implementation.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

use gtk::prelude::*;
use gtk::{Box, ColorButton, Label, Orientation, Scale};
use std::rc::Rc;

use crate::libs::services::color_parser::parse_color_comprehensive;

/// Color picker widget with label, color button, and transparency slider for alpha control.
pub struct GColorPicker {
    container: Box,
    pub color_button: ColorButton,
    alpha_scale: Option<Scale>,
    label: String,
    alpha_cell: Option<Rc<std::cell::Cell<f64>>>, // Store the alpha cell for the closure
}

impl GColorPicker {
    /// Create a new color picker widget.
    ///
    /// # Arguments
    ///
    /// * `label` - Label text for the color picker
    /// * `color` - Initial color as hex string (6 or 8 digit)
    /// * `with_alpha` - Whether to include alpha slider
    pub fn new(label: &str, color: &str, with_alpha: bool) -> Self {
        let container = Box::new(Orientation::Horizontal, 10);
        container.set_halign(gtk::Align::Start);
        container.set_hexpand(false);

        let color_button = ColorButton::builder().use_alpha(with_alpha).build();
        let label_widget = Label::new(Some(label));
        label_widget.set_halign(gtk::Align::Start);
        label_widget.set_hexpand(false);

        // Parse hex color and set it
        let initial_alpha = if let Ok((_, alpha)) = Self::parse_hex_color(color) {
            alpha
        } else {
            1.0
        };

        if let Ok((rgba, _)) = Self::parse_hex_color(color) {
            color_button.set_rgba(&rgba);
        }

        container.append(&color_button);

        // Add alpha slider if requested
        let (alpha_scale, alpha_cell) = if with_alpha {
            let scale = Scale::builder()
                .orientation(Orientation::Horizontal)
                .adjustment(&gtk::Adjustment::new(100.0, 0.0, 100.0, 1.0, 10.0, 0.0))
                .hexpand(true)
                .build();
            scale.set_digits(0);
            scale.set_value_pos(gtk::PositionType::Right);
            scale.set_size_request(100, -1); // Give it a reasonable width

            // Set initial alpha value
            if let Ok((_, alpha)) = Self::parse_hex_color(color) {
                scale.set_value(alpha * 100.0);
            }

            container.append(&scale);

            // Create alpha cell BEFORE setting up connections
            let alpha_cell = std::rc::Rc::new(std::cell::Cell::new(initial_alpha));

            // Set up connections using the SAME alpha_cell
            let color_button_for_slider = color_button.clone();
            scale.connect_value_changed({
                let alpha_cell = alpha_cell.clone();
                move |scale| {
                    let alpha = scale.value() / 100.0;
                    alpha_cell.set(alpha);
                    let mut rgba = color_button_for_slider.rgba();
                    rgba.set_alpha(alpha as f32);
                    color_button_for_slider.set_rgba(&rgba);
                }
            });

            let scale_clone = scale.clone();
            let color_button_for_sync = color_button.clone();
            color_button_for_sync.clone().connect_color_set(move |_| {
                let rgba = color_button_for_sync.rgba();
                scale_clone.set_value(rgba.alpha() as f64 * 100.0);
            });

            (Some(scale), Some(alpha_cell))
        } else {
            (None, None)
        };

        container.append(&label_widget);

        Self {
            container,
            color_button,
            alpha_scale,
            label: label.to_string(),
            alpha_cell,
        }
    }

    /// Create a new color picker without transparency (legacy compatibility).
    pub fn new_simple(label: &str, color: &str) -> Self {
        Self::new(label, color, false)
    }

    /// Get the container widget.
    pub fn container(&self) -> &Box {
        &self.container
    }

    /// Get the current color as hex string (6 or 8 digit depending on alpha support).
    pub fn get_color(&self) -> String {
        let rgba = self.color_button.rgba();

        if let Some(ref alpha_cell) = self.alpha_cell {
            // Include alpha channel from our stored value
            let alpha = alpha_cell.get();
            format!(
                "#{:02x}{:02x}{:02x}{:02x}",
                (rgba.red() * 255.0) as u8,
                (rgba.green() * 255.0) as u8,
                (rgba.blue() * 255.0) as u8,
                (alpha * 255.0) as u8
            )
        } else {
            // 6-digit hex only
            format!(
                "#{:02x}{:02x}{:02x}",
                (rgba.red() * 255.0) as u8,
                (rgba.green() * 255.0) as u8,
                (rgba.blue() * 255.0) as u8
            )
        }
    }

    /// Parse hex color string to RGBA and alpha using centralized parser.
    /// Supports both 6-digit (#RRGGBB) and 8-digit (#RRGGBBAA) formats.
    fn parse_hex_color(hex: &str) -> Result<(gtk::gdk::RGBA, f64), ()> {
        parse_color_comprehensive(hex)
            .map(|result| (result.rgba, result.alpha))
            .map_err(|_| ())
    }
}

impl Clone for GColorPicker {
    fn clone(&self) -> Self {
        let color = self.get_color();
        let with_alpha = self.alpha_scale.is_some();
        let mut new_picker = Self::new(&self.label, &color, with_alpha);

        // Replace the new picker's alpha_cell with a clone of the original's alpha_cell
        new_picker.alpha_cell = self.alpha_cell.clone();

        // Connect color changes from original to cloned picker
        let original_color_button = self.color_button.clone();
        let cloned_color_button = new_picker.color_button.clone();
        self.color_button.connect_color_set(move |_| {
            cloned_color_button.set_rgba(&original_color_button.rgba());
        });

        // Update the new picker's color button to match the original's current color
        new_picker.color_button.set_rgba(&self.color_button.rgba());

        new_picker
    }
}
