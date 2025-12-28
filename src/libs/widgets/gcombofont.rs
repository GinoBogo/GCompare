//! Custom GComboFont widget implementation for font selection with markup support.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

#![allow(dead_code)]

use crate::libs::services::font_service::{FontInfo, FontService};
use gtk::prelude::*;
use gtk::{CellRendererText, ComboBox, ListStore};
use std::cell::RefCell;

/// Custom font selection combo box with Pango markup support for alias fonts.
#[derive(Clone)]
pub struct GComboFont {
    combo: ComboBox,
    list_store: ListStore,
    font_families: RefCell<Vec<FontInfo>>,
    font_service: RefCell<FontService>,
}

impl GComboFont {
    /// Create a new GComboFont widget.
    pub fn new() -> Self {
        let list_store = ListStore::new(&[
            gtk::glib::Type::STRING, // Display name (with markup)
            gtk::glib::Type::STRING, // Clean font name
        ]);

        let combo = ComboBox::with_model(&list_store);
        let renderer = CellRendererText::new();
        combo.pack_start(&renderer, true);
        combo.add_attribute(&renderer, "markup", 0);

        let widget = Self {
            combo,
            list_store,
            font_families: RefCell::new(Vec::new()),
            font_service: RefCell::new(FontService::new()),
        };

        widget.load_fonts();
        widget
    }

    /// Create a new GComboFont widget with custom font service.
    ///
    /// # Arguments
    ///
    /// * `font_service` - Custom font service instance
    pub fn with_font_service(font_service: FontService) -> Self {
        let list_store = ListStore::new(&[
            gtk::glib::Type::STRING, // Display name (with markup)
            gtk::glib::Type::STRING, // Clean font name
        ]);

        let combo = ComboBox::with_model(&list_store);
        let renderer = CellRendererText::new();
        combo.pack_start(&renderer, true);
        combo.add_attribute(&renderer, "markup", 0);

        let widget = Self {
            combo,
            list_store,
            font_families: RefCell::new(Vec::new()),
            font_service: RefCell::new(font_service),
        };

        widget.load_fonts();
        widget
    }

    /// Load system fonts into the combo box.
    fn load_fonts(&self) {
        let font_service = self.font_service.borrow();
        let font_families = font_service.get_monospace_fonts();

        // Clear existing items
        self.list_store.clear();

        // Add fonts to the model
        for font_info in font_families.iter() {
            // Use Pango markup to show alias fonts in gray
            let display_text = if font_info.is_alias {
                format!("<span foreground=\"gray\">{}</span>", font_info.name)
            } else {
                font_info.name.clone()
            };

            let values: [gtk::glib::Value; 2] = [
                gtk::glib::Value::from(&display_text),
                gtk::glib::Value::from(&font_info.name),
            ];
            self.list_store.set(
                &self.list_store.append(),
                &[(0, &values[0]), (1, &values[1])],
            );
        }

        // Store font families for later use
        *self.font_families.borrow_mut() = font_families;
    }

    /// Get the currently selected font name (clean, without markup).
    pub fn active_font(&self) -> Option<String> {
        if let Some(active_iter) = self.combo.active_iter() {
            self.list_store
                .get_value(&active_iter, 1)
                .get::<String>()
                .ok()
        } else {
            None
        }
    }

    /// Set the active font by font name.
    ///
    /// # Arguments
    ///
    /// * `font_name` - Font name to select
    ///
    /// # Returns
    ///
    /// * `bool` - True if font was found and selected, false otherwise
    pub fn set_active_font(&self, font_name: &str) -> bool {
        let font_families = self.font_families.borrow();

        // Find the font in our stored list
        if let Some((index, _)) = font_families
            .iter()
            .enumerate()
            .find(|(_, font)| font.name == font_name)
        {
            self.combo.set_active(Some(index as u32));
            true
        } else {
            false
        }
    }

    /// Get all available font information.
    ///
    /// # Returns
    ///
    /// * `Vec<FontInfo>` - List of all available fonts with alias information
    pub fn available_fonts(&self) -> Vec<FontInfo> {
        self.font_families.borrow().clone()
    }

    /// Reload fonts from the system font service.
    pub fn reload_fonts(&self) {
        self.load_fonts();
    }

    /// Get the font service instance.
    ///
    /// # Returns
    ///
    /// * `FontService` - The font service used by this widget
    pub fn font_service(&self) -> FontService {
        self.font_service.borrow().clone()
    }

    /// Get the underlying ComboBox widget.
    pub fn combo_box(&self) -> &ComboBox {
        &self.combo
    }
}

impl Default for GComboFont {
    fn default() -> Self {
        Self::new()
    }
}
