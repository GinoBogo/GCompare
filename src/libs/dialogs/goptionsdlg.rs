//! Custom options dialog implementation.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

use crate::libs::services::font_service::{FontInfo, FontService};
use crate::libs::widgets::gbutton::{ButtonTheme, GButton};
use crate::libs::widgets::gcolorpicker::GColorPicker;
use crate::libs::widgets::gcombofont::GComboFont;
use gtk::prelude::*;
use gtk::{
    ApplicationWindow, CheckButton, Dialog, Grid, Label, Notebook, ScrolledWindow, SpinButton,
    TextView,
};

pub struct GOptionsDlg {
    dialog: Dialog,
    font_family_combo: GComboFont,
    font_size_spin: SpinButton,
    auto_compare_check: CheckButton,
    sync_scroll_check: CheckButton,
    ignore_whitespace_check: CheckButton,
    apply_button: GButton,
    cancel_button: GButton,
    // Color pickers
    text_diff_remove_bg_picker: GColorPicker,
    text_diff_remove_fg_picker: GColorPicker,
    text_diff_add_bg_picker: GColorPicker,
    text_diff_add_fg_picker: GColorPicker,
    text_diff_empty_bg_picker: GColorPicker,
    text_diff_empty_fg_picker: GColorPicker,
    gutter_numbers_bg_picker: GColorPicker,
    gutter_numbers_fg_picker: GColorPicker,
    minimap_bg_picker: GColorPicker,
    minimap_fg_picker: GColorPicker,
    minimap_diff_remove_picker: GColorPicker,
    minimap_diff_add_picker: GColorPicker,
    minimap_diff_empty_picker: GColorPicker,
    minimap_cursor_bg_picker: GColorPicker,
}

impl GOptionsDlg {
    /// Create a new options dialog.
    ///
    /// # Arguments
    ///
    /// * `parent` - Parent application window
    /// * `current_font_family` - Currently selected font family
    /// * `current_font_size` - Currently selected font size
    /// * `config` - Application configuration containing color settings
    pub fn new(
        parent: &ApplicationWindow,
        current_font_family: &str,
        current_font_size: f64,
        config: &crate::libs::state::AppConfig,
    ) -> Self {
        let dialog = Dialog::new();
        dialog.set_title(Some("GCompare - Options"));
        dialog.set_modal(true);
        dialog.set_transient_for(Some(parent));

        dialog.set_default_size(480, 480);
        dialog.set_size_request(480, 480);

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

        // Create GComboFont widget for font selection
        let font_family_combo = GComboFont::new();

        // Get system monospace fonts
        let font_service = FontService::new();
        let font_families = font_service.get_monospace_fonts();

        // Set current font family, find the best match if not in the list
        let best_match_font = if font_families
            .iter()
            .any(|f: &FontInfo| f.name == current_font_family)
        {
            current_font_family.to_string()
        } else {
            font_service.get_best_monospace_match(current_font_family)
        };

        // Find and select the font in the combobox by iterating the model
        let combo = font_family_combo.combo_box();
        let mut found = false;

        if let Some(model) = combo.model() {
            for target in [best_match_font.as_str(), "Monospace"] {
                if let Some(iter) = model.iter_first() {
                    loop {
                        let name: Option<String> = model.get(&iter, 1);
                        if let Some(name) = name {
                            if name == target {
                                combo.set_active_iter(Some(&iter));
                                found = true;
                                break;
                            }
                        }
                        if !model.iter_next(&iter) {
                            break;
                        }
                    }
                }
                if found {
                    break;
                }
            }
        }

        if !found {
            combo.set_active(Some(0));
        }

        fonts_grid.attach(font_family_combo.combo_box(), 1, 0, 1, 1);

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
            "textview {{ font-family: \"{}\"; font-size: {}pt; }}",
            best_match_font, current_font_size
        );
        css_provider.load_from_data(&css);
        font_example_text_view
            .style_context()
            .add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

