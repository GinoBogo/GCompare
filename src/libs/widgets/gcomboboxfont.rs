//! Custom GComboBoxFont widget implementation for font selection with markup support.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

use crate::libs::services::font_service::FontInfo;
use gtk::prelude::*;
use gtk::{CellRendererText, ComboBox, ListStore};
use std::cell::RefCell;
use std::process::Command;

/// Custom font selection combo box with Pango markup support for alias fonts.
#[derive(Clone)]
pub struct GComboBoxFont {
    combo: ComboBox,
    list_store: ListStore,
    font_families: RefCell<Vec<FontInfo>>,
}

impl GComboBoxFont {
    /// Create a new GComboBoxFont widget.
    ///
    /// # Returns
    ///
    /// New GComboBoxFont instance
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
        };

        widget.load_fonts();
        widget
    }

    /// Load system fonts into the combo box.
    fn load_fonts(&self) {
        let mut fonts = Vec::new();

        let source = font_kit::source::SystemSource::new();
        if let Ok(families) = source.all_families() {
            for family in families {
                if let Ok(family_handle) = source.select_family_by_name(&family) {
                    let handles = family_handle.fonts();
                    if !handles.is_empty() {
                        if let Ok(font) = handles[0].load() {
                            if font.is_monospace() {
                                fonts.push(FontInfo::new(family, false));
                            }
                        }
                    }
                }
            }
        }

        // If we didn't find enough fonts, try a broader search using fc-list
        let output = if fonts.len() < 5 {
            let mut cmd = Command::new("fc-list");
            cmd.args([":family", ":style"]);
            #[cfg(target_os = "windows")]
            {
                use std::os::windows::process::CommandExt;
                cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
            }
            cmd.output().ok()
        } else {
            None
        };

        if let Some(output) = output
            && output.status.success()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let line = line.trim();
                if !line.is_empty() {
                    // Parse format like "Font Name:style=Style" or "/path/to/font: Font Name:style=Style"
                    let font_name = if line.contains('/') {
                        // Extract font name from path format
                        line.split(':').nth(1).unwrap_or("").trim()
                    } else {
                        // Extract font name from simple format
                        line.split(':').next().unwrap_or("").trim()
                    };

                    // Check if this might be a monospace font and is not a style variant
                    if (font_name.to_lowercase().contains("mono")
                        || font_name.to_lowercase().contains("code")
                        || font_name.to_lowercase().contains("console")
                        || font_name.to_lowercase().contains("terminal"))
                        && !fonts.iter().any(|f: &FontInfo| f.name == font_name)
                    {
                        fonts.push(FontInfo::new(font_name.to_string(), false));
                    }
                }
            }
        }

        // Add common fallback monospace fonts
        let common_monospace = vec![
            "Monospace",
            "Courier New",
            "Consolas",
            "Menlo",
            "DejaVu Sans Mono",
            "Ubuntu Mono",
            "Source Code Pro",
        ];

        for font in common_monospace {
            if !fonts.iter().any(|f| f.name == font) {
                fonts.push(FontInfo::new(font.to_string(), true));
            }
        }

        fonts.sort_by(|a, b| a.name.cmp(&b.name));
        let font_families = fonts;

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

    /// Get currently selected font name (clean, without markup).
    ///
    /// # Returns
    ///
    /// Option containing the selected font name or None
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

    /// Get the underlying ComboBox widget.
    ///
    /// # Returns
    ///
    /// Reference to the ComboBox widget
    pub fn combo_box(&self) -> &ComboBox {
        &self.combo
    }
}

impl Default for GComboBoxFont {
    /// Create a default GComboBoxFont instance.
    ///
    /// # Returns
    ///
    /// New GComboBoxFont instance
    fn default() -> Self {
        Self::new()
    }
}
