//! Custom comparison panels widget implementation.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

use gtk::prelude::*;
use gtk::{Box, ComboBoxText, Frame};

use crate::libs::state::AppConfig;
use crate::libs::widgets::gbutton::GButton;
use crate::libs::widgets::gminimap::GMiniMap;
use crate::libs::widgets::gtextview::GTextView;

/// Widget containing file comparison panels and minimap.
#[derive(Clone)]
pub struct ComparisonPanelsWidget {
    container: Box,
    pub panel_a: FilePanelWidget,
    pub panel_b: FilePanelWidget,
    minimap: GMiniMap,
}

impl ComparisonPanelsWidget {
    /// Create a new comparison panels widget.
    ///
    /// # Arguments
    ///
    /// * `config` - Application configuration containing settings
    ///
    /// # Returns
    ///
    /// Tuple of (Self, GMiniMap) containing the widget and minimap
    pub fn new(config: &AppConfig) -> (Self, GMiniMap) {
        let container = Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(6)
            .build();

        // Create panels
        let panel_a = FilePanelWidget::new(
            "File A",
            &config.file_a_history,
            &config.text_diff_remove_bg,
            &config.text_diff_remove_fg,
            &config.font_family,
            config.font_size as f64,
        );

        let panel_b = FilePanelWidget::new(
            "File B",
            &config.file_b_history,
            &config.text_diff_add_bg,
            &config.text_diff_add_fg,
            &config.font_family,
            config.font_size as f64,
        );

        let minimap = GMiniMap::new();

        // Add to container
        container.append(&panel_a.container);
        container.append(&minimap);
        container.append(&panel_b.container);

        let widget = Self {
            container,
            panel_a,
            panel_b,
            minimap: minimap.clone(),
        };

        (widget, minimap)
    }

    /// Get the container widget.
    ///
    /// # Returns
    ///
    /// Reference to the GTK Box container
    pub fn container(&self) -> &Box {
        &self.container
    }

    /// Get panel A text view.
    ///
    /// # Returns
    ///
    /// Reference to panel A's text view widget
    pub fn panel_a_text_view(&self) -> &GTextView {
        &self.panel_a.text_view
    }

    /// Get panel A path combo box.
    ///
    /// # Returns
    ///
    /// Reference to panel A's path combo box
    pub fn panel_a_path_combo(&self) -> &ComboBoxText {
        &self.panel_a.path_combo
    }

    /// Get panel B text view.
    ///
    /// # Returns
    ///
    /// Reference to panel B's text view widget
    pub fn panel_b_text_view(&self) -> &GTextView {
        &self.panel_b.text_view
    }

    /// Get panel B path combo box.
    ///
    /// # Returns
    ///
    /// Reference to panel B's path combo box
    pub fn panel_b_path_combo(&self) -> &ComboBoxText {
        &self.panel_b.path_combo
    }

    /// Get panel A open button.
    ///
    /// # Returns
    ///
    /// Reference to panel A's open button
    pub fn panel_a_open_button(&self) -> &GButton {
        &self.panel_a.open_button
    }

    /// Get panel B open button.
    ///
    /// # Returns
    ///
    /// Reference to panel B's open button
    pub fn panel_b_open_button(&self) -> &GButton {
        &self.panel_b.open_button
    }

    /// Get panel A save button.
    ///
    /// # Returns
    ///
    /// Reference to panel A's save button
    pub fn panel_a_save_button(&self) -> &GButton {
        &self.panel_a.save_button
    }

    /// Get panel B save button.
    ///
    /// # Returns
    ///
    /// Reference to panel B's save button
    pub fn panel_b_save_button(&self) -> &GButton {
        &self.panel_b.save_button
    }

    /// Get minimap reference.
    ///
    /// # Returns
    ///
    /// Reference to the minimap widget
    pub fn minimap(&self) -> &GMiniMap {
        &self.minimap
    }

    /// Update all button colors from config.
    ///
    /// # Arguments
    ///
    /// * `config` - Application configuration containing color settings
    ///
    /// # Returns
    ///
    /// Unit type ()
    pub fn update_button_colors(&self, config: &crate::libs::state::AppConfig) {
        // File A uses remove colors
        self.panel_a
            .open_button
            .set_custom_colors(&config.text_diff_remove_bg, &config.text_diff_remove_fg);
        self.panel_a
            .save_button
            .set_custom_colors(&config.text_diff_remove_bg, &config.text_diff_remove_fg);

        // File B uses add colors
        self.panel_b
            .open_button
            .set_custom_colors(&config.text_diff_add_bg, &config.text_diff_add_fg);
        self.panel_b
            .save_button
            .set_custom_colors(&config.text_diff_add_bg, &config.text_diff_add_fg);
    }
}

/// Individual file panel widget.
#[derive(Clone)]
pub struct FilePanelWidget {
    container: Frame,
    text_view: GTextView,
    pub path_combo: ComboBoxText,
    pub open_button: GButton,
    pub save_button: GButton,
}

impl FilePanelWidget {
    /// Create a new file panel widget.
    ///
    /// # Arguments
    ///
    /// * `title` - Panel title string
    /// * `history` - Slice of file path history strings
    /// * `bg_color` - Background color string
    /// * `fg_color` - Foreground color string
    /// * `font_family` - Font family string
    /// * `font_size` - Font size as f64
    ///
    /// # Returns
    ///
    /// New FilePanelWidget instance
    fn new(
        title: &str,
        history: &[String],
        bg_color: &str,
        fg_color: &str,
        font_family: &str,
        font_size: f64,
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
        let (path_control_bar, open_button, save_button, path_combo) =
            Self::build_path_control_bar(history, bg_color, fg_color);

        // Assemble panel
        panel_grid.attach(&path_control_bar, 0, 0, 1, 1);
        panel_grid.attach(&text_view, 0, 1, 1, 1);

        // Apply font styling
        text_view.set_font(font_family, font_size);

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
            open_button,
            save_button,
        }
    }

    /// Build the path control bar for a file panel.
    ///
    /// # Arguments
    ///
    /// * `history` - Slice of file path history strings
    /// * `bg_color` - Background color string
    /// * `fg_color` - Foreground color string
    ///
    /// # Returns
    ///
    /// Tuple of (gtk::Box, GButton, GButton, ComboBoxText) containing
    /// the control bar and its components
    fn build_path_control_bar(
        history: &[String],
        bg_color: &str,
        fg_color: &str,
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
        let open_button = GButton::new("Open");
        open_button.set_tooltip_text(Some("Open a file for comparison"));
        open_button.set_width_request(60);
        open_button.set_height_request(30);
        open_button.set_custom_colors(bg_color, fg_color);
        path_control_bar.append(&open_button);

        // Save button
        let save_button = GButton::new("Save");
        save_button.set_tooltip_text(Some("Save the current file content"));
        save_button.set_width_request(60);
        save_button.set_height_request(30);
        save_button.set_custom_colors(bg_color, fg_color);
        path_control_bar.append(&save_button);

        (path_control_bar, open_button, save_button, path_combo)
    }
}