        // Setup font change handlers
        let font_family_combo_clone1 = font_family_combo.clone();
        let font_size_spin_clone = font_size_spin.clone();
        let css_provider_clone = css_provider.clone();
        font_family_combo
            .combo_box()
            .connect_changed(move |_combo| {
                if let Some(font_family) = font_family_combo_clone1.active_font() {
                    let font_size = font_size_spin_clone.value();
                    let css = format!(
                        "textview {{ font-family: \"{}\"; font-size: {}pt; }}",
                        font_family, font_size
                    );
                    css_provider_clone.load_from_data(&css);
                }
            });

        let font_family_combo_clone2 = font_family_combo.clone();
        let css_provider_clone2 = css_provider.clone();
        font_size_spin.connect_value_changed(move |spin| {
            let font_size = spin.value();
            if let Some(font_family) = font_family_combo_clone2.active_font() {
                let css = format!(
                    "textview {{ font-family: \"{}\"; font-size: {}pt; }}",
                    font_family, font_size
                );
                css_provider_clone2.load_from_data(&css);
            }
        });

        fonts_grid.attach(&font_example_scrolled, 1, 3, 1, 1);

        notebook.append_page(&fonts_grid, Some(&Label::new(Some("Fonts"))));

        // Display tab
        let display_grid = Grid::new();
        display_grid.set_row_spacing(10);
        display_grid.set_column_spacing(10);
        display_grid.set_margin_top(10);
        display_grid.set_margin_bottom(10);
        display_grid.set_margin_start(10);
        display_grid.set_margin_end(10);

        let auto_compare_check = CheckButton::with_label("Automatically compare files on change");
        auto_compare_check.set_active(config.auto_compare);
        display_grid.attach(&auto_compare_check, 0, 0, 1, 1);

        let sync_scroll_check = CheckButton::with_label("Synchronize scrolling");
        sync_scroll_check.set_active(config.sync_scroll);
        display_grid.attach(&sync_scroll_check, 0, 1, 1, 1);

        let ignore_whitespace_check = CheckButton::with_label("Ignore whitespace");
        ignore_whitespace_check.set_active(config.ignore_whitespace);
        display_grid.attach(&ignore_whitespace_check, 0, 2, 1, 1);

        notebook.append_page(&display_grid, Some(&Label::new(Some("Display"))));

        // Colors tab
        let colors_grid = Grid::new();
        colors_grid.set_row_spacing(10);
        colors_grid.set_column_spacing(10);
        colors_grid.set_margin_top(10);
        colors_grid.set_margin_bottom(10);
        colors_grid.set_margin_start(10);
        colors_grid.set_margin_end(10);

        // Text Differences section
        let text_diff_label = Label::new(Some("Text Differences"));
        text_diff_label.set_halign(gtk::Align::Start);
        text_diff_label.set_markup("<b>Text Differences</b>");
        colors_grid.attach(&text_diff_label, 0, 0, 1, 1);

        // Removed Background
        let text_diff_remove_bg_picker =
            GColorPicker::new_simple("Removed Text Background", &config.text_diff_remove_bg);
        colors_grid.attach(text_diff_remove_bg_picker.container(), 0, 1, 1, 1);

        // Removed Text
        let text_diff_remove_fg_picker =
            GColorPicker::new_simple("Removed Text Color", &config.text_diff_remove_fg);
        colors_grid.attach(text_diff_remove_fg_picker.container(), 0, 2, 1, 1);

        // Added Background
        let text_diff_add_bg_picker =
            GColorPicker::new_simple("Added Text Background", &config.text_diff_add_bg);
        colors_grid.attach(text_diff_add_bg_picker.container(), 0, 3, 1, 1);

        // Added Text
        let text_diff_add_fg_picker =
            GColorPicker::new_simple("Added Text Color", &config.text_diff_add_fg);
        colors_grid.attach(text_diff_add_fg_picker.container(), 0, 4, 1, 1);

        // Empty Background
        let text_diff_empty_bg_picker =
            GColorPicker::new_simple("Empty Text Background", &config.text_diff_empty_bg);
        colors_grid.attach(text_diff_empty_bg_picker.container(), 0, 5, 1, 1);

