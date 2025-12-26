//! GOptions dialog implementation.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

use crate::libs::services::font_service::{FontInfo, FontService};
use crate::libs::widgets::gbutton::{ButtonTheme, GButton};
use gtk::prelude::*;
use gtk::{
    ApplicationWindow, CellRendererText, ComboBox, Dialog, Grid, Label, ListStore, Notebook,
    ScrolledWindow, SpinButton, TextView,
};

pub struct GOptionsDlg {
    dialog: Dialog,
    font_family_combo: ComboBox,
    font_family_model: ListStore,
    font_size_spin: SpinButton,
    apply_button: GButton,
    cancel_button: GButton,
}

impl GOptionsDlg {
    pub fn new(
        parent: &ApplicationWindow,
        current_font_family: &str,
        current_font_size: f64,
    ) -> Self {
        let dialog = Dialog::new();
        dialog.set_title(Some("GCompare - Options"));
        dialog.set_modal(true);
        dialog.set_transient_for(Some(parent));

        dialog.set_default_size(640, 480);
        dialog.set_size_request(640, 480);

        let content_area = dialog.content_area();
        let main_grid = Grid::new();
        main_grid.set_row_spacing(10);
        main_grid.set_column_spacing(10);
        main_grid.set_margin_top(10);
        main_grid.set_margin_bottom(10);
        main_grid.set_margin_start(10);
        main_grid.set_margin_end(10);
        main_grid.set_halign(gtk::Align::Fill);
        main_grid.set_hexpand(true);

        // Row 0: Tabs
        let notebook = Notebook::new();
        notebook.set_halign(gtk::Align::Fill);
        notebook.set_hexpand(true);
        notebook.set_valign(gtk::Align::Fill);
        notebook.set_vexpand(true);

        // Fonts tab
        let fonts_grid = Grid::new();
        fonts_grid.set_row_spacing(10);
        fonts_grid.set_column_spacing(10);
        fonts_grid.set_margin_top(10);
        fonts_grid.set_margin_bottom(10);
        fonts_grid.set_margin_start(10);
        fonts_grid.set_margin_end(10);

        let font_family_label = Label::new(Some("Font Family:"));
        font_family_label.set_halign(gtk::Align::End);
        fonts_grid.attach(&font_family_label, 0, 0, 1, 1);

        // Create a custom ComboBox with ListStore for Pango markup support
        let list_store = ListStore::new(&[
            gtk::glib::Type::STRING, // Display name (with markup)
            gtk::glib::Type::STRING, // Clean font name
        ]);

        let font_family_combo = ComboBox::with_model(&list_store);
        let renderer = CellRendererText::new();
        font_family_combo.pack_start(&renderer, true);
        font_family_combo.add_attribute(&renderer, "markup", 0);

        // Get system monospace fonts
        let font_service = FontService::new();
        let font_families = font_service.get_monospace_fonts();

        // Add fonts to the model
        for font_info in font_families.iter() {
            // Use Pango markup to show alias fonts in gray
            let display_text = if font_info.is_alias {
                format!("<span foreground=\"gray\">{}</span>", font_info.name)
            } else {
                font_info.name.clone()
            };

            let iter = list_store.append();
            list_store.set_value(&iter, 0, &gtk::glib::Value::from(&display_text));
            list_store.set_value(&iter, 1, &gtk::glib::Value::from(&font_info.name));
        }

        // Set current font family, find the best match if not in the list
        let best_match_font = if font_families
            .iter()
            .any(|f: &FontInfo| f.name == current_font_family)
        {
            current_font_family.to_string()
        } else {
            font_service.get_best_monospace_match(current_font_family)
        };

        let active_index = if let Some(index) = font_families
            .iter()
            .position(|f: &FontInfo| f.name == best_match_font)
        {
            Some(index as u32)
        } else if let Some(index) = font_families
            .iter()
            .position(|f: &FontInfo| f.name == "Monospace")
        {
            Some(index as u32)
        } else {
            Some(0)
        };
        font_family_combo.set_active(active_index);

        fonts_grid.attach(&font_family_combo, 1, 0, 1, 1);

        let font_size_label = Label::new(Some("Font Size:"));
        font_size_label.set_halign(gtk::Align::End);
        fonts_grid.attach(&font_size_label, 0, 1, 1, 1);

        let font_size_spin = SpinButton::with_range(8.0, 32.0, 1.0);
        font_size_spin.set_value(current_font_size);
        fonts_grid.attach(&font_size_spin, 1, 1, 1, 1);

        // Font Example
        let font_example_label = Label::new(Some("Font Example:"));
        font_example_label.set_halign(gtk::Align::End);
        fonts_grid.attach(&font_example_label, 0, 2, 1, 1);

        let font_example_text_view = TextView::new();
        font_example_text_view.set_buffer(Some(&gtk::TextBuffer::new(None)));
        font_example_text_view.buffer().set_text("The quick brown fox jumps over the lazy dog\n1234567890\nABCDEFGHIJKLMNOPQRSTUVWXYZ\nabcdefghijklmnopqrstuvwxyz");
        font_example_text_view.set_editable(true);
        font_example_text_view.set_wrap_mode(gtk::WrapMode::Word);
        font_example_text_view.set_hexpand(true);
        font_example_text_view.set_vexpand(true);

        // Create scrolled window for font example
        let font_example_scrolled = ScrolledWindow::new();
        font_example_scrolled.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
        font_example_scrolled.set_hexpand(true);
        font_example_scrolled.set_vexpand(true);
        font_example_scrolled.set_child(Some(&font_example_text_view));

        // Apply current font settings using CSS
        let css_provider = gtk::CssProvider::new();
        let css = format!(
            "textview {{ font-family: {}; font-size: {}pt; }}",
            best_match_font, current_font_size
        );
        css_provider.load_from_data(&css);
        font_example_text_view
            .style_context()
            .add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

        // Setup font change handlers
        let font_size_spin_clone = font_size_spin.clone();
        let css_provider_clone = css_provider.clone();
        let list_store_clone = list_store.clone();
        font_family_combo.connect_changed(move |combo| {
            if let Some(active_iter) = combo.active_iter() {
                if let Ok(font_family) = list_store_clone.get_value(&active_iter, 1).get::<String>()
                {
                    let font_size = font_size_spin_clone.value();
                    let css = format!(
                        "textview {{ font-family: {}; font-size: {}pt; }}",
                        font_family, font_size
                    );
                    css_provider_clone.load_from_data(&css);
                }
            }
        });

        let font_family_combo_clone2 = font_family_combo.clone();
        let css_provider_clone2 = css_provider.clone();
        let list_store_clone2 = list_store.clone();
        font_size_spin.connect_value_changed(move |spin| {
            let font_size = spin.value();
            if let Some(active_iter) = font_family_combo_clone2.active_iter() {
                if let Ok(font_family) =
                    list_store_clone2.get_value(&active_iter, 1).get::<String>()
                {
                    let css = format!(
                        "textview {{ font-family: {}; font-size: {}pt; }}",
                        font_family, font_size
                    );
                    css_provider_clone2.load_from_data(&css);
                }
            }
        });

        fonts_grid.attach(&font_example_scrolled, 1, 3, 1, 1);

        notebook.append_page(&fonts_grid, Some(&Label::new(Some("Fonts"))));

        // Colors tab (placeholder)
        let colors_grid = Grid::new();
        let colors_label = Label::new(Some("Color settings coming soon..."));
        colors_grid.attach(&colors_label, 0, 0, 1, 1);
        notebook.append_page(&colors_grid, Some(&Label::new(Some("Colors"))));

        main_grid.attach(&notebook, 0, 0, 1, 1);

        // Row 1: Buttons
        let button_grid = Grid::new();
        button_grid.set_column_spacing(10);
        button_grid.set_halign(gtk::Align::Center);

        let apply_button = GButton::new("Apply");
        apply_button.set_theme(ButtonTheme::Primary);
        let cancel_button = GButton::new("Cancel");
        cancel_button.set_theme(ButtonTheme::Secondary);

        button_grid.attach(&apply_button, 0, 0, 1, 1);
        button_grid.attach(&cancel_button, 1, 0, 1, 1);

        main_grid.attach(&button_grid, 0, 1, 1, 1);

        content_area.append(&main_grid);

        Self {
            dialog,
            font_family_combo,
            font_family_model: list_store,
            font_size_spin,
            apply_button,
            cancel_button,
        }
    }

    pub fn show(&self, callback: impl Fn(Option<(String, f64)>) + 'static) {
        let font_family_clone = self.font_family_combo.clone();
        let font_family_model_clone = self.font_family_model.clone();
        let font_size_clone = self.font_size_spin.clone();
        let dialog_clone = self.dialog.clone();
        let callback = std::sync::Arc::new(callback);

        // Apply button handler
        let callback_apply = callback.clone();
        self.apply_button.connect_clicked(move |_| {
            let font_family = if let Some(active_iter) = font_family_clone.active_iter() {
                font_family_model_clone
                    .get_value(&active_iter, 1)
                    .get::<String>()
                    .unwrap_or_else(|_| "Monospace".to_string())
            } else {
                "Monospace".to_string()
            };

            let font_size = font_size_clone.value();
            callback_apply(Some((font_family, font_size)));
            dialog_clone.destroy();
        });

        // Cancel button handler
        let dialog_clone_cancel = self.dialog.clone();
        let callback_cancel = callback.clone();
        self.cancel_button.connect_clicked(move |_| {
            callback_cancel(None);
            dialog_clone_cancel.destroy();
        });

        self.dialog.show();
    }
}
