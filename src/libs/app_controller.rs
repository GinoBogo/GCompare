//! Application controller for managing main application state and logic.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

use gtk::prelude::*;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use gtk::{Adjustment, Application, ApplicationWindow, glib};
use similar::ChangeTag;
use std::cell::Cell;
use std::rc::Rc;

use crate::libs::services::config_service::ConfigService;
use crate::libs::services::diff_service::DiffService;
use crate::libs::services::file_service::FileService;
use crate::libs::state::ApplicationState;
use crate::libs::ui::comparison_panels::ComparisonPanelsWidget;
use crate::libs::ui::control_panel::ControlPanelWidget;
use crate::libs::widgets::gdiffmap::GDiffMap;
use crate::libs::widgets::gstatusbar::GStatusBar;

/// Main application controller that coordinates all components.
pub struct AppController {
    state: Rc<ApplicationState>,
    config_service: ConfigService,
    file_service: FileService,
    diff_service: DiffService,
    window: Option<ApplicationWindow>,
    control_panel: Option<ControlPanelWidget>,
    comparison_panels: Option<ComparisonPanelsWidget>,
    status_bar: Option<GStatusBar>,
    diff_map: Option<GDiffMap>,
}

impl AppController {
    /// Create a new application controller.
    pub fn new() -> Self {
        let config_service = ConfigService::new();
        let state = Rc::new(ApplicationState::new(config_service.load_config()));

        Self {
            state,
            config_service,
            file_service: FileService::new(),
            diff_service: DiffService::new(),
            window: None,
            control_panel: None,
            comparison_panels: None,
            status_bar: None,
            diff_map: None,
        }
    }

    /// Initialize the application UI.
    pub fn initialize_ui(&mut self, app: &Application) {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("GCompare - File Comparison Tool")
            .default_width(self.state.config().window_width)
            .default_height(self.state.config().window_height)
            .width_request(800)
            .height_request(600)
            .maximized(self.state.config().window_maximized)
            .build();

        // Create main layout
        let main_grid = gtk::Grid::builder()
            .margin_top(6)
            .margin_bottom(6)
            .margin_start(6)
            .margin_end(6)
            .row_spacing(6)
            .build();

        window.set_child(Some(&main_grid));

        // Create UI components
        let mut control_panel = ControlPanelWidget::new();
        let (comparison_panels, diff_map) = ComparisonPanelsWidget::new(self.state.config());
        let status_bar = GStatusBar::new();

        // Add components to main grid
        main_grid.attach(control_panel.container(), 0, 0, 1, 1);
        main_grid.attach(comparison_panels.container(), 0, 1, 1, 1);
        main_grid.attach(status_bar.container(), 0, 2, 1, 1);

        // Setup signal handlers
        self.setup_signal_handlers(
            &window,
            &mut control_panel,
            &comparison_panels,
            status_bar.clone(),
        );

        // Store references
        self.window = Some(window);
        self.control_panel = Some(control_panel);
        self.comparison_panels = Some(comparison_panels);
        self.status_bar = Some(status_bar);
        self.diff_map = Some(diff_map);
    }

    /// Show the application window.
    pub fn show(&self) {
        if let Some(window) = &self.window {
            window.present();
        }
    }