        // Empty Text
        let text_diff_empty_fg_picker =
            GColorPicker::new_simple("Empty Text Color", &config.text_diff_empty_fg);
        colors_grid.attach(text_diff_empty_fg_picker.container(), 0, 6, 1, 1);

        // Gutter Numbers section
        let gutter_numbers_label = Label::new(Some("Gutter Numbers"));
        gutter_numbers_label.set_halign(gtk::Align::Start);
        gutter_numbers_label.set_markup("<b>Gutter Numbers</b>");
        colors_grid.attach(&gutter_numbers_label, 0, 7, 2, 1);

        let gutter_numbers_bg_picker =
            GColorPicker::new_simple("Gutter Background", &config.gutter_numbers_bg);
        colors_grid.attach(gutter_numbers_bg_picker.container(), 0, 8, 1, 1);

        let gutter_numbers_fg_picker =
            GColorPicker::new_simple("Numbers Color", &config.gutter_numbers_fg);
        colors_grid.attach(gutter_numbers_fg_picker.container(), 0, 9, 1, 1);

        // Minimap section
        let minimap_label = Label::new(Some("Minimap"));
        minimap_label.set_halign(gtk::Align::Start);
        minimap_label.set_markup("<b>Minimap</b>");
        colors_grid.attach(&minimap_label, 0, 10, 2, 1);

        let minimap_bg_picker = GColorPicker::new_simple("Minimap Background", &config.minimap_bg);
        colors_grid.attach(minimap_bg_picker.container(), 0, 11, 1, 1);

        let minimap_fg_picker = GColorPicker::new_simple("Vertical Separator", &config.minimap_fg);
        colors_grid.attach(minimap_fg_picker.container(), 0, 12, 1, 1);

        let minimap_diff_remove_picker =
            GColorPicker::new_simple("Removed Lines Color", &config.minimap_diff_remove);
        colors_grid.attach(minimap_diff_remove_picker.container(), 0, 13, 1, 1);

        let minimap_diff_add_picker =
            GColorPicker::new_simple("Added Lines Color", &config.minimap_diff_add);
        colors_grid.attach(minimap_diff_add_picker.container(), 0, 14, 1, 1);

        let minimap_diff_empty_picker =
            GColorPicker::new_simple("Empty Lines Color", &config.minimap_diff_empty);
        colors_grid.attach(minimap_diff_empty_picker.container(), 0, 15, 1, 1);

        let minimap_cursor_bg_picker =
            GColorPicker::new("Cursor Background", &config.minimap_cursor_bg, true);
        colors_grid.attach(minimap_cursor_bg_picker.container(), 0, 16, 1, 1);

        // Create scrolled window for colors tab
        let colors_scrolled = ScrolledWindow::new();
        colors_scrolled.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
        colors_scrolled.set_hexpand(true);
        colors_scrolled.set_vexpand(true);
        colors_scrolled.set_child(Some(&colors_grid));

