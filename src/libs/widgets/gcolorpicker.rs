//! Color picker widget implementation.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

#![allow(dead_code)]

use gtk::prelude::*;
use gtk::{Box, ColorButton, Label, Orientation};
use std::rc::Rc;

/// Color picker widget with label and color button.
pub struct GColorPicker {
    container: Box,
    pub color_button: ColorButton,
    label: String,
}

pub type SharedGColorPicker = Rc<GColorPicker>;

impl GColorPicker {
    /// Create a new color picker widget.
    ///
    /// # Arguments
    ///
    /// * `label` - Label text for the color picker
    /// * `color` - Initial color as hex string (e.g., "#ff0000")
    pub fn new(label: &str, color: &str) -> Self {
        let container = Box::new(Orientation::Horizontal, 10);
        container.set_halign(gtk::Align::Start);
        container.set_hexpand(false);

        let label_widget = Label::new(Some(label));
        label_widget.set_halign(gtk::Align::Start);
        label_widget.set_hexpand(false);

        let color_button = ColorButton::builder().use_alpha(false).build();

        // Parse hex color and set it
        if let Ok(rgba) = Self::parse_hex_color(color) {
            color_button.set_rgba(&rgba);
        }

        container.append(&color_button);
        container.append(&label_widget);

        Self {
            container,
            color_button,
            label: label.to_string(),
        }
    }

    /// Get the container widget.
    pub fn container(&self) -> &Box {
        &self.container
    }

    /// Get the current color as hex string.
    pub fn get_color(&self) -> String {
        let rgba = self.color_button.rgba();
        format!(
            "#{:02x}{:02x}{:02x}",
            (rgba.red() * 255.0) as u8,
            (rgba.green() * 255.0) as u8,
            (rgba.blue() * 255.0) as u8
        )
    }

    /// Set the color from hex string.
    pub fn set_color(&self, color: &str) {
        if let Ok(rgba) = Self::parse_hex_color(color) {
            self.color_button.set_rgba(&rgba);
        }
    }

    /// Connect to color changed signal.
    pub fn connect_color_changed<F: Fn() + 'static>(&self, callback: F) {
        self.color_button.connect_color_set(move |_| {
            callback();
        });
    }

    /// Parse hex color string to RGBA.
    fn parse_hex_color(hex: &str) -> Result<gtk::gdk::RGBA, ()> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return Err(());
        }

        let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| ())?;
        let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| ())?;
        let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| ())?;

        let rgba = gtk::gdk::RGBA::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0);

        Ok(rgba)
    }
}

impl Clone for GColorPicker {
    fn clone(&self) -> Self {
        let color = self.get_color();
        Self::new(&self.label, &color)
    }
}