    /// Setup all signal handlers for the application.
    fn setup_signal_handlers(
        &self,
        window: &ApplicationWindow,
        control_panel: &mut ControlPanelWidget,
        comparison_panels: &ComparisonPanelsWidget,
        status_bar: GStatusBar,
    ) {
        // Shared state for scroll synchronization
        let sync_enabled = Rc::new(Cell::new(self.state.config().sync_scroll));
        let is_syncing = Rc::new(Cell::new(false));

        // Setup reload button handler
        let panel_a_text_view = comparison_panels.panel_a_text_view().clone();
        let panel_a_path_combo = comparison_panels.panel_a_path_combo().clone();
        let panel_b_text_view = comparison_panels.panel_b_text_view().clone();
        let panel_b_path_combo = comparison_panels.panel_b_path_combo().clone();
        let file_service = self.file_service.clone();

        let panel_a_text_view_reload = panel_a_text_view.clone();
        let panel_b_text_view_reload = panel_b_text_view.clone();
        let status_bar_clone = status_bar.clone();
        let diff_map = comparison_panels.diff_map().clone();

        control_panel.reload_button.connect_clicked(move |_| {
            let (bytes_a, lines_a) =
                file_service.reload_file_from_path(&panel_a_text_view_reload, &panel_a_path_combo);
            let (bytes_b, lines_b) =
                file_service.reload_file_from_path(&panel_b_text_view_reload, &panel_b_path_combo);

            // Update status bar with file information
            status_bar_clone.set_status_a_file_info(bytes_a, lines_a);
            status_bar_clone.set_status_b_file_info(bytes_b, lines_b);

            // Clear diff map
            diff_map.clear_diff_lines();
        });

        // Setup Open Buttons
        let setup_open_button = |button: &crate::libs::widgets::gbutton::GButton,
                                 text_view: &crate::libs::widgets::gtextview::GTextView,
                                 combo: &gtk::ComboBoxText| {
            let window = window.clone();
            let text_view = text_view.content_view();
            let combo = combo.clone();
            let file_service = self.file_service.clone();
            button.connect_clicked(move |_| {
                file_service.open_file_dialog(&window, &text_view, &combo);
            });
        };

        setup_open_button(
            comparison_panels.panel_a_open_button(),
            comparison_panels.panel_a_text_view(),
            comparison_panels.panel_a_path_combo(),
        );

        setup_open_button(
            comparison_panels.panel_b_open_button(),
            comparison_panels.panel_b_text_view(),
            comparison_panels.panel_b_path_combo(),
        );

        // Setup Save Buttons
        let setup_save_button = |button: &crate::libs::widgets::gbutton::GButton,
                                 text_view: &crate::libs::widgets::gtextview::GTextView,
                                 combo: &gtk::ComboBoxText| {
            let window = window.clone();
            let text_view = text_view.content_view();
            let combo = combo.clone();
            let file_service = self.file_service.clone();
            button.connect_clicked(move |_| {
                file_service.save_file_dialog(&window, &text_view, &combo);
            });
        };

        setup_save_button(
            comparison_panels.panel_a_save_button(),
            comparison_panels.panel_a_text_view(),
            comparison_panels.panel_a_path_combo(),
        );

        setup_save_button(
            comparison_panels.panel_b_save_button(),
            comparison_panels.panel_b_text_view(),
            comparison_panels.panel_b_path_combo(),
        );

        // Setup window close handler
        let state = self.state.clone();
        let config_service = self.config_service.clone();
        let panel_a_combo = comparison_panels.panel_a_path_combo().clone();
        let panel_b_combo = comparison_panels.panel_b_path_combo().clone();

        window.connect_close_request(move |window| {
            // Update configuration with current state
            let updated_config =
                state.update_config_from_ui(window, &panel_a_combo, &panel_b_combo);
            config_service.save_config(&updated_config);
            glib::Propagation::Proceed
        });

        // Connect text view changes to GDiffMap updates
        let diff_map = comparison_panels.diff_map().clone();

        // Panel A text changes
        let panel_a_buffer = panel_a_text_view.content_view().buffer();
        let panel_a_buffer_clone = panel_a_buffer.clone();
        let panel_a_buffer_for_changed = panel_a_buffer.clone();
        let panel_a_text_view_for_changed = panel_a_text_view.clone();
        let diff_map_a = diff_map.clone();
        panel_a_buffer.connect_changed(move |_| {
            let line_count = panel_a_buffer_for_changed.line_count() as usize;
            let imp = diff_map_a.imp();
            let info = imp.text_info.get();
            let current_visible_lines = info.a.visible_lines;

            let visible_lines = if current_visible_lines == 0 {
                if let Some(adj) = panel_a_text_view_for_changed.content_view().vadjustment() {
                    let y = adj.value().max(0.0) as i32;
                    let visible_height = adj.page_size();
                    let (start_iter, _) = panel_a_text_view_for_changed.content_view().line_at_y(y);
                    let (end_iter, _) = panel_a_text_view_for_changed
                        .content_view()
                        .line_at_y(y + visible_height as i32);
                    let calculated = (end_iter.line() - start_iter.line()) as usize;
                    if calculated > 50 || calculated == 0 {
                        35
                    } else {
                        calculated
                    }
                } else {
                    35
                }
            } else {
                current_visible_lines
            };

            diff_map_a.update_text_info(
                crate::libs::widgets::gdiffmap::PanelId::A,
                info.a.upper_line,
                line_count,
                visible_lines,
            );
        });

        // Panel B text changes
        let panel_b_buffer = panel_b_text_view.content_view().buffer();
        let panel_b_buffer_clone = panel_b_buffer.clone();
        let panel_b_buffer_for_changed = panel_b_buffer.clone();
        let panel_b_text_view_for_changed = panel_b_text_view.clone();
        let diff_map_b = diff_map.clone();
        panel_b_buffer.connect_changed(move |_| {
            let line_count = panel_b_buffer_for_changed.line_count() as usize;
            let imp = diff_map_b.imp();
            let info = imp.text_info.get();
            let current_visible_lines = info.b.visible_lines;

            let visible_lines = if current_visible_lines == 0 {
                if let Some(adj) = panel_b_text_view_for_changed.content_view().vadjustment() {
                    let y = adj.value().max(0.0) as i32;
                    let visible_height = adj.page_size();
                    let (start_iter, _) = panel_b_text_view_for_changed.content_view().line_at_y(y);
                    let (end_iter, _) = panel_b_text_view_for_changed
                        .content_view()
                        .line_at_y(y + visible_height as i32);
                    let calculated = (end_iter.line() - start_iter.line()) as usize;
                    if calculated > 50 || calculated == 0 {
                        35
                    } else {
                        calculated
                    }
                } else {
                    35
                }
            } else {
                current_visible_lines
            };

            diff_map_b.update_text_info(
                crate::libs::widgets::gdiffmap::PanelId::B,
                info.b.upper_line,
                line_count,
                visible_lines,
            );
        });

        // Panel A scroll changes
        if let (Some(adj_a), Some(adj_b)) = (
            panel_a_text_view.content_view().vadjustment(),
            panel_b_text_view.content_view().vadjustment(),
        ) {
            let diff_map_a = diff_map.clone();
            let panel_a_buffer_clone = panel_a_buffer_clone.clone();
            let panel_a_text_view_clone = panel_a_text_view.clone();
            let sync_enabled = sync_enabled.clone();
            let is_syncing = is_syncing.clone();

            adj_a.connect_value_changed(move |adj: &Adjustment| {
                // Handle Sync Scroll
                if sync_enabled.get() && !is_syncing.get() {
                    is_syncing.set(true);
                    adj_b.set_value(adj.value());
                    is_syncing.set(false);
                }

                // Update DiffMap
                let y = adj.value().max(0.0) as i32;
                let (start_iter, _) = panel_a_text_view_clone.content_view().line_at_y(y);
                let upper_line = start_iter.line() as usize;
                let line_count = panel_a_buffer_clone.line_count() as usize;

                let visible_height = adj.page_size();
                let (end_iter, _) = panel_a_text_view_clone
                    .content_view()
                    .line_at_y(y + visible_height as i32);
                let visible_lines = (end_iter.line() - start_iter.line()) as usize;

                diff_map_a.update_text_info(
                    crate::libs::widgets::gdiffmap::PanelId::A,
                    upper_line,
                    line_count,
                    visible_lines,
                );
            });
        }

        // Panel B scroll changes
        if let (Some(adj_a), Some(adj_b)) = (
            panel_a_text_view.content_view().vadjustment(),
            panel_b_text_view.content_view().vadjustment(),
        ) {
            let diff_map_b = diff_map.clone();
            let panel_b_buffer_clone = panel_b_buffer_clone.clone();
            let panel_b_text_view_clone = panel_b_text_view.clone();
            let sync_enabled = sync_enabled.clone();
            let is_syncing = is_syncing.clone();

            adj_b.connect_value_changed(move |adj: &Adjustment| {
                // Handle Sync Scroll
                if sync_enabled.get() && !is_syncing.get() {
                    is_syncing.set(true);
                    adj_a.set_value(adj.value());
                    is_syncing.set(false);
                }

                // Update DiffMap
                let y = adj.value().max(0.0) as i32;
                let (start_iter, _) = panel_b_text_view_clone.content_view().line_at_y(y);
                let upper_line = start_iter.line() as usize;
                let line_count = panel_b_buffer_clone.line_count() as usize;

                let visible_height = adj.page_size();
                let (end_iter, _) = panel_b_text_view_clone
                    .content_view()
                    .line_at_y(y + visible_height as i32);
                let visible_lines = (end_iter.line() - start_iter.line()) as usize;

                diff_map_b.update_text_info(
                    crate::libs::widgets::gdiffmap::PanelId::B,
                    upper_line,
                    line_count,
                    visible_lines,
                );
            });
        }

        // Connect GDiffMap scroll-to signal (Drag support)
        if let (Some(adj_a), Some(adj_b)) = (
            panel_a_text_view.content_view().vadjustment(),
            panel_b_text_view.content_view().vadjustment(),
        ) {
            diff_map.connect_local("scroll-to", false, move |values| {
                let ratio = values[1].get::<f64>().unwrap();

                // Helper to set adjustment based on ratio
                let set_adj_ratio = |adj: &Adjustment, r: f64| {
                    let upper = adj.upper();
                    let page_size = adj.page_size();
                    let max_value = (upper - page_size).max(0.0);
                    adj.set_value(max_value * r);
                };

                set_adj_ratio(&adj_a, ratio);
                set_adj_ratio(&adj_b, ratio);

                None
            });
        }

        // Setup Compare Button Handler
        let panel_a_text_view = comparison_panels.panel_a_text_view().clone();
        let panel_b_text_view = comparison_panels.panel_b_text_view().clone();
        let diff_service = self.diff_service.clone();
        let diff_map = comparison_panels.diff_map().clone();

        // Create additional clones for navigation buttons
        let diff_map_nav_prev = diff_map.clone();
        let diff_map_nav_next = diff_map.clone();

        control_panel.compare_button.connect_clicked(move |_| {
            let buffer_a = panel_a_text_view.content_view().buffer();
            let buffer_b = panel_b_text_view.content_view().buffer();

            // Helper to get color from CSS class
            let style_context = panel_a_text_view.content_view().style_context();
            let get_theme_color = |class_name: &str| {
                style_context.add_class(class_name);
                let color = style_context.color();
                style_context.remove_class(class_name);
                color
            };

            // Create tags for highlighting if they don't exist
            let create_tag = |buffer: &gtk::TextBuffer, name: &str, css_class: &str| {
                if buffer.tag_table().lookup(name).is_none() {
                    let tag = gtk::TextTag::new(Some(name));
                    let rgba = get_theme_color(css_class);
                    tag.set_background_rgba(Some(&rgba));
                    buffer.tag_table().add(&tag);
                }
            };

            create_tag(&buffer_a, "diff_remove", "diff-text-remove");
            create_tag(&buffer_b, "diff_add", "diff-text-add");

            // Get content
            let (start_a, end_a) = buffer_a.bounds();
            let text_a = buffer_a.text(&start_a, &end_a, true);

            let (start_b, end_b) = buffer_b.bounds();
            let text_b = buffer_b.text(&start_b, &end_b, true);

            // Compute Diff
            let changes = diff_service.compute_diff(text_a.as_str(), text_b.as_str());

            // Clear buffers to redraw with highlights
            buffer_a.set_text("");
            buffer_b.set_text("");

            let mut lines_a = Vec::new();
            let mut lines_b = Vec::new();
            let mut current_line_a = 0;
            let mut current_line_b = 0;

            // Apply changes
            for change in changes {
                match change.tag {
                    ChangeTag::Equal => {
                        buffer_a.insert(&mut buffer_a.end_iter(), &change.content);
                        buffer_b.insert(&mut buffer_b.end_iter(), &change.content);
                        current_line_a += 1;
                        current_line_b += 1;
                    }
                    ChangeTag::Delete => {
                        lines_a.push(current_line_a);
                        buffer_a.insert_with_tags_by_name(
                            &mut buffer_a.end_iter(),
                            &change.content,
                            &["diff_remove"],
                        );
                        current_line_a += 1;
                        // For simple alignment, we might want to insert newlines in B,
                        // but for now we just highlight the deletion in A.
                    }
                    ChangeTag::Insert => {
                        lines_b.push(current_line_b);
                        buffer_b.insert_with_tags_by_name(
                            &mut buffer_b.end_iter(),
                            &change.content,
                            &["diff_add"],
                        );
                        current_line_b += 1;
                    }
                }
            }

            diff_map.set_diff_lines(lines_a, lines_b);
        });

        // Setup navigation buttons
        let panel_a_text_view_nav = comparison_panels.panel_a_text_view().clone();
        let diff_map_nav = diff_map_nav_prev;

        control_panel.previous_button.connect_clicked(move |_| {
            // Get current line from panel A
            if let Some(adj) = panel_a_text_view_nav.content_view().vadjustment() {
                let y = adj.value().max(0.0) as i32;
                let (start_iter, _) = panel_a_text_view_nav.content_view().line_at_y(y);
                let current_line = start_iter.line() as usize;

                if let Some(target_line) = diff_map_nav.previous_difference(current_line) {
                    // Calculate scroll position for target line
                    let buffer = panel_a_text_view_nav.content_view().buffer();
                    if let Some(line_iter) = buffer.iter_at_line(target_line as i32) {
                        let line_y = panel_a_text_view_nav
                            .content_view()
                            .line_yrange(&line_iter)
                            .0 as f64;
                        adj.set_value(line_y);
                    }
                }
            }
        });

        let panel_a_text_view_nav_next = comparison_panels.panel_a_text_view().clone();
        let diff_map_nav_next = diff_map_nav_next;

        control_panel.next_button.connect_clicked(move |_| {
            // Get current line from panel A
            if let Some(adj) = panel_a_text_view_nav_next.content_view().vadjustment() {
                let y = adj.value().max(0.0) as i32;
                let (start_iter, _) = panel_a_text_view_nav_next.content_view().line_at_y(y);
                let current_line = start_iter.line() as usize;

                if let Some(target_line) = diff_map_nav_next.next_difference(current_line) {
                    // Calculate scroll position for target line
                    let buffer = panel_a_text_view_nav_next.content_view().buffer();
                    if let Some(line_iter) = buffer.iter_at_line(target_line as i32) {
                        let line_y = panel_a_text_view_nav_next
                            .content_view()
                            .line_yrange(&line_iter)
                            .0 as f64;
                        adj.set_value(line_y);
                    }
                }
            }
        });

        // control_panel._options_button.connect_clicked(...);

        // TODO: Connect Sync Scroll Toggle Button
        // control_panel.sync_scroll_check_button.connect_toggled(move |btn| {
        //     sync_enabled.set(btn.is_active());
        // });
    }
}
