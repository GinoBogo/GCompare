//! Application controller for managing main application state and logic.
//!
//! * Author: Gino Bogo
//! * License: MIT
//! * Version: 1.0

use gtk::prelude::*;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use gtk::{Adjustment, Application, ApplicationWindow, MessageDialog, glib};
use std::cell::Cell;
use std::rc::Rc;

use crate::libs::app_handlers;
use crate::libs::services::color_parser::parse_color_with_fallback;
use crate::libs::services::config_service::ConfigService;
use crate::libs::services::diff_service::DiffService;
use crate::libs::services::file_service::FileService;
use crate::libs::services::incremental_diff_service::IncrementalDiffService;
use crate::libs::services::text_highlighter::TextHighlighter;
use crate::libs::state::ApplicationState;
use crate::libs::theme;
use crate::libs::ui::comparison_panels::ComparisonPanelsWidget;
use crate::libs::ui::control_panel::ControlPanelWidget;
use crate::libs::widgets::gminimap::GMiniMap;
use crate::libs::widgets::gstatusbar::GStatusBar;

/// Main application controller that coordinates all components.
pub struct AppController {
    state: Rc<ApplicationState>,
    config_service: ConfigService,
    file_service: FileService,
    diff_service: DiffService,
    incremental_diff_service: IncrementalDiffService,
    text_highlighter: TextHighlighter,
    css_provider: Rc<gtk::CssProvider>,
    window: Option<ApplicationWindow>,
    control_panel: Option<ControlPanelWidget>,
    comparison_panels: Option<ComparisonPanelsWidget>,
    status_bar: Option<GStatusBar>,
    minimap: Option<GMiniMap>,
}

impl AppController {
    /// Create a new application controller.
    pub fn new() -> Self {
        let config_service = ConfigService::new();
        let state = Rc::new(ApplicationState::new(config_service.get_config()));

        // Initialize theme and get CSS provider
        let css_provider = Rc::new(theme::init());

        // Update theme with config colors
        theme::update_provider_with_config(&css_provider, &state.config());

        Self {
            state,
            config_service,
            file_service: FileService::new(),
            diff_service: DiffService::new(),
            incremental_diff_service: IncrementalDiffService::new(),
            text_highlighter: TextHighlighter::new(),
            css_provider,
            window: None,
            control_panel: None,
            comparison_panels: None,
            status_bar: None,
            minimap: None,
        }
    }

    /// Initialize the application UI.
    pub fn initialize_ui(
        &mut self,
        app: &Application,
        file_a_path: Option<String>,
        file_b_path: Option<String>,
    ) {
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
        let (comparison_panels, minimap) = ComparisonPanelsWidget::new(&self.state.config());
        let status_bar = GStatusBar::new();

        // Add components to main grid
        main_grid.attach(control_panel.container(), 0, 0, 1, 1);
        main_grid.attach(comparison_panels.container(), 0, 1, 1, 1);
        main_grid.attach(status_bar.container(), 0, 2, 1, 1);

        // Hide the auto-compare button as it's now in options
        control_panel.auto_compare_button.set_visible(false);

        // Setup signal handlers
        self.setup_signal_handlers(
            &window,
            &mut control_panel,
            &comparison_panels,
            status_bar.clone(),
        );

        // Setup real-time diff tracking
        self.setup_realtime_diff(
            comparison_panels.panel_a_text_view(),
            comparison_panels.panel_b_text_view(),
            &minimap,
            &status_bar,
        );

        // Store references
        self.window = Some(window);
        self.control_panel = Some(control_panel);
        self.comparison_panels = Some(comparison_panels);
        self.status_bar = Some(status_bar);
        self.minimap = Some(minimap);

        // Load files from command-line arguments if provided
        self.load_files_from_args(file_a_path, file_b_path);
    }

    /// Show the application window.
    pub fn show(&self) {
        if let Some(window) = &self.window {
            window.present();
        }
    }