        notebook.append_page(&colors_scrolled, Some(&Label::new(Some("Colors"))));

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
            font_size_spin,
            auto_compare_check,
            sync_scroll_check,
            ignore_whitespace_check,
            apply_button,
            cancel_button,
            text_diff_remove_bg_picker,
            text_diff_remove_fg_picker,
            text_diff_add_bg_picker,
            text_diff_add_fg_picker,
            text_diff_empty_bg_picker,
            text_diff_empty_fg_picker,
            gutter_numbers_bg_picker,
            gutter_numbers_fg_picker,
            minimap_bg_picker,
            minimap_fg_picker,
            minimap_diff_remove_picker,
            minimap_diff_add_picker,
            minimap_diff_empty_picker,
            minimap_cursor_bg_picker,
        }
    }

    /// Show the dialog and execute callback with result.
    ///
    /// # Arguments
    ///
    /// * `callback` - Function to call with dialog result (font family, font size, color config) or None if cancelled
    pub fn show(
        &self,
        callback: impl Fn(Option<(String, f64, crate::libs::state::AppConfig)>) + 'static,
    ) {
        let font_family_clone = self.font_family_combo.clone();
        let font_size_clone = self.font_size_spin.clone();
        let auto_compare_check_clone = self.auto_compare_check.clone();
        let sync_scroll_check_clone = self.sync_scroll_check.clone();
        let ignore_whitespace_check_clone = self.ignore_whitespace_check.clone();
        let dialog_clone = self.dialog.clone();

        // Clone ColorButtons directly (not GColorPicker) to avoid duplication
        let text_diff_remove_bg_picker = self.text_diff_remove_bg_picker.clone();
        let text_diff_remove_fg_picker = self.text_diff_remove_fg_picker.clone();
        let text_diff_add_bg_picker = self.text_diff_add_bg_picker.clone();
        let text_diff_add_fg_picker = self.text_diff_add_fg_picker.clone();
        let text_diff_empty_bg_picker = self.text_diff_empty_bg_picker.clone();
        let text_diff_empty_fg_picker = self.text_diff_empty_fg_picker.clone();
        let gutter_numbers_bg_picker = self.gutter_numbers_bg_picker.clone();
        let gutter_numbers_fg_picker = self.gutter_numbers_fg_picker.clone();
        let minimap_bg_picker = self.minimap_bg_picker.clone();
        let minimap_fg_picker = self.minimap_fg_picker.clone();
        let minimap_diff_remove_picker = self.minimap_diff_remove_picker.clone();
        let minimap_diff_add_picker = self.minimap_diff_add_picker.clone();
        let minimap_diff_empty_picker = self.minimap_diff_empty_picker.clone();
        let minimap_cursor_bg_picker = self.minimap_cursor_bg_picker.clone();

        let callback = std::sync::Arc::new(callback);

        // Apply button handler
        let callback_apply = callback.clone();
        let font_service = FontService::new();
        self.apply_button.connect_clicked(move |_| {
            let font_family = font_family_clone
                .active_font()
                .unwrap_or_else(|| "Monospace".to_string());

            let font_family = font_service.get_best_monospace_match(&font_family);

            let font_size = font_size_clone.value();
            let auto_compare = auto_compare_check_clone.is_active();
            let sync_scroll = sync_scroll_check_clone.is_active();
            let ignore_whitespace = ignore_whitespace_check_clone.is_active();

            // Create color configuration using current colors from UI
            let color_config = crate::libs::state::AppConfig {
                window_width: 0,         // Not updated in this dialog
                window_height: 0,        // Not updated in this dialog
                window_maximized: false, // Not updated in this dialog
                font_family: font_family.clone(),
                font_size: font_size as i32,
                file_a_history: Vec::new(), // Not updated in this dialog
                file_b_history: Vec::new(), // Not updated in this dialog
                sync_scroll,
                auto_compare,
                ignore_whitespace,
                text_diff_remove_bg: text_diff_remove_bg_picker.get_color(),
                text_diff_remove_fg: text_diff_remove_fg_picker.get_color(),
                text_diff_add_bg: text_diff_add_bg_picker.get_color(),
                text_diff_add_fg: text_diff_add_fg_picker.get_color(),
                text_diff_empty_bg: text_diff_empty_bg_picker.get_color(),
                text_diff_empty_fg: text_diff_empty_fg_picker.get_color(),
                gutter_numbers_bg: gutter_numbers_bg_picker.get_color(),
                gutter_numbers_fg: gutter_numbers_fg_picker.get_color(),
                minimap_bg: minimap_bg_picker.get_color(),
                minimap_fg: minimap_fg_picker.get_color(),
                minimap_diff_remove: minimap_diff_remove_picker.get_color(),
                minimap_diff_add: minimap_diff_add_picker.get_color(),
                minimap_diff_empty: minimap_diff_empty_picker.get_color(),
                minimap_cursor_bg: minimap_cursor_bg_picker.get_color(),
            };

            callback_apply(Some((font_family, font_size, color_config)));
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
