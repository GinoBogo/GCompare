//! Custom comparison panels widget implementation.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

use gtk::prelude::*;
use gtk::{Box, ComboBoxText, CssProvider, Frame};

use crate::libs::state::AppConfig;
use crate::libs::widgets::gbutton::{ButtonTheme, GButton};
use crate::libs::widgets::gdiffmap::GDiffMap;
use crate::libs::widgets::gtextview::GTextView;

/// Widget containing file comparison panels and diff map.
pub struct ComparisonPanelsWidget {
    container: Box,
    panel_a: FilePanelWidget,
    panel_b: FilePanelWidget,
    diff_map: GDiffMap,
}

impl ComparisonPanelsWidget {
    /// Create a new comparison panels widget.
    pub fn new(config: &AppConfig) -> (Self, GDiffMap) {
        let container = Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(6)
            .build();

        // Create font provider
        let font_provider = CssProvider::new();
        font_provider.load_from_data(&format!(
            "textview {{ font-family: \"{}\"; font-size: {}pt; }}",
            config.font_family, config.font_size
        ));

        // Create panels
        let panel_a = FilePanelWidget::new(
            "File A",
            &config.file_a_history,
            ButtonTheme::LightGreen,
            &font_provider,
        );

        let panel_b = FilePanelWidget::new(
            "File B",
            &config.file_b_history,
            ButtonTheme::LightBlue,
            &font_provider,
        );

        let diff_map = GDiffMap::new();

        // Add to container
        container.append(&panel_a.container);
        container.append(&diff_map);
        container.append(&panel_b.container);

        let widget = Self {
            container,
            panel_a,
            panel_b,
            diff_map: diff_map.clone(),
        };

        (widget, diff_map)
    }

    /// Get the container widget.
    pub fn container(&self) -> &Box {
        &self.container
    }

    /// Get panel A text view.
    pub fn panel_a_text_view(&self) -> &GTextView {
        &self.panel_a.text_view
    }

    /// Get panel A path combo box.
    pub fn panel_a_path_combo(&self) -> &ComboBoxText {
        &self.panel_a.path_combo
    }

    /// Get panel B text view.
    pub fn panel_b_text_view(&self) -> &GTextView {
        &self.panel_b.text_view
    }

    /// Get panel B path combo box.
    pub fn panel_b_path_combo(&self) -> &ComboBoxText {
        &self.panel_b.path_combo
    }

    /// Get panel A open button.
    pub fn panel_a_open_button(&self) -> &GButton {
        &self.panel_a.btn_open
    }

    /// Get panel B open button.
    pub fn panel_b_open_button(&self) -> &GButton {
        &self.panel_b.btn_open
    }

    /// Get panel A save button.
    pub fn panel_a_save_button(&self) -> &GButton {
        &self.panel_a.btn_save
    }

    /// Get panel B save button.
    pub fn panel_b_save_button(&self) -> &GButton {
        &self.panel_b.btn_save
    }

    /// Get diff map reference.
    pub fn diff_map(&self) -> &GDiffMap {
        &self.diff_map
    }
}

/// Individual file panel widget.
struct FilePanelWidget {
    container: Frame,
    text_view: GTextView,
    path_combo: ComboBoxText,
    btn_open: GButton,
    btn_save: GButton,
}

impl FilePanelWidget {
    /// Create a new file panel widget.
    fn new(
        title: &str,
        history: &[String],
        button_theme: ButtonTheme,
        font_provider: &CssProvider,
    ) -> Self {
        // Create panel grid
        let panel_grid = gtk::Grid::builder()
            .row_spacing(6)
            .column_spacing(6)
            .margin_top(6)
            .margin_bottom(6)
            .margin_start(6)
            .margin_end(6)
            .build();

        // Create components
        let text_view = GTextView::new();
        let (path_control_bar, btn_open, btn_save, path_combo) =
            Self::build_path_control_bar(history, button_theme);

        // Assemble panel
        panel_grid.attach(&path_control_bar, 0, 0, 1, 1);
        panel_grid.attach(&text_view, 0, 1, 1, 1);

        // Apply font styling
        text_view
            .content_view()
            .style_context()
            .add_provider(font_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

        text_view
            .gutter_view()
            .style_context()
            .add_provider(font_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

        // Wrap in frame
        let container = Frame::builder()
            .label(format!(" {} ", title))
            .vexpand(true)
            .hexpand(true)
            .child(&panel_grid)
            .build();

        Self {
            container,
            text_view,
            path_combo,
            btn_open,
            btn_save,
        }
    }

    /// Build the path control bar for a file panel.
    fn build_path_control_bar(
        history: &[String],
        button_theme: ButtonTheme,
    ) -> (gtk::Box, GButton, GButton, ComboBoxText) {
        let path_control_bar = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(6)
            .build();

        // Path label
        let path_label = gtk::Label::builder().label("Path:").build();
        path_control_bar.append(&path_label);

        // Path combo box
        let path_combo = ComboBoxText::with_entry();
        path_combo.set_hexpand(true);

        for path in history {
            path_combo.append_text(path);
        }
        path_control_bar.append(&path_combo);

        // Open button
        let btn_open = GButton::new("Open");
        btn_open.set_width_request(60);
        btn_open.set_height_request(30);
        btn_open.set_theme(button_theme);
        path_control_bar.append(&btn_open);

        // Save button
        let btn_save = GButton::new("Save");
        btn_save.set_width_request(60);
        btn_save.set_height_request(30);
        btn_save.set_theme(button_theme);
        path_control_bar.append(&btn_save);

        (path_control_bar, btn_open, btn_save, path_combo)
    }
}