    /// Load files from command-line arguments into the text views.
    ///
    /// # Arguments
    ///
    /// * `file_a_path` - Optional path to file A.
    /// * `file_b_path` - Optional path to file B.
    fn load_files_from_args(&mut self, file_a_path: Option<String>, file_b_path: Option<String>) {
        if let (Some(comparison_panels), Some(status_bar)) =
            (&self.comparison_panels, &self.status_bar)
        {
            let panel_a_text_view = comparison_panels.panel_a_text_view();
            let panel_a_path_combo = comparison_panels.panel_a_path_combo();
            let panel_b_text_view = comparison_panels.panel_b_text_view();
            let panel_b_path_combo = comparison_panels.panel_b_path_combo();

            // Load file A if provided
            if let Some(path_a) = file_a_path
                && std::path::Path::new(&path_a).exists()
            {
                // Set the path in the combo box
                if let Some(entry) = panel_a_path_combo
                    .child()
                    .and_then(|c| c.downcast::<gtk::Entry>().ok())
                {
                    entry.set_text(&path_a);
                }

                // Load the file content
                let (bytes_a, lines_a) = self.file_service.load_file_from_path(
                    panel_a_text_view,
                    panel_a_path_combo,
                    None,
                );
                status_bar.set_status_a_file_info(bytes_a, lines_a);
            }

            // Load file B if provided
            if let Some(path_b) = file_b_path
                && std::path::Path::new(&path_b).exists()
            {
                // Set the path in the combo box
                if let Some(entry) = panel_b_path_combo
                    .child()
                    .and_then(|c| c.downcast::<gtk::Entry>().ok())
                {
                    entry.set_text(&path_b);
                }

                // Load the file content
                let (bytes_b, lines_b) = self.file_service.load_file_from_path(
                    panel_b_text_view,
                    panel_b_path_combo,
                    None,
                );
                status_bar.set_status_b_file_info(bytes_b, lines_b);
            }
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

        // Shared state for loading flag to suppress "modified" label updates
        let is_loading = Rc::new(Cell::new(false));

        // Setup load button handler
        let panel_a_text_view = comparison_panels.panel_a_text_view().clone();
        let panel_a_path_combo = comparison_panels.panel_a_path_combo().clone();
        let panel_b_text_view = comparison_panels.panel_b_text_view().clone();
        let panel_b_path_combo = comparison_panels.panel_b_path_combo().clone();
        let file_service = self.file_service.clone();

        let panel_a_text_view_reload = panel_a_text_view.clone();
        let panel_b_text_view_reload = panel_b_text_view.clone();
        let status_bar_clone = status_bar.clone();
        let minimap = comparison_panels.minimap().clone();

        let panel_a_path_combo_reload = panel_a_path_combo.clone();
        let panel_b_path_combo_reload = panel_b_path_combo.clone();

        let is_loading_reload = is_loading.clone();

        // Helper to find the status label associated with a combo box
        let find_status_label = Rc::new(
            |combo: &gtk::ComboBoxText, base_text: &str| -> Option<gtk::Label> {
                let mut current_parent = combo.parent();
                for _ in 0..3 {
                    if let Some(parent) = current_parent {
                        let mut child = parent.first_child();
                        while let Some(widget) = child {
                            if let Some(label) = widget.downcast_ref::<gtk::Label>() {
                                if label.label().contains(base_text) {
                                    return Some(label.clone());
                                }
                            }
                            child = widget.next_sibling();
                        }
                        current_parent = parent.parent();
                    } else {
                        break;
                    }
                }
                None
            },
        );

        // Helper to check if modified
        let is_modified = Rc::new({
            let find_status_label = find_status_label.clone();
            move |combo: &gtk::ComboBoxText, base_text: &str| -> bool {
                (*find_status_label)(combo, base_text)
                    .map(|l| l.label().contains('*'))
                    .unwrap_or(false)
            }
        });

        // Helper to prompt for save
        let prompt_save = {
            let window = window.clone();
            move |has_unsaved: bool, on_discard: Rc<dyn Fn()>, on_save: Option<Rc<dyn Fn()>>| {
                if !has_unsaved {
                    on_discard();
                    return;
                }

                let dialog = MessageDialog::builder()
                    .transient_for(&window)
                    .modal(true)
                    .message_type(gtk::MessageType::Question)
                    .buttons(gtk::ButtonsType::None)
                    .text("Unsaved Changes")
                    .secondary_text("You have unsaved changes. Do you want to save them?")
                    .build();

                dialog.add_button("Cancel", gtk::ResponseType::Cancel);
                dialog.add_button("Discard Changes", gtk::ResponseType::Close);
                if on_save.is_some() {
                    dialog.add_button("Save", gtk::ResponseType::Accept);
                }

                let on_discard = on_discard.clone();
                let on_save = on_save.clone();

                dialog.connect_response(move |dlg, response| {
                    dlg.close();
                    match response {
                        gtk::ResponseType::Close => on_discard(),
                        gtk::ResponseType::Accept => {
                            if let Some(save_cb) = &on_save {
                                save_cb();
                            }
                        }
                        _ => {}
                    }
                });
                dialog.show();
            }
        };

        // Helper to reset label (remove *)
        let reset_label = Rc::new({
            let find_status_label = find_status_label.clone();
            move |combo: &gtk::ComboBoxText, base_text: &str| {
                if let Some(label) = (*find_status_label)(combo, base_text) {
                    label.set_label(base_text);
                }
            }
        });

        // Helper to set label to modified state (add *)
        let set_modified_label = Rc::new({
            let find_status_label = find_status_label.clone();
            move |combo: &gtk::ComboBoxText, base_text: &str| {
                if let Some(label) = (*find_status_label)(combo, base_text) {
                    let text = label.label();
                    if !text.contains('*') {
                        label.set_label(&format!("{}*", base_text));
                    }
                }
            }
        });

        let panel_a_path_combo_reset = panel_a_path_combo.clone();
        let panel_b_path_combo_reset = panel_b_path_combo.clone();

        let prompt_save_reload = prompt_save.clone();
        let file_service_reload = file_service.clone();
        let window_reload = window.clone();

        let is_modified_reload = is_modified.clone();
        let reset_label_reload = reset_label.clone();

        control_panel.load_button.connect_clicked(move |_| {
            // Check if files are specified
            let get_path = |combo: &gtk::ComboBoxText| {
                combo
                    .child()
                    .and_then(|c| c.downcast::<gtk::Entry>().ok())
                    .map(|e| e.text().to_string())
                    .unwrap_or_default()
            };

            let path_a = get_path(&panel_a_path_combo_reload);
            let path_b = get_path(&panel_b_path_combo_reload);

            if path_a.trim().is_empty() && path_b.trim().is_empty() {
                let dialog = MessageDialog::builder()
                    .transient_for(&window_reload)
                    .modal(true)
                    .message_type(gtk::MessageType::Warning)
                    .buttons(gtk::ButtonsType::Ok)
                    .text("No Files Specified")
                    .secondary_text("Please specify at least one file to load.")
                    .build();
                dialog.connect_response(|dlg, _| dlg.close());
                dialog.show();
                return;
            }

            let mod_a = (*is_modified_reload)(&panel_a_path_combo_reload, "File A");
            let mod_b = (*is_modified_reload)(&panel_b_path_combo_reload, "File B");

            let do_reload = {
                let file_service = file_service.clone();
                let panel_a_text_view_reload = panel_a_text_view_reload.clone();
                let panel_a_path_combo_reload = panel_a_path_combo_reload.clone();
                let is_loading_reload = is_loading_reload.clone();
                let panel_b_text_view_reload = panel_b_text_view_reload.clone();
                let panel_b_path_combo_reload = panel_b_path_combo_reload.clone();
                let panel_a_path_combo_reset = panel_a_path_combo_reset.clone();
                let panel_b_path_combo_reset = panel_b_path_combo_reset.clone();
                let status_bar_clone = status_bar_clone.clone();
                let minimap = minimap.clone();
                let reset_label = reset_label_reload.clone();
                Rc::new(move || {
                    let (bytes_a, lines_a) = file_service.load_file_from_path(
                        &panel_a_text_view_reload,
                        &panel_a_path_combo_reload,
                        Some(is_loading_reload.clone()),
                    );
                    let (bytes_b, lines_b) = file_service.load_file_from_path(
                        &panel_b_text_view_reload,
                        &panel_b_path_combo_reload,
                        Some(is_loading_reload.clone()),
                    );

                    // Reset labels to clean state
                    (*reset_label)(&panel_a_path_combo_reset, "File A");
                    (*reset_label)(&panel_b_path_combo_reset, "File B");

                    // Update status bar with file information
                    status_bar_clone.set_status_a_file_info(bytes_a, lines_a);
                    status_bar_clone.set_status_b_file_info(bytes_b, lines_b);

                    // Clear diff map
                    minimap.clear_diff_lines();
                })
            };

            let do_save = if mod_a || mod_b {
                let file_service = file_service_reload.clone();
                let window = window_reload.clone();
                let tv_a = panel_a_text_view_reload.clone();
                let cb_a = panel_a_path_combo_reload.clone();
                let tv_b = panel_b_text_view_reload.clone();
                let cb_b = panel_b_path_combo_reload.clone();
                let do_reload = do_reload.clone();

                Some(Rc::new(move || {
                    let do_reload = do_reload.clone();
                    let file_service_b = file_service.clone();
                    let window_b = window.clone();
                    let tv_b = tv_b.clone();
                    let cb_b = cb_b.clone();

                    let save_b_chain = move || {
                        if mod_b {
                            let do_reload = do_reload.clone();
                            file_service_b.save_file_dialog(
                                &window_b,
                                &tv_b.content_view(),
                                &cb_b,
                                Some(Box::new(move || do_reload())),
                            );
                        } else {
                            do_reload();
                        }
                    };

                    if mod_a {
                        file_service.save_file_dialog(
                            &window,
                            &tv_a.content_view(),
                            &cb_a,
                            Some(Box::new(save_b_chain)),
                        );
                    } else {
                        save_b_chain();
                    }
                }) as Rc<dyn Fn()>)
            } else {
                None
            };

            prompt_save_reload(mod_a || mod_b, do_reload, do_save);
        });

        // Setup Merge Button Handler
        let panel_a_path_combo_merge = comparison_panels.panel_a_path_combo().clone();
        let panel_b_path_combo_merge = comparison_panels.panel_b_path_combo().clone();
        let panel_a_text_view_merge = comparison_panels.panel_a_text_view().clone();
        let panel_b_text_view_merge = comparison_panels.panel_b_text_view().clone();
        let window_merge = window.clone();
        let config_service_merge = self.config_service.clone();

        control_panel.merge_button.connect_clicked(move |_| {
            // Helper to get path string from combo box entry
            let get_path = |combo: &gtk::ComboBoxText| {
                combo
                    .child()
                    .and_then(|c| c.downcast::<gtk::Entry>().ok())
                    .map(|e| e.text().to_string())
                    .unwrap_or_default()
            };

            let path_a = get_path(&panel_a_path_combo_merge);
            let path_b = get_path(&panel_b_path_combo_merge);

            let file_a_exists = !path_a.is_empty() && std::path::Path::new(&path_a).exists();
            let file_b_exists = !path_b.is_empty() && std::path::Path::new(&path_b).exists();

            if !file_a_exists || !file_b_exists {
                let message = if !file_a_exists && !file_b_exists {
                    "Both File A and File B do not exist or have invalid paths."
                } else if !file_a_exists {
                    "File A does not exist or has an invalid path."
                } else {
                    "File B does not exist or has an invalid path."
                };

                let dialog = MessageDialog::builder()
                    .transient_for(&window_merge)
                    .modal(true)
                    .message_type(gtk::MessageType::Error)
                    .buttons(gtk::ButtonsType::Ok)
                    .text("Cannot Merge")
                    .secondary_text(message)
                    .build();

                dialog.connect_response(|dlg, _| dlg.close());
                dialog.show();
            } else {
                let buffer_a = panel_a_text_view_merge.content_view().buffer();
                let (start_a, end_a) = buffer_a.bounds();
                let text_a = buffer_a.text(&start_a, &end_a, true);

                let buffer_b = panel_b_text_view_merge.content_view().buffer();
                let (start_b, end_b) = buffer_b.bounds();
                let text_b = buffer_b.text(&start_b, &end_b, true);

                let merge_view = crate::libs::ui::merge_view::GMergeView::new(
                    &window_merge,
                    text_a.as_str(),
                    text_b.as_str(),
                    config_service_merge.clone(),
                );
                merge_view.show();
            }
        });

        // Setup Open Buttons
        let setup_open_button = |button: &crate::libs::widgets::gbutton::GButton,
                                 text_view: &crate::libs::widgets::gtextview::GTextView,
                                 combo: &gtk::ComboBoxText,
                                 is_panel_a: bool| {
            let window = window.clone();
            let text_view = text_view.content_view();
            let combo = combo.clone();
            let file_service = self.file_service.clone();
            let is_loading = is_loading.clone();
            let prompt_save = prompt_save.clone();

            // Create callback for successful load to reset label
            let combo_for_callback = combo.clone();
            let on_load: Rc<dyn Fn()> = Rc::new(move || {
                let base_text = if is_panel_a { "File A" } else { "File B" };
                let mut current_parent = combo_for_callback.parent();
                for _ in 0..3 {
                    if let Some(parent) = current_parent {
                        let mut child = parent.first_child();
                        while let Some(widget) = child {
                            if let Some(label) = widget.downcast_ref::<gtk::Label>() {
                                if label.label().contains(base_text) {
                                    label.set_label(base_text);
                                    return;
                                }
                            }
                            child = widget.next_sibling();
                        }
                        current_parent = parent.parent();
                    } else {
                        break;
                    }
                }
            });

            let is_modified = is_modified.clone();

            button.connect_clicked(move |_| {
                let mod_file = (*is_modified)(&combo, if is_panel_a { "File A" } else { "File B" });

                let do_open = {
                    let file_service = file_service.clone();
                    let window = window.clone();
                    let text_view = text_view.clone();
                    let combo = combo.clone();
                    let is_loading = is_loading.clone();
                    let on_load = on_load.clone();
                    Rc::new(move || {
                        let on_load_for_dialog = Box::new({
                            let on_load = on_load.clone();
                            move || on_load()
                        });
                        file_service.open_file_dialog(
                            &window,
                            &text_view,
                            &combo,
                            Some(is_loading.clone()),
                            Some(on_load_for_dialog),
                        );
                    })
                };

                let do_save = if mod_file {
                    let file_service = file_service.clone();
                    let window = window.clone();
                    let text_view = text_view.clone();
                    let combo = combo.clone();
                    let do_open = do_open.clone();

                    Some(Rc::new(move || {
                        let do_open = do_open.clone();
                        file_service.save_file_dialog(
                            &window,
                            &text_view,
                            &combo,
                            Some(Box::new(move || do_open())),
                        );
                    }) as Rc<dyn Fn()>)
                } else {
                    None
                };

                prompt_save(mod_file, do_open, do_save);
            });
        };

        setup_open_button(
            comparison_panels.panel_a_open_button(),
            comparison_panels.panel_a_text_view(),
            comparison_panels.panel_a_path_combo(),
            true,
        );

        setup_open_button(
            comparison_panels.panel_b_open_button(),
            comparison_panels.panel_b_text_view(),
            comparison_panels.panel_b_path_combo(),
            false,
        );

        // Setup Save Buttons
        let setup_save_button = |button: &crate::libs::widgets::gbutton::GButton,
                                 text_view: &crate::libs::widgets::gtextview::GTextView,
                                 combo: &gtk::ComboBoxText,
                                 is_panel_a: bool| {
            let window = window.clone();
            let text_view = text_view.content_view();
            let combo = combo.clone();
            let file_service = self.file_service.clone();

            // Create callback for successful save
            let combo_for_callback = combo.clone();
            let on_save: Option<std::rc::Rc<dyn Fn()>> = Some(std::rc::Rc::new(move || {
                let base_text = if is_panel_a { "File A" } else { "File B" };
                let modified_text = format!("{}*", base_text);
                let mut current_parent = combo_for_callback.parent();
                // Search up to 3 levels of parents to find the label
                for _ in 0..3 {
                    if let Some(parent) = current_parent {
                        let mut child = parent.first_child();
                        while let Some(widget) = child {
                            if let Some(label) = widget.downcast_ref::<gtk::Label>() {
                                if label.label().contains(&modified_text) {
                                    label.set_label(base_text);
                                    return;
                                }
                            }
                            child = widget.next_sibling();
                        }
                        current_parent = parent.parent();
                    } else {
                        break;
                    }
                }
            }));

            button.connect_clicked(move |_| {
                let on_save_box = on_save.as_ref().map(|rc| {
                    let rc_clone = rc.clone();
                    Box::new(move || rc_clone()) as Box<dyn Fn() + 'static>
                });
                file_service.save_file_dialog(&window, &text_view, &combo, on_save_box);
            });
        };

        setup_save_button(
            comparison_panels.panel_a_save_button(),
            comparison_panels.panel_a_text_view(),
            comparison_panels.panel_a_path_combo(),
            true,
        );

        setup_save_button(
            comparison_panels.panel_b_save_button(),
            comparison_panels.panel_b_text_view(),
            comparison_panels.panel_b_path_combo(),
            false,
        );

        // Setup window close handler
        let state_for_close = self.state.clone();
        let config_service = self.config_service.clone();
        let panel_a_combo = comparison_panels.panel_a_path_combo().clone();
        let panel_b_combo = comparison_panels.panel_b_path_combo().clone();
        let file_service_close = self.file_service.clone();
        let panel_a_text_view_close = comparison_panels.panel_a_text_view().clone();
        let panel_b_text_view_close = comparison_panels.panel_b_text_view().clone();

        let is_modified_close = is_modified.clone();

        window.connect_close_request(move |window| {
            let mod_a = (*is_modified_close)(&panel_a_combo, "File A");
            let mod_b = (*is_modified_close)(&panel_b_combo, "File B");

            if mod_a || mod_b {
                let do_close = {
                    let window_for_destroy = window.clone();
                    let state_for_close = state_for_close.clone();
                    let config_service = config_service.clone();
                    let panel_a_combo_for_close = panel_a_combo.clone();
                    let panel_b_combo_for_close = panel_b_combo.clone();
                    Rc::new(move || {
                        let mut updated_config = state_for_close.update_config_from_ui(
                            &window_for_destroy,
                            &panel_a_combo_for_close,
                            &panel_b_combo_for_close,
                        );

                        // Get current config from service to preserve merge
                        // window geometry updates
                        let current_service_config = config_service.get_config();
                        updated_config.merge_window_width =
                            current_service_config.merge_window_width;
                        updated_config.merge_window_height =
                            current_service_config.merge_window_height;
                        updated_config.merge_window_maximized =
                            current_service_config.merge_window_maximized;

                        config_service.update_config(updated_config);
                        config_service.save_config();
                        window_for_destroy.destroy();
                    })
                };

                let do_save = {
                    let file_service = file_service_close.clone();
                    let window = window.clone();
                    let tv_a = panel_a_text_view_close.clone();
                    let cb_a = panel_a_combo.clone();
                    let tv_b = panel_b_text_view_close.clone();
                    let cb_b = panel_b_combo.clone();
                    let do_close = do_close.clone();

                    Rc::new(move || {
                        let do_close = do_close.clone();
                        let file_service_b = file_service.clone();
                        let window_b = window.clone();
                        let tv_b = tv_b.clone();
                        let cb_b = cb_b.clone();

                        let save_b_chain = move || {
                            if mod_b {
                                let do_close = do_close.clone();
                                file_service_b.save_file_dialog(
                                    &window_b,
                                    &tv_b.content_view(),
                                    &cb_b,
                                    Some(Box::new(move || do_close())),
                                );
                            } else {
                                do_close();
                            }
                        };

                        if mod_a {
                            file_service.save_file_dialog(
                                &window,
                                &tv_a.content_view(),
                                &cb_a,
                                Some(Box::new(save_b_chain)),
                            );
                        } else {
                            save_b_chain();
                        }
                    })
                };

                prompt_save(true, do_close, Some(do_save));
                return glib::Propagation::Stop;
            }

            // Update configuration with current state and preserve merge window
            // geometry from ConfigService
            let mut updated_config =
                state_for_close.update_config_from_ui(window, &panel_a_combo, &panel_b_combo);

            // Get current config from service to preserve merge window geometry
            // updates
            let current_service_config = config_service.get_config();
            updated_config.merge_window_width = current_service_config.merge_window_width;
            updated_config.merge_window_height = current_service_config.merge_window_height;
            updated_config.merge_window_maximized = current_service_config.merge_window_maximized;

            config_service.update_config(updated_config);
            config_service.save_config();

            // Quit the application
            window.destroy();
            glib::Propagation::Proceed
        });

        let minimap = comparison_panels.minimap().clone();

        // Helper for buffer change handling
        let setup_buffer_handler = {
            let set_modified_label = set_modified_label.clone();
            move |buffer: &gtk::TextBuffer,
                  other_buffer: &gtk::TextBuffer,
                  text_view: &crate::libs::widgets::gtextview::GTextView,
                  minimap: &crate::libs::widgets::gminimap::GMiniMap,
                  status_bar: &crate::libs::widgets::gstatusbar::GStatusBar,
                  combo: &gtk::ComboBoxText,
                  is_loading: &Rc<Cell<bool>>,
                  panel_id: crate::libs::widgets::gminimap::PanelId,
                  label_text: &'static str| {
                let buffer_for_changed = buffer.clone();
                let other_buffer_for_status = other_buffer.clone();
                let text_view_for_changed = text_view.clone();
                let minimap = minimap.clone();
                let status_bar = status_bar.clone();
                let combo_for_changed = combo.clone();
                let is_loading = is_loading.clone();
                let set_modified_label = set_modified_label.clone();

                buffer.connect_changed(move |_| {
                    let line_count = buffer_for_changed.line_count() as usize;
                    let imp = minimap.imp();
                    let info = imp.text_info.get();

                    let current_visible_lines = match panel_id {
                        crate::libs::widgets::gminimap::PanelId::A => info.a.visible_lines,
                        crate::libs::widgets::gminimap::PanelId::B => info.b.visible_lines,
                    };

                    let visible_lines = if current_visible_lines == 0 {
                        if let Some(adj) = text_view_for_changed.content_view().vadjustment() {
                            let y = adj.value().max(0.0) as i32;
                            let visible_height = adj.page_size();
                            let (start_iter, _) = text_view_for_changed.content_view().line_at_y(y);
                            let (end_iter, _) = text_view_for_changed
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

                    let upper_line = match panel_id {
                        crate::libs::widgets::gminimap::PanelId::A => info.a.upper_line,
                        crate::libs::widgets::gminimap::PanelId::B => info.b.upper_line,
                    };

                    minimap.update_text_info(panel_id, upper_line, line_count, visible_lines);

                    // Update status bar with current info
                    let (buf_a, buf_b) = match panel_id {
                        crate::libs::widgets::gminimap::PanelId::A => {
                            (&buffer_for_changed, &other_buffer_for_status)
                        }
                        crate::libs::widgets::gminimap::PanelId::B => {
                            (&other_buffer_for_status, &buffer_for_changed)
                        }
                    };
                    status_bar.update_status_from_buffers(buf_a, buf_b, &minimap);

                    // Update label
                    if !is_loading.get() {
                        (*set_modified_label)(&combo_for_changed, label_text);
                    }
                })
            }
        };

        // Setup Panel A text changes
        let panel_a_buffer = panel_a_text_view.content_view().buffer();
        let panel_b_buffer = panel_b_text_view.content_view().buffer();

        setup_buffer_handler(
            &panel_a_buffer,
            &panel_b_buffer,
            &panel_a_text_view,
            &minimap,
            &status_bar,
            &panel_a_path_combo,
            &is_loading,
            crate::libs::widgets::gminimap::PanelId::A,
            "File A",
        );

        // Setup Panel B text changes
        setup_buffer_handler(
            &panel_b_buffer,
            &panel_a_buffer,
            &panel_b_text_view,
            &minimap,
            &status_bar,
            &panel_b_path_combo,
            &is_loading,
            crate::libs::widgets::gminimap::PanelId::B,
            "File B",
        );

        // Helper for scroll handling
        let setup_scroll_handler =
            |adj: &Adjustment,
             other_adj: &Adjustment,
             text_view: &crate::libs::widgets::gtextview::GTextView,
             buffer: &gtk::TextBuffer,
             minimap: &crate::libs::widgets::gminimap::GMiniMap,
             sync_enabled: &Rc<Cell<bool>>,
             is_syncing: &Rc<Cell<bool>>,
             panel_id: crate::libs::widgets::gminimap::PanelId| {
                let minimap = minimap.clone();
                let buffer = buffer.clone();
                let text_view = text_view.clone();
                let sync_enabled = sync_enabled.clone();
                let is_syncing = is_syncing.clone();
                let other_adj = other_adj.clone();

                adj.connect_value_changed(move |adj: &Adjustment| {
                    // Handle Sync Scroll
                    if sync_enabled.get() && !is_syncing.get() {
                        is_syncing.set(true);
                        other_adj.set_value(adj.value());
                        is_syncing.set(false);
                    }

                    // Update DiffMap
                    let y = adj.value().max(0.0) as i32;
                    let (start_iter, _) = text_view.content_view().line_at_y(y);
                    let upper_line = start_iter.line() as usize;
                    let line_count = buffer.line_count() as usize;

                    let visible_height = adj.page_size();
                    let (end_iter, _) = text_view
                        .content_view()
                        .line_at_y(y + visible_height as i32);
                    let visible_lines = (end_iter.line() - start_iter.line()) as usize;

                    minimap.update_text_info(panel_id, upper_line, line_count, visible_lines);
                })
            };

        // Setup Scroll Handlers
        if let (Some(adj_a), Some(adj_b)) = (
            panel_a_text_view.content_view().vadjustment(),
            panel_b_text_view.content_view().vadjustment(),
        ) {
            setup_scroll_handler(
                &adj_a,
                &adj_b,
                &panel_a_text_view,
                &panel_a_buffer,
                &minimap,
                &sync_enabled,
                &is_syncing,
                crate::libs::widgets::gminimap::PanelId::A,
            );

            setup_scroll_handler(
                &adj_b,
                &adj_a,
                &panel_b_text_view,
                &panel_b_buffer,
                &minimap,
                &sync_enabled,
                &is_syncing,
                crate::libs::widgets::gminimap::PanelId::B,
            );
        }

        // Connect GMiniMap scroll-to signal (Drag support)
        if let (Some(adj_a), Some(adj_b)) = (
            panel_a_text_view.content_view().vadjustment(),
            panel_b_text_view.content_view().vadjustment(),
        ) {
            minimap.connect_local("scroll-to", false, move |values| {
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
        let panel_a_combo = comparison_panels.panel_a_path_combo().clone();
        let panel_b_combo = comparison_panels.panel_b_path_combo().clone();
        let minimap = comparison_panels.minimap().clone();
        let status_bar_compare = status_bar.clone();

        app_handlers::setup_compare_interaction(
            window,
            &control_panel.compare_button,
            &panel_a_text_view,
            &panel_b_text_view,
            &panel_a_combo,
            &panel_b_combo,
            &minimap,
            &status_bar_compare,
            self.state.clone(),
            self.diff_service.clone(),
            self.incremental_diff_service.clone(),
            self.text_highlighter.clone(),
            is_loading.clone(),
        );

        // Setup navigation buttons
        app_handlers::setup_navigation_interaction(
            window,
            &control_panel.previous_button,
            &control_panel.next_button,
            &panel_a_text_view,
            &minimap,
        );

        // Options button click handler
        app_handlers::setup_options_interaction(
            &control_panel.options_button,
            window,
            self.state.clone(),
            self.config_service.clone(),
            &panel_a_text_view,
            &panel_b_text_view,
            self.css_provider.clone(),
            &minimap,
            sync_enabled.clone(),
        );
    }

    /// Setup real-time diff tracking for both text buffers.
    fn setup_realtime_diff(
        &self,
        panel_a_text_view: &crate::libs::widgets::gtextview::GTextView,
        panel_b_text_view: &crate::libs::widgets::gtextview::GTextView,
        minimap: &crate::libs::widgets::gminimap::GMiniMap,
        status_bar: &crate::libs::widgets::gstatusbar::GStatusBar,
    ) {
        use std::rc::Rc;

        let buffer_a = panel_a_text_view.content_view().buffer();
        let buffer_b = panel_b_text_view.content_view().buffer();

        // Ensure tags exist to prevent Gtk-WARNINGs during auto-compare
        let config = self.state.config();
        let create_tags = |buffer: &gtk::TextBuffer| {
            let table = buffer.tag_table();
            let tags = [
                (
                    "diff_remove",
                    &config.text_diff_remove_bg,
                    &config.text_diff_remove_fg,
                ),
                (
                    "diff_add",
                    &config.text_diff_add_bg,
                    &config.text_diff_add_fg,
                ),
                (
                    "diff_empty",
                    &config.text_diff_empty_bg,
                    &config.text_diff_empty_fg,
                ),
            ];
            for (name, bg, fg) in tags {
                if table.lookup(name).is_none() {
                    let tag = gtk::TextTag::new(Some(name));
                    let bg_rgba = parse_color_with_fallback(bg, 255, 255, 255, 1.0);
                    let fg_rgba = parse_color_with_fallback(fg, 0, 0, 0, 1.0);
                    tag.set_background_rgba(Some(&bg_rgba));
                    tag.set_foreground_rgba(Some(&fg_rgba));
                    table.add(&tag);
                }
            }
        };
        create_tags(&buffer_a);
        create_tags(&buffer_b);

        // Shared state for debouncing
        let debounce_token = Rc::new(Cell::new(0u64));
        let is_computing = Rc::new(Cell::new(false));

        // Clone needed values for the closure
        let incremental_diff_service = self.incremental_diff_service.clone();
        let text_highlighter = self.text_highlighter.clone();
        let minimap_clone = minimap.clone();
        let status_bar_clone = status_bar.clone();
        let state = self.state.clone();

        // Function to perform the diff update
        let perform_diff_update = {
            let is_computing = is_computing.clone();
            let incremental_diff_service = incremental_diff_service.clone();
            let text_highlighter = text_highlighter.clone();
            let minimap_clone = minimap_clone.clone();
            let status_bar_clone = status_bar_clone.clone();
            let state = state.clone();

            move |buffer_a: gtk::TextBuffer, buffer_b: gtk::TextBuffer| {
                // Skip if auto-comparison is disabled
                if !state.config().auto_compare {
                    return;
                }

                is_computing.set(true);

                // Get current text content
                let (start_a, end_a) = buffer_a.bounds();
                let text_a = buffer_a.text(&start_a, &end_a, true);

                let (start_b, end_b) = buffer_b.bounds();
                let text_b = buffer_b.text(&start_b, &end_b, true);

                // Compute line differences
                let diff_result =
                    incremental_diff_service.compute_line_diff(text_a.as_str(), text_b.as_str());

                // Apply highlighting
                let config = state.config();
                let (empty_lines_a, empty_lines_b) = if config.ignore_whitespace {
                    (Vec::new(), Vec::new())
                } else {
                    (diff_result.empty_lines_a, diff_result.empty_lines_b)
                };

                text_highlighter.apply_line_highlighting(
                    &buffer_a,
                    &buffer_b,
                    &diff_result.changed_lines_a,
                    &diff_result.changed_lines_b,
                    &empty_lines_a,
                    &empty_lines_b,
                    &config,
                );

                // Update diff map
                minimap_clone.set_all_diff_lines(
                    diff_result.changed_lines_a,
                    diff_result.changed_lines_b,
                    empty_lines_a,
                    empty_lines_b,
                );

                // Update status bar
                status_bar_clone.update_status_from_buffers(&buffer_a, &buffer_b, &minimap_clone);

                is_computing.set(false);
            }
        };

        // Function to schedule diff update with debouncing
        let schedule_diff_update = {
            let debounce_token = debounce_token.clone();
            let is_computing = is_computing.clone();
            let perform_diff_update = perform_diff_update.clone();

            move |buffer_a: gtk::TextBuffer, buffer_b: gtk::TextBuffer| {
                // Skip if already computing
                if is_computing.get() {
                    return;
                }

                // Increment token to invalidate previous pending updates
                let current_token = debounce_token.get() + 1;
                debounce_token.set(current_token);

                // Schedule new update with 300ms debounce
                let perform_diff_update = perform_diff_update.clone();
                let debounce_token = debounce_token.clone();
                let _ = glib::timeout_add_local_once(std::time::Duration::from_millis(300), {
                    let buffer_a = buffer_a.clone();
                    let buffer_b = buffer_b.clone();
                    move || {
                        // Only perform update if token hasn't changed
                        if debounce_token.get() == current_token {
                            perform_diff_update(buffer_a, buffer_b);
                        }
                    }
                });
            }
        };

        // Connect buffer change handlers for real-time diff
        buffer_a.connect_changed({
            let buffer_b = buffer_b.clone();
            let schedule_diff_update = schedule_diff_update.clone();

            move |buffer_a| {
                schedule_diff_update(buffer_a.clone(), buffer_b.clone());
            }
        });

        buffer_b.connect_changed({
            let buffer_a = buffer_a.clone();
            let schedule_diff_update = schedule_diff_update.clone();

            move |buffer_b| {
                schedule_diff_update(buffer_a.clone(), buffer_b.clone());
            }
        });
    }
}
